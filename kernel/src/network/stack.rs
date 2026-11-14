//! NovaOS Apex — Central Network Stack Coordinator
//! Integrates Ethernet II, ARP, IPv4, ICMP, UDP, and TCP with VirtIO-Net packet drivers.

extern crate alloc;
use alloc::vec::Vec;

use crate::drivers::virtio_net;
use crate::network::arp::{ArpPacket, ARP_CACHE, ARP_OPER_REQUEST};
use crate::network::ethernet::{EthernetFrame, ETHERTYPE_ARP, ETHERTYPE_IPV4};
use crate::network::icmp::{IcmpPacket, ICMP_TYPE_ECHO_REQUEST};
use crate::network::ipv4::{Ipv4Address, Ipv4Header, IP_PROTO_ICMP, IP_PROTO_TCP, IP_PROTO_UDP};
use crate::network::mac::MacAddress;
use crate::network::socket::SOCKET_TABLE;
use crate::network::tcp::TcpPacket;
use crate::network::udp::UdpPacket;
use crate::scheduler::sync::SpinLock;
use crate::serial;

pub struct NetworkInterface {
    pub name: &'static str,
    pub mac: MacAddress,
    pub ip: Ipv4Address,
    pub netmask: Ipv4Address,
    pub gateway: Ipv4Address,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

pub static NET_IFACE: SpinLock<NetworkInterface> = SpinLock::new("net_iface", NetworkInterface {
    name: "eth0",
    mac: MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
    ip: Ipv4Address([10, 0, 2, 15]),
    netmask: Ipv4Address([255, 255, 255, 0]),
    gateway: Ipv4Address([10, 0, 2, 2]),
    rx_packets: 0,
    tx_packets: 0,
});

pub fn init_stack() {
    let raw_mac = virtio_net::get_mac_address();
    let mut iface = NET_IFACE.lock();
    iface.mac = MacAddress(raw_mac);

    // Seed ARP cache with gateway
    ARP_CACHE.lock().insert(iface.gateway, MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x02]));

    serial::write_str("[NETWORK] Network Stack initialized on interface eth0 (IP: 10.0.2.15/24, Gateway: 10.0.2.2)\n");
}

/// Transmit a raw Ethernet II frame out through VirtIO-Net
pub fn transmit_frame(frame: &EthernetFrame) -> Result<(), ()> {
    let raw = frame.serialize();
    if virtio_net::send_packet(&raw).is_ok() {
        NET_IFACE.lock().tx_packets += 1;
        Ok(())
    } else {
        Err(())
    }
}

/// Transmit an IPv4 packet with automatic ARP destination MAC resolution
pub fn send_ipv4(dest_ip: Ipv4Address, protocol: u8, payload: &[u8]) -> Result<(), ()> {
    let iface = NET_IFACE.lock();
    let src_ip = iface.ip;
    let src_mac = iface.mac;

    let dest_mac = if dest_ip.is_broadcast() {
        MacAddress::BROADCAST
    } else if let Some(mac) = ARP_CACHE.lock().lookup(&dest_ip) {
        mac
    } else {
        // Fallback to gateway MAC
        MacAddress([0x52, 0x54, 0x00, 0x12, 0x34, 0x02])
    };
    drop(iface);

    let ip_hdr = Ipv4Header::new(src_ip, dest_ip, protocol, payload.len());
    let mut ip_packet = Vec::with_capacity(20 + payload.len());
    ip_packet.extend_from_slice(&ip_hdr.serialize());
    ip_packet.extend_from_slice(payload);

    let eth_frame = EthernetFrame::new(dest_mac, src_mac, ETHERTYPE_IPV4, ip_packet);
    transmit_frame(&eth_frame)
}

/// Transmit a UDP datagram
pub fn send_udp(src_port: u16, dest_ip: Ipv4Address, dest_port: u16, payload: &[u8]) -> Result<(), ()> {
    let src_ip = NET_IFACE.lock().ip;
    let udp = UdpPacket::new(src_port, dest_port, src_ip, dest_ip, payload);
    send_ipv4(dest_ip, IP_PROTO_UDP, &udp.serialize())
}

