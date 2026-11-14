//! NovaOS Apex — GUID Partition Table (GPT) Engine
//! Implements UEFI 2.10 GPT specifications, Protective MBR, partition entry structures,
//! CRC32 validation, and disk layout management.

extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;

use super::crc32::crc32;

pub const SECTOR_SIZE: usize = 512;
pub const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";
pub const GPT_REVISION_1_0: u32 = 0x0001_0000;
pub const GPT_HEADER_SIZE: u32 = 92;
pub const DEFAULT_NUM_PARTITIONS: u32 = 128;
pub const PARTITION_ENTRY_SIZE: u32 = 128;
pub const PARTITION_ARRAY_SECTORS: u64 = (DEFAULT_NUM_PARTITIONS as u64 * PARTITION_ENTRY_SIZE as u64) / SECTOR_SIZE as u64; // 32 sectors

// Standard Partition Type GUIDs
pub const GUID_UNUSED: [u8; 16] = [0u8; 16];

// EFI System Partition: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
pub const GUID_ESP: [u8; 16] = [
    0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B,
];

// NovaOS Apex System Partition (Read-Only Kernel & Services): A19F2001-A7E8-4D02-9102-000000000001
pub const GUID_NOVAOS_SYSTEM: [u8; 16] = [
    0x01, 0x20, 0x9F, 0xA1, 0xE8, 0xA7, 0x02, 0x4D, 0x91, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

// NovaOS Apex Data / Storage Partition (NovaFS Read-Write): A19F2002-A7E8-4D02-9102-000000000002
pub const GUID_NOVAOS_DATA: [u8; 16] = [
    0x02, 0x20, 0x9F, 0xA1, 0xE8, 0xA7, 0x02, 0x4D, 0x91, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
];

// NovaOS Apex Recovery Partition: A19F2003-A7E8-4D02-9102-000000000003
pub const GUID_NOVAOS_RECOVERY: [u8; 16] = [
    0x03, 0x20, 0x9F, 0xA1, 0xE8, 0xA7, 0x02, 0x4D, 0x91, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
];

// Partition Attributes
pub const PART_ATTR_SYSTEM: u64 = 1 << 0;
pub const PART_ATTR_BOOTABLE: u64 = 1 << 1;
pub const PART_ATTR_READ_ONLY: u64 = 1 << 60;

#[derive(Debug, PartialEq, Eq)]
pub enum GptError {
    InvalidSignature,
    InvalidRevision,
    InvalidHeaderSize,
    HeaderCrcMismatch,
    PartitionArrayCrcMismatch,
    OutOfBounds,
    BufferTooSmall,
    SectorIoError,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GptHeader {
    pub signature: [u8; 8],
    pub revision: u32,
    pub header_size: u32,
    pub header_crc32: u32,
    pub reserved: u32,
    pub my_lba: u64,
    pub alternate_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entry_lba: u64,
    pub num_partition_entries: u32,
    pub size_partition_entry: u32,
    pub partition_entry_array_crc32: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartitionEntry {
    pub type_guid: [u8; 16],
    pub unique_guid: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub attributes: u64,
    pub name: [u16; 36], // UTF-16LE
}

impl PartitionEntry {
    pub const fn empty() -> Self {
        Self {
            type_guid: GUID_UNUSED,
            unique_guid: [0u8; 16],
            starting_lba: 0,
            ending_lba: 0,
            attributes: 0,
            name: [0u16; 36],
        }
    }

    pub fn is_used(&self) -> bool {
        self.type_guid != GUID_UNUSED
    }

    pub fn size_sectors(&self) -> u64 {
        if self.ending_lba >= self.starting_lba {
            self.ending_lba - self.starting_lba + 1
        } else {
            0
        }
    }

    pub fn set_name(&mut self, ascii_name: &str) {
        let mut idx = 0;
        for b in ascii_name.bytes() {
            if idx >= 35 {
                break;
            }
            self.name[idx] = b as u16;
            idx += 1;
        }
        while idx < 36 {
            self.name[idx] = 0;
            idx += 1;
        }
    }

    pub fn get_name_ascii(&self, buf: &mut [u8]) -> usize {
        let mut written = 0;
        for &u in &self.name {
            if u == 0 || written >= buf.len() {
                break;
            }
            buf[written] = (u & 0xFF) as u8;
            written += 1;
        }
        written
    }
}

/// Builds a 512-byte Protective MBR sector at LBA 0.
pub fn create_protective_mbr(disk_sectors: u64, out: &mut [u8; SECTOR_SIZE]) {
    out.fill(0);

    // Bootstrap code (bytes 0..445 left zeroed)
    // Partition 1 Record at offset 446 (0x1BE)
    let p = &mut out[446..462];
    p[0] = 0x00; // Bootable flag
    p[1] = 0x00; // Starting CHS head
    p[2] = 0x02; // Starting CHS sector (2)
    p[3] = 0x00; // Starting CHS cylinder
    p[4] = 0xEE; // OS type: GPT Protective MBR
    p[5] = 0xFF; // Ending CHS head
    p[6] = 0xFF; // Ending CHS sector
    p[7] = 0xFF; // Ending CHS cylinder

    // Starting LBA = 1
    let start_lba = 1u32.to_le_bytes();
    p[8..12].copy_from_slice(&start_lba);

    // Size in LBA (clamped to u32::MAX if disk exceeds 2 TiB)
    let size_sectors = if disk_sectors > 1 {
        (disk_sectors - 1).min(0xFFFF_FFFF) as u32
    } else {
        1
    };
    p[12..16].copy_from_slice(&size_sectors.to_le_bytes());

    // MBR Signature 0x55, 0xAA at offset 510..512
    out[510] = 0x55;
    out[511] = 0xAA;
}

/// Creates Primary and Backup GPT Headers along with a serialized Partition Entries array.
pub fn build_gpt_structures(
    disk_guid: [u8; 16],
    entries: &[PartitionEntry],
    disk_sectors: u64,
) -> Result<(GptHeader, GptHeader, Vec<u8>), GptError> {
    if disk_sectors < 68 {
        return Err(GptError::OutOfBounds);
    }

    let mut partition_array_bytes = Vec::with_capacity((DEFAULT_NUM_PARTITIONS * PARTITION_ENTRY_SIZE) as usize);
    for i in 0..DEFAULT_NUM_PARTITIONS as usize {
        if i < entries.len() {
            let entry_bytes: [u8; PARTITION_ENTRY_SIZE as usize] = unsafe {
                core::mem::transmute_copy(&entries[i])
            };
            partition_array_bytes.extend_from_slice(&entry_bytes);
        } else {
            let empty = [0u8; PARTITION_ENTRY_SIZE as usize];
            partition_array_bytes.extend_from_slice(&empty);
        }
    }

    let array_crc = crc32(&partition_array_bytes);
    let last_lba = disk_sectors - 1;
    let first_usable = 34u64;
    let last_usable = if last_lba >= 34 { last_lba - 34 } else { first_usable };

    // 1. Primary GPT Header (LBA 1)
    let mut primary = GptHeader {
        signature: *GPT_SIGNATURE,
        revision: GPT_REVISION_1_0,
        header_size: GPT_HEADER_SIZE,
        header_crc32: 0,
        reserved: 0,
        my_lba: 1,
        alternate_lba: last_lba,
        first_usable_lba: first_usable,
        last_usable_lba: last_usable,
        disk_guid,
        partition_entry_lba: 2,
        num_partition_entries: DEFAULT_NUM_PARTITIONS,
        size_partition_entry: PARTITION_ENTRY_SIZE,
        partition_entry_array_crc32: array_crc,
    };
    primary.header_crc32 = compute_header_crc(&primary);

    // 2. Backup GPT Header (LBA last_lba)
    let backup_entry_lba = if last_lba >= 33 { last_lba - 32 } else { 2 };
    let mut backup = GptHeader {
        signature: *GPT_SIGNATURE,
        revision: GPT_REVISION_1_0,
        header_size: GPT_HEADER_SIZE,
        header_crc32: 0,
        reserved: 0,
        my_lba: last_lba,
        alternate_lba: 1,
        first_usable_lba: first_usable,
        last_usable_lba: last_usable,
        disk_guid,
        partition_entry_lba: backup_entry_lba,
        num_partition_entries: DEFAULT_NUM_PARTITIONS,
        size_partition_entry: PARTITION_ENTRY_SIZE,
        partition_entry_array_crc32: array_crc,
    };
    backup.header_crc32 = compute_header_crc(&backup);

    Ok((primary, backup, partition_array_bytes))
}

/// Computes the CRC32 of a GPT Header with the header_crc32 field zeroed.
pub fn compute_header_crc(hdr: &GptHeader) -> u32 {
    let mut copy = *hdr;
    copy.header_crc32 = 0;
    let bytes: [u8; size_of::<GptHeader>()] = unsafe {
        core::mem::transmute_copy(&copy)
    };
    crc32(&bytes[..hdr.header_size as usize])
}

/// Parses and verifies a GPT Header from a 512-byte raw sector.
pub fn parse_gpt_header(sector: &[u8; SECTOR_SIZE]) -> Result<GptHeader, GptError> {
    if &sector[0..8] != GPT_SIGNATURE {
        return Err(GptError::InvalidSignature);
    }

    let hdr: GptHeader = unsafe {
        core::ptr::read_unaligned(sector.as_ptr() as *const GptHeader)
    };

    if hdr.revision != GPT_REVISION_1_0 {
        return Err(GptError::InvalidRevision);
    }
    if hdr.header_size != GPT_HEADER_SIZE {
        return Err(GptError::InvalidHeaderSize);
    }

    let calculated_crc = compute_header_crc(&hdr);
    if calculated_crc != hdr.header_crc32 {
        return Err(GptError::HeaderCrcMismatch);
    }

    Ok(hdr)
}

/// Parses a partition entry from a 128-byte raw buffer.
pub fn parse_partition_entry(entry_bytes: &[u8; 128]) -> Option<PartitionEntry> {
    let entry: PartitionEntry = unsafe {
        core::ptr::read_unaligned(entry_bytes.as_ptr() as *const PartitionEntry)
    };
    if entry.is_used() {
        Some(entry)
    } else {
        None
    }
}
