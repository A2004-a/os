//! NovaOS Apex — User Datagram Protocol (UDP)
//! Connectionless transport datagrams with pseudo-header checksum validation.

extern crate alloc;
use alloc::vec::Vec;
use crate::network::ipv4::{calculate_checksum, Ipv4Address, IP_PROTO_UDP};

pub const UDP_HEADER_LEN: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UdpPacket {
    pub src_port: u16,
    pub dest_port: u16,
    pub length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn new(
        src_port: u16,
        dest_port: u16,
        src_ip: Ipv4Address,
        dest_ip: Ipv4Address,
        payload: &[u8],
    ) -> Self {
        let length = (UDP_HEADER_LEN + payload.len()) as u16;
        let mut pkt = Self {
            src_port,
            dest_port,
            length,
            checksum: 0,
            payload: payload.to_vec(),
        };

        // Compute UDP pseudo-header checksum
        let mut pseudo = Vec::with_capacity(12 + UDP_HEADER_LEN + payload.len());
        pseudo.extend_from_slice(&src_ip.0);
        pseudo.extend_from_slice(&dest_ip.0);
        pseudo.push(0);
        pseudo.push(IP_PROTO_UDP);
        pseudo.push((length >> 8) as u8);
        pseudo.push((length & 0xFF) as u8);

        // UDP Header
        pseudo.push((src_port >> 8) as u8);
        pseudo.push((src_port & 0xFF) as u8);
        pseudo.push((dest_port >> 8) as u8);
        pseudo.push((dest_port & 0xFF) as u8);
        pseudo.push((length >> 8) as u8);
        pseudo.push((length & 0xFF) as u8);
        pseudo.push(0); // Checksum placeholder
        pseudo.push(0);

        pseudo.extend_from_slice(payload);

        let chk = calculate_checksum(&pseudo);
        pkt.checksum = if chk == 0 { 0xFFFF } else { chk };
        pkt
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(UDP_HEADER_LEN + self.payload.len());
        raw.push((self.src_port >> 8) as u8);
        raw.push((self.src_port & 0xFF) as u8);
        raw.push((self.dest_port >> 8) as u8);
        raw.push((self.dest_port & 0xFF) as u8);
        raw.push((self.length >> 8) as u8);
        raw.push((self.length & 0xFF) as u8);
        raw.push((self.checksum >> 8) as u8);
        raw.push((self.checksum & 0xFF) as u8);
        raw.extend_from_slice(&self.payload);
        raw
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < UDP_HEADER_LEN {
            return None;
        }

        let src_port = ((buf[0] as u16) << 8) | (buf[1] as u16);
        let dest_port = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let length = ((buf[4] as u16) << 8) | (buf[5] as u16);
        let checksum = ((buf[6] as u16) << 8) | (buf[7] as u16);

        let mut payload = Vec::with_capacity(buf.len() - UDP_HEADER_LEN);
        payload.extend_from_slice(&buf[UDP_HEADER_LEN..]);

        Some(Self {
            src_port,
            dest_port,
            length,
            checksum,
            payload,
        })
    }
}
