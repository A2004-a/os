//! NovaOS Zero — Address Resolution Protocol (ARP)
//! RFC 826 Ethernet-to-IPv4 Hardware Address Resolution & Dynamic Cache

use super::ethernet::MacAddress;

pub const ARP_HTYPE_ETHERNET: u16 = 1;
pub const ARP_PTYPE_IPV4: u16     = 0x0800;
pub const ARP_OP_REQUEST: u16     = 1;
pub const ARP_OP_REPLY: u16       = 2;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct ArpPacket {
    pub htype: u16,        // Hardware Type (1 for Ethernet)
    pub ptype: u16,        // Protocol Type (0x0800 for IPv4)
    pub hlen: u8,          // Hardware address length (6)
    pub plen: u8,          // Protocol address length (4)
    pub opcode: u16,       // 1 = Request, 2 = Reply
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

pub const ARP_PACKET_LEN: usize = core::mem::size_of::<ArpPacket>(); // 28 bytes

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArpEntry {
    pub ip: [u8; 4],
    pub mac: MacAddress,
    pub is_valid: bool,
}

pub const ARP_CACHE_CAPACITY: usize = 16;

pub struct ArpCache {
    entries: [ArpEntry; ARP_CACHE_CAPACITY],
}

impl ArpCache {
    pub const fn new() -> Self {
        ArpCache {
            entries: [ArpEntry {
                ip: [0; 4],
                mac: MacAddress::ZERO,
                is_valid: false,
            }; ARP_CACHE_CAPACITY],
        }
    }

    /// Update or insert an IP-to-MAC resolution into the cache
    pub fn update(&mut self, ip: [u8; 4], mac: MacAddress) {
        // Check if IP already exists
        for entry in self.entries.iter_mut() {
            if entry.is_valid && entry.ip == ip {
                entry.mac = mac;
                return;
            }
        }

        // Insert into first free slot
        for entry in self.entries.iter_mut() {
            if !entry.is_valid {
                *entry = ArpEntry {
                    ip,
                    mac,
                    is_valid: true,
                };
                return;
            }
        }

        // Evict first slot if full
        self.entries[0] = ArpEntry { ip, mac, is_valid: true };
    }

    /// Lookup MAC address corresponding to an IPv4 destination
    pub fn lookup(&self, ip: [u8; 4]) -> Option<MacAddress> {
        for entry in self.entries.iter() {
            if entry.is_valid && entry.ip == ip {
                return Some(entry.mac);
            }
        }
        None
    }
}

impl ArpPacket {
    pub fn new_request(sender_mac: MacAddress, sender_ip: [u8; 4], target_ip: [u8; 4]) -> Self {
        ArpPacket {
            htype: ARP_HTYPE_ETHERNET.to_be(),
            ptype: ARP_PTYPE_IPV4.to_be(),
            hlen: 6,
            plen: 4,
            opcode: ARP_OP_REQUEST.to_be(),
            sender_mac: sender_mac.0,
            sender_ip,
            target_mac: [0; 6],
            target_ip,
        }
    }

    pub fn new_reply(sender_mac: MacAddress, sender_ip: [u8; 4], target_mac: MacAddress, target_ip: [u8; 4]) -> Self {
        ArpPacket {
            htype: ARP_HTYPE_ETHERNET.to_be(),
            ptype: ARP_PTYPE_IPV4.to_be(),
            hlen: 6,
            plen: 4,
            opcode: ARP_OP_REPLY.to_be(),
            sender_mac: sender_mac.0,
            sender_ip,
            target_mac: target_mac.0,
            target_ip,
        }
    }

    pub fn parse(buf: &[u8]) -> Result<&Self, &'static str> {
        if buf.len() < ARP_PACKET_LEN {
            return Err("Packet smaller than ARP header");
        }
        let pkt = unsafe { &*(buf.as_ptr() as *const ArpPacket) };
        if u16::from_be(pkt.htype) != ARP_HTYPE_ETHERNET {
            return Err("Unsupported hardware type");
        }
        if u16::from_be(pkt.ptype) != ARP_PTYPE_IPV4 {
            return Err("Unsupported protocol type");
        }
        Ok(pkt)
    }
}
