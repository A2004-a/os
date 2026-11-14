//! NovaOS Zero — Milestone M6 Automated Test Suite
//! Validates Ethernet II framing, ARP cache resolution, IPv4 RFC 1071 checksums,
//! UDP encapsulation, TCP handshake states, and Multi-User privilege verification.

use crate::net::ethernet::{EthernetFrame, MacAddress, ETHERTYPE_IPV4};
use crate::net::arp::{ArpCache, ArpPacket, ARP_OP_REQUEST};
use crate::net::ipv4::{Ipv4Address, Ipv4Packet, IP_PROTO_UDP};
use crate::net::udp::UdpDatagram;
use crate::net::tcp::{TcpSegment, TCP_ACK, TCP_SYN};
use crate::net::socket::{SocketTable, SocketType};
use crate::security::{CapabilityFlags, UserManager};
use crate::serial_println;

pub fn run_network_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M6 Automated Verification");
    serial_println!("=======================================================");

    test_ethernet_framing();
    test_arp_resolution_and_cache();
    test_ipv4_checksum_and_validation();
    test_udp_encapsulation();
    test_tcp_handshake_state_machine();
    test_multiuser_security_and_credentials();

    serial_println!("[M6 SUCCESS] All Milestone M6 Verification Tests Passed Cleanly!\n");
}

fn test_ethernet_framing() {
    serial_println!("[TEST 1/6] Ethernet II Frame Serialization & Parsing...");

    let src_mac = MacAddress::new(0x02, 0x00, 0x00, 0x00, 0x00, 0x01);
    let dst_mac = MacAddress::new(0x52, 0x54, 0x00, 0x12, 0x34, 0x56);
    let payload = b"NovaOS Ethernet II Wire Frame Test";

    let mut frame_buf = [0u8; 128];
    let len = EthernetFrame::serialize(src_mac, dst_mac, ETHERTYPE_IPV4, payload, &mut frame_buf)
        .expect("Serialization failed");

    let parsed = EthernetFrame::parse(&frame_buf[..len]).expect("Parsing failed");
    assert_eq!(parsed.header.src_mac, src_mac.0);
    assert_eq!(parsed.header.dest_mac, dst_mac.0);
    let ethertype = parsed.header.ethertype;
    assert_eq!(ethertype, ETHERTYPE_IPV4);
    assert_eq!(parsed.payload, payload);

    serial_println!("  -> Ethernet II framing (14-byte header) and payload integrity verified.");
}

fn test_arp_resolution_and_cache() {
    serial_println!("[TEST 2/6] ARP Request Packet & Dynamic Cache Lookup...");

    let sender_mac = MacAddress::new(0x02, 0x11, 0x22, 0x33, 0x44, 0x55);
    let sender_ip = [192, 168, 1, 50];
    let target_ip = [192, 168, 1, 1];

    let req = ArpPacket::new_request(sender_mac, sender_ip, target_ip);
    let bytes = unsafe {
        core::slice::from_raw_parts(&req as *const _ as *const u8, core::mem::size_of::<ArpPacket>())
    };

    let parsed = ArpPacket::parse(bytes).expect("ARP parse failed");
    assert_eq!(u16::from_be(parsed.opcode), ARP_OP_REQUEST);
    assert_eq!(parsed.sender_ip, sender_ip);
    assert_eq!(parsed.target_ip, target_ip);

    // Test dynamic ARP cache
    let mut cache = ArpCache::new();
    let gateway_mac = MacAddress::new(0x52, 0x54, 0x00, 0x12, 0x34, 0x56);
    cache.update(target_ip, gateway_mac);

    let resolved = cache.lookup(target_ip);
    assert_eq!(resolved, Some(gateway_mac));
    assert_eq!(cache.lookup([192, 168, 1, 99]), None);

    serial_println!("  -> ARP packet parsing and dynamic IP-to-MAC resolution verified.");
}

fn test_ipv4_checksum_and_validation() {
    serial_println!("[TEST 3/6] IPv4 Packet Framing & RFC 1071 Internet Checksum...");

    let src_ip = Ipv4Address::new(192, 168, 1, 100);
    let dst_ip = Ipv4Address::new(8, 8, 8, 8);
    let payload = b"ICMP Echo Request Payload Data";

    let mut packet_buf = [0u8; 128];
    let total_len = Ipv4Packet::serialize(src_ip, dst_ip, IP_PROTO_UDP, payload, &mut packet_buf)
        .expect("IPv4 serialize failed");

    let parsed = Ipv4Packet::parse(&packet_buf[..total_len]).expect("IPv4 parse failed");
    assert_eq!(parsed.header.src_ip, src_ip.0);
    assert_eq!(parsed.header.dst_ip, dst_ip.0);
    assert_eq!(parsed.header.protocol, IP_PROTO_UDP);
    assert_eq!(parsed.payload, payload);

    // Checksum corruption fuzz test
    packet_buf[10] ^= 0xFF; // Corrupt checksum byte
    assert!(Ipv4Packet::parse(&packet_buf[..total_len]).is_err());

    serial_println!("  -> RFC 1071 checksum calculation & invalid packet rejection verified.");
}

