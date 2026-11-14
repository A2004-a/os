//! NovaOS Zero — Disk Partitioner, System Installer & Recovery Snapshot Engine
//! Provisions standard GPT partitions, formats NovaFS root volumes, and manages recovery states.

use super::BlockDevice;
use super::gpt::{
    calculate_crc32, GptHeader, GptPartitionEntry, ProtectiveMbr, GUID_ESP, GUID_NOVAOS_RECOVERY,
    GUID_NOVAOS_ROOT,
};

#[derive(Debug, Clone, Copy)]
pub struct PartitionSummary {
    pub esp_start_lba: u64,
    pub esp_sectors: u64,
    pub root_start_lba: u64,
    pub root_sectors: u64,
    pub recovery_start_lba: u64,
    pub recovery_sectors: u64,
}

pub struct DiskPartitioner;

impl DiskPartitioner {
    /// Partition a block device into standard UEFI layout:
    /// - LBA 0: Protective MBR
    /// - LBA 1: Primary GPT Header
    /// - LBA 2: Partition Entry Array
    /// - LBA 34..64: EFI System Partition (ESP)
    /// - LBA 65..(N - 50): NovaOS Root Partition (NovaFS)
    /// - LBA (N - 49)..(N - 34): NovaOS Recovery Partition
    /// - LBA (N - 2): Backup Partition Array
    /// - LBA (N - 1): Backup GPT Header
    pub fn create_default_gpt_layout<B: BlockDevice>(dev: &mut B) -> Result<PartitionSummary, &'static str> {
        let total_sectors = dev.total_sectors();
        if total_sectors < 128 {
            return Err("Disk too small for GPT partitioning (minimum 128 sectors required)");
        }

        // 1. Write Protective MBR on LBA 0
        let mbr = ProtectiveMbr::new(total_sectors);
        let mut mbr_sector = [0u8; 512];
        mbr.serialize(&mut mbr_sector);
        dev.write_sector(0, &mbr_sector)?;

        // 2. Prepare 4 Partition Entries (128 bytes each = 512 bytes sector)
        let first_usable = 34u64;
        let last_usable = total_sectors.saturating_sub(34);

        let esp_start = first_usable;
        let esp_end = esp_start + 30; // 31 sectors

        let recovery_end = last_usable;
        let recovery_start = recovery_end.saturating_sub(15); // 16 sectors

        let root_start = esp_end + 1;
        let root_end = recovery_start.saturating_sub(1);

        if root_start >= root_end {
            return Err("Partition geometry bounds error");
        }

        let p1 = GptPartitionEntry::new(
            GUID_ESP,
            [1; 16],
            esp_start,
            esp_end,
            0,
            "EFI System Partition",
        );
        let p2 = GptPartitionEntry::new(
            GUID_NOVAOS_ROOT,
            [2; 16],
            root_start,
            root_end,
            0,
            "NovaOS Root",
        );
        let p3 = GptPartitionEntry::new(
            GUID_NOVAOS_RECOVERY,
            [3; 16],
            recovery_start,
            recovery_end,
            0,
            "NovaOS Recovery",
        );
        let p4 = GptPartitionEntry::empty();

        let mut entries_sector = [0u8; 512];
        unsafe {
            let ptr = entries_sector.as_mut_ptr() as *mut GptPartitionEntry;
            core::ptr::write_unaligned(ptr.add(0), p1);
            core::ptr::write_unaligned(ptr.add(1), p2);
            core::ptr::write_unaligned(ptr.add(2), p3);
            core::ptr::write_unaligned(ptr.add(3), p4);
        }

        let entries_crc32 = calculate_crc32(&entries_sector);

        // 3. Write Partition Table on LBA 2
        dev.write_sector(2, &entries_sector)?;

        // 4. Write Primary GPT Header on LBA 1
        let primary_hdr = GptHeader::new(
            1,
            total_sectors - 1,
            total_sectors,
            [0xAA; 16],
            2,
            4,
            entries_crc32,
        );
        let mut hdr_sector = [0u8; 512];
        unsafe {
            core::ptr::write_unaligned(hdr_sector.as_mut_ptr() as *mut GptHeader, primary_hdr);
        }
        dev.write_sector(1, &hdr_sector)?;

        // 5. Write Backup Partition Table on LBA (total_sectors - 2)
        dev.write_sector(total_sectors - 2, &entries_sector)?;

