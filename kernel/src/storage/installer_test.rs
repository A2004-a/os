//! NovaOS Zero — Milestone M10 Automated Verification Suite
//! Validates IEEE 802.3 CRC32, Protective MBR formatting, GPT Header integrity,
//! disk layout partitioning, NovaFS volume formatting, and recovery snapshot rollback.

use crate::serial_println;
use crate::storage::gpt::{
    calculate_crc32, GptHeader, ProtectiveMbr, GUID_ESP, GUID_NOVAOS_RECOVERY,
    GUID_NOVAOS_ROOT, MBR_GPT_PARTITION_TYPE, MBR_SIGNATURE,
};
use crate::storage::partitioner::DiskPartitioner;
use crate::storage::{BlockDevice, RamBlockDevice};

pub fn run_installer_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M10 Automated Verification");
    serial_println!("=======================================================");

    test_crc32_algorithm();
    test_protective_mbr_generation_and_validation();
    test_gpt_header_validation_and_crc();
    test_end_to_end_disk_partitioning();
    test_novafs_root_formatting_and_snapshot_recovery();

    serial_println!("[M10 SUCCESS] All Milestone M10 Verification Tests Passed Cleanly!\n");
}

fn test_crc32_algorithm() {
    serial_println!("[TEST 1/5] IEEE 802.3 Standard CRC-32 Algorithm...");

    // Standard IEEE 802.3 test vectors
    assert_eq!(calculate_crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(calculate_crc32(b""), 0x0000_0000);
    assert_eq!(calculate_crc32(b"NovaOS"), 0x075A_C2B6);

    serial_println!("  -> CRC-32 standard test vector polynomial computation verified.");
}

fn test_protective_mbr_generation_and_validation() {
    serial_println!("[TEST 2/5] Protective MBR Structure & LBA 0 Geometry...");

    let mbr = ProtectiveMbr::new(512);
    assert!(mbr.is_valid());
    assert_eq!(mbr.get_signature(), MBR_SIGNATURE);
    assert_eq!(mbr.get_partition(0).get_os_type(), MBR_GPT_PARTITION_TYPE);
    assert_eq!(mbr.get_partition(0).get_starting_lba(), 1);
    assert_eq!(mbr.get_partition(0).get_size_in_lba(), 511);

    let mut sector = [0u8; 512];
    mbr.serialize(&mut sector);

    // Verify deserialization
    let parsed = ProtectiveMbr::deserialize(&sector);
    assert!(parsed.is_some());
    assert_eq!(parsed.unwrap(), mbr);

    // Corrupted signature rejection
    let mut bad_sector = sector;
    bad_sector[510] = 0x00;
    assert!(ProtectiveMbr::deserialize(&bad_sector).is_none());

    serial_println!("  -> Protective MBR generation, serialization, and signature validation verified.");
}

fn test_gpt_header_validation_and_crc() {
    serial_println!("[TEST 3/5] GPT Header Generation, Offsets & Header CRC32 Validation...");

    let hdr = GptHeader::new(1, 511, 512, [0x11; 16], 2, 4, 0x1234_5678);
    assert!(hdr.is_valid());
    assert_eq!(hdr.get_current_lba(), 1);
    assert_eq!(hdr.get_backup_lba(), 511);
    assert_eq!(hdr.get_first_usable_lba(), 34);
    assert_eq!(hdr.get_last_usable_lba(), 478);
    assert_eq!(hdr.get_num_entries(), 4);

    // Corrupting header byte must invalidate CRC32
    let mut bad_hdr = hdr;
    bad_hdr.reserved = 0xDEAD_BEEF;
    assert!(!bad_hdr.is_valid());

    serial_println!("  -> GPT Header magic, LBA ranges, and dynamic CRC32 self-check verified.");
}

fn test_end_to_end_disk_partitioning() {
    serial_println!("[TEST 4/5] End-to-End Disk Partitioning on RamBlockDevice...");

    let mut ramdisk = RamBlockDevice::new();
    let summary = DiskPartitioner::create_default_gpt_layout(&mut ramdisk)
        .expect("Disk partitioning failed");

    assert_eq!(summary.esp_start_lba, 34);
    assert_eq!(summary.esp_sectors, 31);
    assert_eq!(summary.root_start_lba, 65);
    assert_eq!(summary.recovery_start_lba, 463);

    // Read back and validate from block device
    let partitions = DiskPartitioner::read_gpt_partitions(&mut ramdisk)
        .expect("Reading GPT partitions failed");

    // Partition 1: ESP
    assert!(partitions[0].is_some());
    let esp = partitions[0].unwrap();
    assert_eq!(esp.type_guid, GUID_ESP);
    assert_eq!(esp.get_starting_lba(), 34);
    let mut esp_name = [0u8; 32];
    let len = esp.get_name_ascii(&mut esp_name);
    assert_eq!(&esp_name[..len], b"EFI System Partition");

    // Partition 2: NovaOS Root
    assert!(partitions[1].is_some());
    let root = partitions[1].unwrap();
    assert_eq!(root.type_guid, GUID_NOVAOS_ROOT);
    assert_eq!(root.get_starting_lba(), 65);
    let mut root_name = [0u8; 32];
    let len = root.get_name_ascii(&mut root_name);
    assert_eq!(&root_name[..len], b"NovaOS Root");

    // Partition 3: NovaOS Recovery
    assert!(partitions[2].is_some());
    let recovery = partitions[2].unwrap();
    assert_eq!(recovery.type_guid, GUID_NOVAOS_RECOVERY);
    assert_eq!(recovery.get_starting_lba(), 463);
    let mut rec_name = [0u8; 32];
    let len = recovery.get_name_ascii(&mut rec_name);
    assert_eq!(&rec_name[..len], b"NovaOS Recovery");

    // Partition 4: Empty slot
    assert!(partitions[3].is_none());

    serial_println!("  -> Primary/Backup GPT layout, partition entries, and names verified.");
}

fn test_novafs_root_formatting_and_snapshot_recovery() {
    serial_println!("[TEST 5/5] NovaFS Root Formatting & Recovery Snapshot Rollback...");

    let mut ramdisk = RamBlockDevice::new();
    let summary = DiskPartitioner::create_default_gpt_layout(&mut ramdisk)
        .expect("Disk partitioning failed");

    // Format root partition with NovaFS superblock
    DiskPartitioner::format_novaos_root(&mut ramdisk, summary.root_start_lba, summary.root_sectors)
        .expect("NovaFS root format failed");

    // Read back root superblock
    let mut sb_sector = [0u8; 512];
    ramdisk.read_sector(summary.root_start_lba, &mut sb_sector).expect("Read root failed");
    let magic = u32::from_le_bytes([sb_sector[0], sb_sector[1], sb_sector[2], sb_sector[3]]);
    let version = u16::from_le_bytes([sb_sector[4], sb_sector[5]]);
    assert_eq!(magic, 0x4E4F_5641); // 'NOVA'
    assert_eq!(version, 1);

    // Create recovery snapshot (copy root to recovery)
    DiskPartitioner::create_recovery_snapshot(
        &mut ramdisk,
        summary.root_start_lba,
        summary.recovery_start_lba,
        1,
    )
    .expect("Snapshot creation failed");

    // Simulate sector corruption on Root
    let zero_sector = [0u8; 512];
    ramdisk.write_sector(summary.root_start_lba, &zero_sector).expect("Corrupt root failed");

    // Verify corruption
    ramdisk.read_sector(summary.root_start_lba, &mut sb_sector).expect("Read root failed");
    assert_eq!(sb_sector[0..4], [0, 0, 0, 0]);

    // Restore from Recovery Snapshot
    DiskPartitioner::restore_recovery_snapshot(
        &mut ramdisk,
        summary.recovery_start_lba,
        summary.root_start_lba,
        1,
    )
    .expect("Snapshot restore failed");

    // Verify restoration
    ramdisk.read_sector(summary.root_start_lba, &mut sb_sector).expect("Read restored root failed");
    let restored_magic = u32::from_le_bytes([sb_sector[0], sb_sector[1], sb_sector[2], sb_sector[3]]);
    assert_eq!(restored_magic, 0x4E4F_5641);

    serial_println!("  -> NovaFS root superblock formatting and snapshot recovery rollback verified.");
}
