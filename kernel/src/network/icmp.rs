//! NovaOS Apex — Internet Control Message Protocol (ICMP)
//! Ping/Echo requests and replies with RFC 792 checksum computation.

extern crate alloc;
use alloc::vec::Vec;
use crate::network::ipv4::calculate_checksum;

pub const ICMP_TYPE_ECHO_REPLY: u8 = 0;
pub const ICMP_TYPE_ECHO_REQUEST: u8 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcmpPacket {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub id: u16,
    pub seq: u16,
    pub payload: Vec<u8>,
}

impl IcmpPacket {
    pub fn new_echo_request(id: u16, seq: u16, payload: &[u8]) -> Self {
        let mut pkt = Self {
            icmp_type: ICMP_TYPE_ECHO_REQUEST,
            code: 0,
            checksum: 0,
            id,
            seq,
            payload: payload.to_vec(),
        };
        let raw = pkt.serialize_unchecksummed();
        pkt.checksum = calculate_checksum(&raw);
        pkt
    }

    pub fn new_echo_reply(id: u16, seq: u16, payload: &[u8]) -> Self {
        let mut pkt = Self {
            icmp_type: ICMP_TYPE_ECHO_REPLY,
            code: 0,
            checksum: 0,
            id,
            seq,
            payload: payload.to_vec(),
        };
        let raw = pkt.serialize_unchecksummed();
        pkt.checksum = calculate_checksum(&raw);
        pkt
    }

    fn serialize_unchecksummed(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(8 + self.payload.len());
        raw.push(self.icmp_type);
        raw.push(self.code);
        raw.push((self.checksum >> 8) as u8);
        raw.push((self.checksum & 0xFF) as u8);
        raw.push((self.id >> 8) as u8);
        raw.push((self.id & 0xFF) as u8);
        raw.push((self.seq >> 8) as u8);
        raw.push((self.seq & 0xFF) as u8);
        raw.extend_from_slice(&self.payload);
        raw
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut raw = self.serialize_unchecksummed();
        raw[2] = (self.checksum >> 8) as u8;
        raw[3] = (self.checksum & 0xFF) as u8;
        raw
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }

        let icmp_type = buf[0];
        let code = buf[1];
        let checksum = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let id = ((buf[4] as u16) << 8) | (buf[5] as u16);
        let seq = ((buf[6] as u16) << 8) | (buf[7] as u16);

        let mut payload = Vec::with_capacity(buf.len() - 8);
        payload.extend_from_slice(&buf[8..]);

        Some(Self {
            icmp_type,
            code,
            checksum,
            id,
            seq,
            payload,
        })
    }
}
