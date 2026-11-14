//! NovaOS Zero — Ethernet II Data Link Layer
//! Implements 48-bit MAC addresses and 14-byte Ethernet II frame encapsulation.

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16  = 0x0806;
pub const ETHERTYPE_IPV6: u16 = 0x86DD;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    pub const ZERO: Self      = Self([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    pub const fn new(b0: u8, b1: u8, b2: u8, b3: u8, b4: u8, b5: u8) -> Self {
        Self([b0, b1, b2, b3, b4, b5])
    }

    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST
    }

    pub fn is_multicast(&self) -> bool {
        (self.0[0] & 1) != 0
    }
}

impl core::fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct EthernetHeader {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16, // Big-endian on the wire
}

pub const ETHERNET_HEADER_LEN: usize = core::mem::size_of::<EthernetHeader>(); // 14 bytes

pub struct EthernetFrame<'a> {
    pub header: EthernetHeader,
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    /// Parse raw Ethernet II frame bytes from wire
    pub fn parse(buffer: &'a [u8]) -> Result<Self, &'static str> {
        if buffer.len() < ETHERNET_HEADER_LEN {
            return Err("Frame shorter than Ethernet header");
        }

        let header = unsafe { &*(buffer.as_ptr() as *const EthernetHeader) };
        let ethertype = u16::from_be(header.ethertype);

        Ok(EthernetFrame {
            header: EthernetHeader {
                dest_mac: header.dest_mac,
                src_mac: header.src_mac,
                ethertype,
            },
            payload: &buffer[ETHERNET_HEADER_LEN..],
        })
    }

    /// Serialize Ethernet II frame into raw byte buffer
    pub fn serialize(
        src: MacAddress,
        dest: MacAddress,
        ethertype: u16,
        payload: &[u8],
        out: &mut [u8],
    ) -> Result<usize, &'static str> {
        let total_len = ETHERNET_HEADER_LEN + payload.len();
        if out.len() < total_len {
            return Err("Output buffer too small");
        }

        let hdr = EthernetHeader {
            dest_mac: dest.0,
            src_mac: src.0,
            ethertype: ethertype.to_be(),
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                &hdr as *const EthernetHeader as *const u8,
                out.as_mut_ptr(),
                ETHERNET_HEADER_LEN,
            );
        }

        out[ETHERNET_HEADER_LEN..total_len].copy_from_slice(payload);
        Ok(total_len)
    }
}