/// Transmit an ICMP Echo Request
pub fn send_icmp_ping(dest_ip: Ipv4Address, id: u16, seq: u16, payload: &[u8]) -> Result<(), ()> {
    let icmp = IcmpPacket::new_echo_request(id, seq, payload);
    send_ipv4(dest_ip, IP_PROTO_ICMP, &icmp.serialize())
}

/// Processes an incoming raw packet frame from the network driver
pub fn process_incoming_packet(raw_bytes: &[u8]) {
    NET_IFACE.lock().rx_packets += 1;

    let frame = match EthernetFrame::parse(raw_bytes) {
        Some(f) => f,
        None => return,
    };

    match frame.ethertype {
        ETHERTYPE_ARP => {
            if let Some(arp) = ArpPacket::parse(&frame.payload) {
                // Update ARP cache with sender
                ARP_CACHE.lock().insert(arp.sender_ip, arp.sender_mac);

                let my_ip = NET_IFACE.lock().ip;
                let my_mac = NET_IFACE.lock().mac;

                // If this is a request for our IP, send reply
                if arp.oper == ARP_OPER_REQUEST && arp.target_ip == my_ip {
                    let reply = ArpPacket::new_reply(my_mac, my_ip, arp.sender_mac, arp.sender_ip);
                    let reply_frame = EthernetFrame::new(
                        arp.sender_mac,
                        my_mac,
                        ETHERTYPE_ARP,
                        reply.serialize().to_vec(),
                    );
                    let _ = transmit_frame(&reply_frame);
                }
            }
        }

        ETHERTYPE_IPV4 => {
            if let Some((ip_hdr, hdr_len)) = Ipv4Header::parse(&frame.payload) {
                let my_ip = NET_IFACE.lock().ip;
                let is_for_us = ip_hdr.dest_ip == my_ip
                    || ip_hdr.dest_ip.is_broadcast()
                    || ip_hdr.dest_ip.is_loopback();

                if !is_for_us {
                    return;
                }

                let l4_payload = &frame.payload[hdr_len..];

                match ip_hdr.protocol {
                    IP_PROTO_ICMP => {
                        if let Some(icmp) = IcmpPacket::parse(l4_payload) {
                            if icmp.icmp_type == ICMP_TYPE_ECHO_REQUEST {
                                // Send ICMP Echo Reply
                                let reply = IcmpPacket::new_echo_reply(icmp.id, icmp.seq, &icmp.payload);
                                let _ = send_ipv4(ip_hdr.src_ip, IP_PROTO_ICMP, &reply.serialize());
                            }
                        }
                    }

                    IP_PROTO_UDP => {
                        if let Some(udp) = UdpPacket::parse(l4_payload) {
                            // Find socket bound to udp.dest_port
                            let mut st = SOCKET_TABLE.lock();
                            for (_, sock) in st.sockets.iter_mut() {
                                if sock.sock_type == crate::network::socket::SOCK_DGRAM
                                    && sock.local_port == udp.dest_port
                                {
                                    sock.rx_queue.push(udp.payload.clone());
                                    break;
                                }
                            }
                        }
                    }

                    IP_PROTO_TCP => {
                        if let Some(tcp) = TcpPacket::parse(l4_payload) {
                            let mut st = SOCKET_TABLE.lock();
                            for (_, sock) in st.sockets.iter_mut() {
                                if sock.sock_type == crate::network::socket::SOCK_STREAM
                                    && (sock.local_port == tcp.dest_port || sock.local_port == 0)
                                {
                                    if let Some(conn) = &mut sock.tcp_conn {
                                        if let Some(resp) = conn.handle_packet(&tcp) {
                                            drop(st);
                                            let _ = send_ipv4(ip_hdr.src_ip, IP_PROTO_TCP, &resp.serialize());
                                            return;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        _ => {}
    }
}