        // 6. Write Backup GPT Header on LBA (total_sectors - 1)
        let backup_hdr = GptHeader::new(
            total_sectors - 1,
            1,
            total_sectors,
            [0xAA; 16],
            total_sectors - 2,
            4,
            entries_crc32,
        );
        let mut backup_hdr_sector = [0u8; 512];
        unsafe {
            core::ptr::write_unaligned(backup_hdr_sector.as_mut_ptr() as *mut GptHeader, backup_hdr);
        }
        dev.write_sector(total_sectors - 1, &backup_hdr_sector)?;

        Ok(PartitionSummary {
            esp_start_lba: esp_start,
            esp_sectors: esp_end - esp_start + 1,
            root_start_lba: root_start,
            root_sectors: root_end - root_start + 1,
            recovery_start_lba: recovery_start,
            recovery_sectors: recovery_end - recovery_start + 1,
        })
    }

    /// Read and validate GPT partition table from block device
    pub fn read_gpt_partitions<B: BlockDevice>(dev: &mut B) -> Result<[Option<GptPartitionEntry>; 4], &'static str> {
        // 1. Verify MBR
        let mut mbr_sector = [0u8; 512];
        dev.read_sector(0, &mut mbr_sector)?;
        let mbr = ProtectiveMbr::deserialize(&mbr_sector).ok_or("Invalid or missing Protective MBR")?;
        if !mbr.is_valid() {
            return Err("Protective MBR invalid");
        }

        // 2. Read Primary GPT Header
        let mut hdr_sector = [0u8; 512];
        dev.read_sector(1, &mut hdr_sector)?;
        let hdr = unsafe { core::ptr::read_unaligned(hdr_sector.as_ptr() as *const GptHeader) };
        if !hdr.is_valid() {
            return Err("Primary GPT header signature or CRC32 corrupted");
        }

        // 3. Read Partition Table
        let mut entries_sector = [0u8; 512];
        dev.read_sector(hdr.get_first_usable_lba().min(2), &mut entries_sector)?;

        let computed_crc = calculate_crc32(&entries_sector);
        if computed_crc != hdr.get_entries_crc32() {
            return Err("Partition table array CRC32 corrupted");
        }

        let mut result: [Option<GptPartitionEntry>; 4] = [None, None, None, None];
        unsafe {
            let ptr = entries_sector.as_ptr() as *const GptPartitionEntry;
            for i in 0..4 {
                let entry = core::ptr::read_unaligned(ptr.add(i));
                if entry.is_used() {
                    result[i] = Some(entry);
                }
            }
        }

        Ok(result)
    }

    /// Format NovaOS root partition with NovaFS superblock
    pub fn format_novaos_root<B: BlockDevice>(
        dev: &mut B,
        root_start_lba: u64,
        _root_sectors: u64,
    ) -> Result<(), &'static str> {
        let mut superblock_sector = [0u8; 512];
        // NovaFS Superblock Magic: 0x4E4F5641 ("NOVA")
        superblock_sector[0..4].copy_from_slice(&0x4E4F5641u32.to_le_bytes());
        // Version: 1
        superblock_sector[4..6].copy_from_slice(&1u16.to_le_bytes());
        // Block size: 512
        superblock_sector[6..8].copy_from_slice(&512u16.to_le_bytes());

        // Write Superblock at start of root partition
        dev.write_sector(root_start_lba, &superblock_sector)?;
        Ok(())
    }

    /// Capture a recovery snapshot by copying critical sectors
    pub fn create_recovery_snapshot<B: BlockDevice>(
        dev: &mut B,
        src_lba: u64,
        dst_lba: u64,
        sector_count: u64,
    ) -> Result<(), &'static str> {
        let mut temp_buf = [0u8; 512];
        for i in 0..sector_count {
            dev.read_sector(src_lba + i, &mut temp_buf)?;
            dev.write_sector(dst_lba + i, &temp_buf)?;
        }
        Ok(())
    }

    /// Restore system from recovery snapshot
    pub fn restore_recovery_snapshot<B: BlockDevice>(
        dev: &mut B,
        src_lba: u64,
        dst_lba: u64,
        sector_count: u64,
    ) -> Result<(), &'static str> {
        Self::create_recovery_snapshot(dev, src_lba, dst_lba, sector_count)
    }
}
