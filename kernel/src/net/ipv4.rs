//! NovaOS Zero — Internet Protocol version 4 (IPv4)
//! RFC 791 Packet Framing, Header Validation & RFC 1071 Internet Checksum

pub const IP_PROTO_ICMP: u8 = 1;
pub const IP_PROTO_TCP: u8  = 6;
pub const IP_PROTO_UDP: u8  = 17;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const LOCALHOST: Self = Self([127, 0, 0, 1]);
    pub const BROADCAST: Self = Self([255, 255, 255, 255]);
    pub const ZERO: Self      = Self([0, 0, 0, 0]);

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }
}

impl core::fmt::Debug for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Ipv4Header {
    pub version_ihl: u8,       // Version (4 bits) | Internet Header Length (4 bits)
    pub type_of_service: u8,
    pub total_length: u16,     // Total packet length in octets
    pub identification: u16,
    pub flags_fragment: u16,
    pub time_to_live: u8,      // TTL hop limit
    pub protocol: u8,          // TCP (6), UDP (17), ICMP (1)
    pub header_checksum: u16,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
}

pub const IPV4_HEADER_MIN_LEN: usize = core::mem::size_of::<Ipv4Header>(); // 20 bytes

pub struct Ipv4Packet<'a> {
    pub header: Ipv4Header,
    pub payload: &'a [u8],
}

impl<'a> Ipv4Packet<'a> {
    /// Parse raw IPv4 packet bytes and verify Internet checksum
    pub fn parse(buffer: &'a [u8]) -> Result<Self, &'static str> {
        if buffer.len() < IPV4_HEADER_MIN_LEN {
            return Err("Packet too short for IPv4 header");
        }

        let header = unsafe { &*(buffer.as_ptr() as *const Ipv4Header) };

        let version = (header.version_ihl >> 4) & 0x0F;
        let ihl = (header.version_ihl & 0x0F) as usize * 4;

        if version != 4 {
            return Err("Not an IPv4 packet");
        }
        if ihl < IPV4_HEADER_MIN_LEN || ihl > buffer.len() {
            return Err("Invalid IHL length");
        }

        let total_len = u16::from_be(header.total_length) as usize;
        if total_len > buffer.len() {
            return Err("Packet total length exceeds buffer size");
        }

        // Verify RFC 1071 Header Checksum
        if calculate_checksum(&buffer[..ihl]) != 0 {
            return Err("Invalid IPv4 header checksum");
        }

        Ok(Ipv4Packet {
            header: *header,
            payload: &buffer[ihl..total_len],
        })
    }

    /// Construct and serialize an IPv4 packet with calculated checksum
    pub fn serialize(
        src: Ipv4Address,
        dst: Ipv4Address,
        protocol: u8,
        payload: &[u8],
        out: &mut [u8],
    ) -> Result<usize, &'static str> {
        let total_len = IPV4_HEADER_MIN_LEN + payload.len();
        if out.len() < total_len {
            return Err("Output buffer too small");
        }

        let mut header = Ipv4Header {
            version_ihl: (4 << 4) | (5 & 0x0F), // IPv4, 5 * 4 = 20 bytes header
            type_of_service: 0,
            total_length: (total_len as u16).to_be(),
            identification: 0x1234u16.to_be(),
            flags_fragment: 0x4000u16.to_be(), // Don't fragment flag
            time_to_live: 64,                  // Standard default TTL
            protocol,
            header_checksum: 0,                // Zeroed prior to checksum computation
            src_ip: src.0,
            dst_ip: dst.0,
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                &header as *const Ipv4Header as *const u8,
                out.as_mut_ptr(),
                IPV4_HEADER_MIN_LEN,
            );
        }

        // Calculate and patch checksum
        let checksum = calculate_checksum(&out[..IPV4_HEADER_MIN_LEN]);
        header.header_checksum = checksum.to_be();

        unsafe {
            core::ptr::copy_nonoverlapping(
                &header as *const Ipv4Header as *const u8,
                out.as_mut_ptr(),
                IPV4_HEADER_MIN_LEN,
            );
        }

        out[IPV4_HEADER_MIN_LEN..total_len].copy_from_slice(payload);
        Ok(total_len)
    }
}

/// Computes standard RFC 1071 16-bit one's complement Internet checksum
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;

    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        sum = sum.wrapping_add(word);
        i += 2;
    }

    if i < data.len() {
        let word = u16::from_be_bytes([data[i], 0]) as u32;
        sum = sum.wrapping_add(word);
    }

    // Fold 32-bit sum to 16-bit
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}
