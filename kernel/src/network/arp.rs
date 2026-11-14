//! NovaOS Apex — Address Resolution Protocol (ARP)
//! IP-to-MAC resolution, ARP request/reply generation, and ARP cache management.

extern crate alloc;
use alloc::collections::BTreeMap;
use crate::network::ipv4::Ipv4Address;
use crate::network::mac::MacAddress;
use crate::scheduler::sync::SpinLock;

pub const ARP_HTYPE_ETHERNET: u16 = 1;
pub const ARP_PTYPE_IPV4: u16 = 0x0800;
pub const ARP_OPER_REQUEST: u16 = 1;
pub const ARP_OPER_REPLY: u16 = 2;

pub const ARP_PACKET_LEN: usize = 28;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArpPacket {
    pub htype: u16,
    pub ptype: u16,
    pub hlen: u8,
    pub plen: u8,
    pub oper: u16,
    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Address,
    pub target_mac: MacAddress,
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    pub fn new_request(sender_mac: MacAddress, sender_ip: Ipv4Address, target_ip: Ipv4Address) -> Self {
        Self {
            htype: ARP_HTYPE_ETHERNET,
            ptype: ARP_PTYPE_IPV4,
            hlen: 6,
            plen: 4,
            oper: ARP_OPER_REQUEST,
            sender_mac,
            sender_ip,
            target_mac: MacAddress::ZERO,
            target_ip,
        }
    }

    pub fn new_reply(
        sender_mac: MacAddress,
        sender_ip: Ipv4Address,
        target_mac: MacAddress,
        target_ip: Ipv4Address,
    ) -> Self {
        Self {
            htype: ARP_HTYPE_ETHERNET,
            ptype: ARP_PTYPE_IPV4,
            hlen: 6,
            plen: 4,
            oper: ARP_OPER_REPLY,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }

    pub fn serialize(&self) -> [u8; ARP_PACKET_LEN] {
        let mut buf = [0u8; ARP_PACKET_LEN];
        buf[0] = (self.htype >> 8) as u8;
        buf[1] = (self.htype & 0xFF) as u8;
        buf[2] = (self.ptype >> 8) as u8;
        buf[3] = (self.ptype & 0xFF) as u8;
        buf[4] = self.hlen;
        buf[5] = self.plen;
        buf[6] = (self.oper >> 8) as u8;
        buf[7] = (self.oper & 0xFF) as u8;

        buf[8..14].copy_from_slice(&self.sender_mac.0);
        buf[14..18].copy_from_slice(&self.sender_ip.0);
        buf[18..24].copy_from_slice(&self.target_mac.0);
        buf[24..28].copy_from_slice(&self.target_ip.0);
        buf
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < ARP_PACKET_LEN {
            return None;
        }

        let htype = ((buf[0] as u16) << 8) | (buf[1] as u16);
        let ptype = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let hlen = buf[4];
        let plen = buf[5];
        let oper = ((buf[6] as u16) << 8) | (buf[7] as u16);

        if htype != ARP_HTYPE_ETHERNET || ptype != ARP_PTYPE_IPV4 || hlen != 6 || plen != 4 {
            return None;
        }

        let mut smac = [0u8; 6];
        let mut sip = [0u8; 4];
        let mut tmac = [0u8; 6];
        let mut tip = [0u8; 4];

        smac.copy_from_slice(&buf[8..14]);
        sip.copy_from_slice(&buf[14..18]);
        tmac.copy_from_slice(&buf[18..24]);
        tip.copy_from_slice(&buf[24..28]);

        Some(Self {
            htype,
            ptype,
            hlen,
            plen,
            oper,
            sender_mac: MacAddress(smac),
            sender_ip: Ipv4Address(sip),
            target_mac: MacAddress(tmac),
            target_ip: Ipv4Address(tip),
        })
    }
}

pub struct ArpCache {
    entries: BTreeMap<Ipv4Address, MacAddress>,
}

impl ArpCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress) {
        self.entries.insert(ip, mac);
    }

    pub fn lookup(&self, ip: &Ipv4Address) -> Option<MacAddress> {
        self.entries.get(ip).copied()
    }
}

pub static ARP_CACHE: SpinLock<ArpCache> = SpinLock::new("arp_cache", ArpCache {
    entries: BTreeMap::new(),
});
