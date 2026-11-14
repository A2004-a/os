//! NovaOS Apex — IEEE 802.3 CRC32 Implementation
//! Used for UEFI GPT header and partition array integrity checksum verification.

/// Standard IEEE 802.3 CRC32 polynomial (reversed / little-endian representation).
pub const CRC32_POLYNOMIAL: u32 = 0xEDB8_8320;

/// Computes the IEEE 802.3 32-bit Cyclic Redundancy Check (CRC32) over the given byte slice.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ CRC32_POLYNOMIAL;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

pub fn test_crc32() -> bool {
    // Standard test vector: "123456789" -> 0xCBF43926
    let vector = b"123456789";
    crc32(vector) == 0xCBF4_3926
}