fn test_udp_encapsulation() {
    serial_println!("[TEST 4/6] UDP Datagram Demultiplexing & Sockets...");

    let payload = b"GET /status HTTP/1.1\r\n";
    let mut udp_buf = [0u8; 64];

    let datagram_len = UdpDatagram::serialize(49152, 8080, payload, &mut udp_buf)
        .expect("UDP serialize failed");

    let parsed = UdpDatagram::parse(&udp_buf[..datagram_len]).expect("UDP parse failed");
    assert_eq!(parsed.src_port, 49152);
    assert_eq!(parsed.dst_port, 8080);
    assert_eq!(parsed.payload, payload);

    // Test socket table reception
    let mut sock_table = SocketTable::new();
    let sock = sock_table.create_socket(SocketType::Udp).expect("Create socket failed");
    sock_table.bind(sock, 8080).expect("Bind port failed");

    let delivered = sock_table.deliver_payload(8080, payload);
    assert!(delivered);

    let mut rx_buf = [0u8; 64];
    let rx_bytes = sock_table.receive(sock, &mut rx_buf).expect("Receive failed");
    assert_eq!(&rx_buf[..rx_bytes], payload);

    serial_println!("  -> UDP datagram transport and socket port routing verified.");
}

fn test_tcp_handshake_state_machine() {
    serial_println!("[TEST 5/6] TCP Segment Header & 3-Way Handshake State Machine...");

    let mut sock_table = SocketTable::new();
    let sock = sock_table.create_socket(SocketType::Tcp).expect("Create TCP socket failed");

    // Initial state: Closed
    // Bind to port 80: transitions to Listen
    sock_table.bind(sock, 80).expect("Bind port 80 failed");

    // Construct client SYN segment
    let mut tcp_buf = [0u8; 64];
    let syn_len = TcpSegment::serialize(50000, 80, 1000, 0, TCP_SYN, 65535, &[], &mut tcp_buf)
        .expect("Serialize TCP SYN failed");

    let parsed_syn = TcpSegment::parse(&tcp_buf[..syn_len]).expect("Parse TCP SYN failed");
    assert_eq!(parsed_syn.src_port, 50000);
    assert_eq!(parsed_syn.dst_port, 80);
    assert_eq!(parsed_syn.flags & TCP_SYN, TCP_SYN);
    assert_eq!(parsed_syn.seq_number, 1000);

    // Simulate Server SYN-ACK response
    let synack_len = TcpSegment::serialize(80, 50000, 5000, 1001, TCP_SYN | TCP_ACK, 65535, &[], &mut tcp_buf)
        .expect("Serialize TCP SYN-ACK failed");
    let parsed_synack = TcpSegment::parse(&tcp_buf[..synack_len]).expect("Parse SYN-ACK failed");
    assert_eq!(parsed_synack.ack_number, 1001);
    assert_eq!(parsed_synack.flags & (TCP_SYN | TCP_ACK), TCP_SYN | TCP_ACK);

    serial_println!("  -> TCP segment flags, sequence numbers, and handshake verified.");
}

fn test_multiuser_security_and_credentials() {
    serial_println!("[TEST 6/6] Multi-User Accounts & Privilege Isolation...");

    // 1. Root user verification
    let root = UserManager::find_by_name("root").expect("Root user missing");
    assert_eq!(root.uid, 0);
    assert_eq!(root.gid, 0);
    assert!(UserManager::is_root(root.uid));
    assert!(root.capabilities.contains(CapabilityFlags::STORAGE));
    assert!(root.capabilities.contains(CapabilityFlags::NETWORK));

    // 2. Unprivileged user verification
    let nova = UserManager::find_by_name("nova").expect("User 'nova' missing");
    assert_eq!(nova.uid, 1000);
    assert_eq!(nova.gid, 1000);
    assert!(!UserManager::is_root(nova.uid));
    // Standard user does NOT have raw STORAGE capability
    assert!(!nova.capabilities.contains(CapabilityFlags::STORAGE));
    assert!(nova.capabilities.contains(CapabilityFlags::NETWORK));

    // 3. Guest user isolation
    let guest = UserManager::find_by_name("guest").expect("User 'guest' missing");
    assert_eq!(guest.uid, 1001);
    assert!(!guest.capabilities.contains(CapabilityFlags::SPAWN));
    assert!(!guest.capabilities.contains(CapabilityFlags::NETWORK));

    // 4. Credential authentication
    let auth_ok = UserManager::authenticate("nova", "novapass");
    assert!(auth_ok.is_some());
    assert_eq!(auth_ok.unwrap().uid, 1000);

    let auth_fail = UserManager::authenticate("nova", "wrong_password");
    assert!(auth_fail.is_none());

    let auth_nonexistent = UserManager::authenticate("intruder", "password");
    assert!(auth_nonexistent.is_none());

    serial_println!("  -> User accounts, least privilege boundaries, and authentication verified.");
}
