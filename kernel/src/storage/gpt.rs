//! NovaOS Zero — GUID Partition Table (GPT) & Protective MBR Engine
//! Compliant with UEFI 2.10 Specification (Chapter 5: GUID Partition Table Format).

pub const MBR_SIGNATURE: u16 = 0xAA55;
pub const MBR_GPT_PARTITION_TYPE: u8 = 0xEE;

pub const GPT_SIGNATURE: u64 = 0x5452_4150_2049_4645; // "EFI PART" in little-endian
pub const GPT_REVISION: u32 = 0x0001_0000;           // Version 1.0
pub const GPT_HEADER_SIZE: u32 = 92;
pub const GPT_ENTRY_SIZE: u32 = 128;

// Standard UEFI & NovaOS Partition Type GUIDs
pub const GUID_UNUSED: [u8; 16] = [0; 16];

// EFI System Partition (ESP): C12A7328-F81F-11D2-BA4B-00A0C93EC93B
pub const GUID_ESP: [u8; 16] = [
    0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B,
];

// NovaOS Root Partition (NovaFS): A1B2C3D4-E5F6-4A5B-8C9D-0E1F2A3B4C5D
pub const GUID_NOVAOS_ROOT: [u8; 16] = [
    0xD4, 0xC3, 0xB2, 0xA1, 0xF6, 0xE5, 0x5B, 0x4A, 0x8C, 0x9D, 0x0E, 0x1F, 0x2A, 0x3B, 0x4C, 0x5D,
];

// NovaOS Recovery Partition: D4C3B2A1-F6E5-5B4A-9D8C-5D4C3B2A1F0E
pub const GUID_NOVAOS_RECOVERY: [u8; 16] = [
    0xA1, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6, 0x4A, 0x5B, 0x9D, 0x8C, 0x5D, 0x4C, 0x3B, 0x2A, 0x1F, 0x0E,
];

/// IEEE 802.3 standard CRC-32 calculation for GPT headers and partition entries
pub fn calculate_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MbrPartitionRecord {
    pub boot_indicator: u8,
    pub start_chs: [u8; 3],
    pub os_type: u8,
    pub end_chs: [u8; 3],
    pub starting_lba: u32,
    pub size_in_lba: u32,
}

impl MbrPartitionRecord {
    pub fn get_os_type(&self) -> u8 { self.os_type }
    pub fn get_starting_lba(&self) -> u32 { self.starting_lba }
    pub fn get_size_in_lba(&self) -> u32 { self.size_in_lba }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectiveMbr {
    pub boot_code: [u8; 446],
    pub partitions: [MbrPartitionRecord; 4],
    pub signature: u16,
}

impl ProtectiveMbr {
    pub fn new(total_sectors: u64) -> Self {
        let size = if total_sectors > 0xFFFF_FFFF {
            0xFFFF_FFFF
        } else {
            (total_sectors.saturating_sub(1)) as u32
        };

        let gpt_record = MbrPartitionRecord {
            boot_indicator: 0x00,
            start_chs: [0x00, 0x02, 0x00],
            os_type: MBR_GPT_PARTITION_TYPE,
            end_chs: [0xFF, 0xFF, 0xFF],
            starting_lba: 1,
            size_in_lba: size,
        };

        let empty_record = MbrPartitionRecord {
            boot_indicator: 0,
            start_chs: [0; 3],
            os_type: 0,
            end_chs: [0; 3],
            starting_lba: 0,
            size_in_lba: 0,
        };

        ProtectiveMbr {
            boot_code: [0; 446],
            partitions: [gpt_record, empty_record, empty_record, empty_record],
            signature: MBR_SIGNATURE,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.signature == MBR_SIGNATURE && self.partitions[0].os_type == MBR_GPT_PARTITION_TYPE
    }

    pub fn get_signature(&self) -> u16 { self.signature }
    pub fn get_partition(&self, idx: usize) -> MbrPartitionRecord { self.partitions[idx] }

    pub fn serialize(&self, buf: &mut [u8; 512]) {
        unsafe {
            core::ptr::write_unaligned(buf.as_mut_ptr() as *mut ProtectiveMbr, *self);
        }
    }

    pub fn deserialize(buf: &[u8; 512]) -> Option<Self> {
        let mbr = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const ProtectiveMbr) };
        if mbr.is_valid() {
            Some(mbr)
        } else {
            None
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GptHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub header_crc32: u32,
    pub reserved: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entries_lba: u64,
    pub num_partition_entries: u32,
    pub partition_entry_size: u32,
    pub partition_entries_crc32: u32,
}

impl GptHeader {
    pub fn new(
        current_lba: u64,
        backup_lba: u64,
        total_sectors: u64,
        disk_guid: [u8; 16],
        entries_lba: u64,
        num_entries: u32,
        entries_crc32: u32,
    ) -> Self {
        let mut header = GptHeader {
            signature: GPT_SIGNATURE,
            revision: GPT_REVISION,
            header_size: GPT_HEADER_SIZE,
            header_crc32: 0,
            reserved: 0,
            current_lba,
            backup_lba,
            first_usable_lba: 34,
            last_usable_lba: total_sectors.saturating_sub(34),
            disk_guid,
            partition_entries_lba: entries_lba,
            num_partition_entries: num_entries,
            partition_entry_size: GPT_ENTRY_SIZE,
            partition_entries_crc32: entries_crc32,
        };

        // Compute header CRC32 with crc field zeroed
        let bytes = unsafe {
            core::slice::from_raw_parts(
                &header as *const GptHeader as *const u8,
                GPT_HEADER_SIZE as usize,
            )
        };
        header.header_crc32 = calculate_crc32(bytes);
        header
    }

