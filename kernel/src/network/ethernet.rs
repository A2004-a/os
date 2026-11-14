//! NovaOS Apex — Ethernet II Framing
//! IEEE 802.3 MAC frame encapsulation, EtherType multiplexing, and packet validation.

extern crate alloc;
use alloc::vec::Vec;
use crate::network::mac::MacAddress;

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;

pub const ETHERNET_HEADER_LEN: usize = 14;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EthernetFrame {
    pub dest_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ethertype: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn new(dest_mac: MacAddress, src_mac: MacAddress, ethertype: u16, payload: Vec<u8>) -> Self {
        Self {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(ETHERNET_HEADER_LEN + self.payload.len());
        raw.extend_from_slice(&self.dest_mac.0);
        raw.extend_from_slice(&self.src_mac.0);
        raw.push((self.ethertype >> 8) as u8);
        raw.push((self.ethertype & 0xFF) as u8);
        raw.extend_from_slice(&self.payload);
        raw
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < ETHERNET_HEADER_LEN {
            return None;
        }

        let mut dest = [0u8; 6];
        let mut src = [0u8; 6];
        dest.copy_from_slice(&buf[0..6]);
        src.copy_from_slice(&buf[6..12]);
        let ethertype = ((buf[12] as u16) << 8) | (buf[13] as u16);

        let mut payload = Vec::with_capacity(buf.len() - ETHERNET_HEADER_LEN);
        payload.extend_from_slice(&buf[ETHERNET_HEADER_LEN..]);

        Some(Self {
            dest_mac: MacAddress(dest),
            src_mac: MacAddress(src),
            ethertype,
            payload,
        })
    }
}
