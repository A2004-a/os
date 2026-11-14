//! NovaOS Apex — Internet Protocol Version 4 (IPv4)
//! Address abstractions, packet header definitions, and RFC 1071 Internet checksums.

extern crate alloc;

pub const IP_PROTO_ICMP: u8 = 1;
pub const IP_PROTO_TCP: u8 = 6;
pub const IP_PROTO_UDP: u8 = 17;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const LOCALHOST: Self = Self([127, 0, 0, 1]);
    pub const BROADCAST: Self = Self([255, 255, 255, 255]);
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    pub fn is_loopback(&self) -> bool {
        self.0[0] == 127
    }

    pub fn is_broadcast(&self) -> bool {
        self.0 == [255, 255, 255, 255]
    }
}

/// Calculates RFC 1071 16-bit one's complement Internet checksum
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = ((data[i] as u32) << 8) | (data[i + 1] as u32);
        sum = sum.wrapping_add(word);
        i += 2;
    }
    if i < data.len() {
        sum = sum.wrapping_add((data[i] as u32) << 8);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ipv4Header {
    pub version: u8,
    pub ihl: u8,
    pub tos: u8,
    pub total_len: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_ip: Ipv4Address,
    pub dest_ip: Ipv4Address,
}

impl Ipv4Header {
    pub fn new(src_ip: Ipv4Address, dest_ip: Ipv4Address, protocol: u8, payload_len: usize) -> Self {
        let total_len = (20 + payload_len) as u16;
        let mut header = Self {
            version: 4,
            ihl: 5,
            tos: 0,
            total_len,
            identification: 0x1234,
            flags: 0x02, // Don't fragment
            fragment_offset: 0,
            ttl: 64,
            protocol,
            checksum: 0,
            src_ip,
            dest_ip,
        };
        let raw = header.serialize();
        header.checksum = calculate_checksum(&raw);
        header
    }

    pub fn serialize(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0] = (self.version << 4) | (self.ihl & 0x0F);
        buf[1] = self.tos;
        buf[2] = (self.total_len >> 8) as u8;
        buf[3] = (self.total_len & 0xFF) as u8;
        buf[4] = (self.identification >> 8) as u8;
        buf[5] = (self.identification & 0xFF) as u8;

        let flags_frag = ((self.flags as u16 & 0x07) << 13) | (self.fragment_offset & 0x1FFF);
        buf[6] = (flags_frag >> 8) as u8;
        buf[7] = (flags_frag & 0xFF) as u8;

        buf[8] = self.ttl;
        buf[9] = self.protocol;
        buf[10] = (self.checksum >> 8) as u8;
        buf[11] = (self.checksum & 0xFF) as u8;

        buf[12..16].copy_from_slice(&self.src_ip.0);
        buf[16..20].copy_from_slice(&self.dest_ip.0);
        buf
    }

    pub fn parse(buf: &[u8]) -> Option<(Self, usize)> {
        if buf.len() < 20 {
            return None;
        }

        let version = buf[0] >> 4;
        let ihl = buf[0] & 0x0F;
        if version != 4 || ihl < 5 {
            return None;
        }

        let header_len = (ihl as usize) * 4;
        if buf.len() < header_len {
            return None;
        }

        let tos = buf[1];
        let total_len = ((buf[2] as u16) << 8) | (buf[3] as u16);
        let identification = ((buf[4] as u16) << 8) | (buf[5] as u16);
        let flags_frag = ((buf[6] as u16) << 8) | (buf[7] as u16);
        let flags = ((flags_frag >> 13) & 0x07) as u8;
        let fragment_offset = flags_frag & 0x1FFF;
        let ttl = buf[8];
        let protocol = buf[9];
        let checksum = ((buf[10] as u16) << 8) | (buf[11] as u16);

        let mut src = [0u8; 4];
        let mut dest = [0u8; 4];
        src.copy_from_slice(&buf[12..16]);
        dest.copy_from_slice(&buf[16..20]);

        Some((
            Self {
                version,
                ihl,
                tos,
                total_len,
                identification,
                flags,
                fragment_offset,
                ttl,
                protocol,
                checksum,
                src_ip: Ipv4Address(src),
                dest_ip: Ipv4Address(dest),
            },
            header_len,
        ))
    }
}