    pub fn get_signature(&self) -> u64 { self.signature }
    pub fn get_revision(&self) -> u32 { self.revision }
    pub fn get_header_size(&self) -> u32 { self.header_size }
    pub fn get_header_crc32(&self) -> u32 { self.header_crc32 }
    pub fn get_current_lba(&self) -> u64 { self.current_lba }
    pub fn get_backup_lba(&self) -> u64 { self.backup_lba }
    pub fn get_first_usable_lba(&self) -> u64 { self.first_usable_lba }
    pub fn get_last_usable_lba(&self) -> u64 { self.last_usable_lba }
    pub fn get_num_entries(&self) -> u32 { self.num_partition_entries }
    pub fn get_entry_size(&self) -> u32 { self.partition_entry_size }
    pub fn get_entries_crc32(&self) -> u32 { self.partition_entries_crc32 }

    pub fn is_valid(&self) -> bool {
        if self.get_signature() != GPT_SIGNATURE || self.get_header_size() != GPT_HEADER_SIZE {
            return false;
        }

        let mut copy = *self;
        copy.header_crc32 = 0;
        let bytes = unsafe {
            core::slice::from_raw_parts(
                &copy as *const GptHeader as *const u8,
                GPT_HEADER_SIZE as usize,
            )
        };
        calculate_crc32(bytes) == self.get_header_crc32()
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GptPartitionEntry {
    pub type_guid: [u8; 16],
    pub unique_guid: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub attributes: u64,
    pub name: [u16; 36],
}

impl GptPartitionEntry {
    pub fn empty() -> Self {
        GptPartitionEntry {
            type_guid: GUID_UNUSED,
            unique_guid: GUID_UNUSED,
            starting_lba: 0,
            ending_lba: 0,
            attributes: 0,
            name: [0; 36],
        }
    }

    pub fn new(
        type_guid: [u8; 16],
        unique_guid: [u8; 16],
        starting_lba: u64,
        ending_lba: u64,
        attributes: u64,
        name_str: &str,
    ) -> Self {
        let mut name = [0u16; 36];
        for (i, c) in name_str.chars().take(36).enumerate() {
            name[i] = c as u16;
        }

        GptPartitionEntry {
            type_guid,
            unique_guid,
            starting_lba,
            ending_lba,
            attributes,
            name,
        }
    }

    pub fn is_used(&self) -> bool {
        self.type_guid != GUID_UNUSED
    }

    pub fn get_starting_lba(&self) -> u64 { self.starting_lba }
    pub fn get_ending_lba(&self) -> u64 { self.ending_lba }
    pub fn get_sector_count(&self) -> u64 {
        if self.is_used() && self.ending_lba >= self.starting_lba {
            self.ending_lba - self.starting_lba + 1
        } else {
            0
        }
    }

    pub fn get_name_ascii(&self, buf: &mut [u8]) -> usize {
        let name_copy = self.name;
        let mut len = 0;
        for &code in name_copy.iter() {
            if code == 0 || len >= buf.len() {
                break;
            }
            buf[len] = code as u8;
            len += 1;
        }
        len
    }
}
