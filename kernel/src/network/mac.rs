//! NovaOS Apex — Media Access Control (MAC) Address Abstraction
//! Represents 48-bit IEEE 802.3 Ethernet MAC addresses with formatting and comparison.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    pub const ZERO: Self = Self([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    pub const fn new(b: [u8; 6]) -> Self {
        Self(b)
    }

    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    pub fn is_multicast(&self) -> bool {
        (self.0[0] & 0x01) != 0
    }

    pub fn is_unicast(&self) -> bool {
        !self.is_multicast() && !self.is_broadcast()
    }
}
