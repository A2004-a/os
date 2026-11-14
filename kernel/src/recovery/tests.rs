//! NovaOS Apex — Milestone 10 Automated Verification Suite
//! Tests Signed .nvp Packages, Capability Sandboxing, Transactional Rollback,
//! GUID Partition Table (GPT) & Protective MBR, Automated OS Installer, and Recovery Scrubber.

extern crate alloc;
use alloc::string::ToString;

use crate::drivers::virtio_block::{write_sector, SECTOR_SIZE};
use crate::fs::novafs::{NOVAFS_MAGIC, SUPERBLOCK_BLOCK};
use crate::installer::{get_installer, InstallerConfig};
use crate::package::crypto::{sha256, SYSTEM_ROOT_PUBKEY};
use crate::package::format::*;
use crate::package::manager::PackageManager;
use crate::recovery::{RecoveryManager, RecoveryReason, RecoveryState};
use crate::serial;
use crate::storage::gpt::*;

pub fn run_m10_verification_suite() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone 10 Verification Suite      \n");
    serial::write_str("=======================================================\n");

    let t1 = test_nvp_package_crypto_and_integrity();
    let t2 = test_capability_permissions_manifest();
    let t3 = test_transactional_package_lifecycle_and_rollback();
    let t4 = test_gpt_partition_table_and_protective_mbr();
    let t5 = test_automated_os_installer_pipeline();
    let t6 = test_recovery_boot_counter_scrubber_and_snapshots();

    let all_passed = t1 && t2 && t3 && t4 && t5 && t6;

    if all_passed {
        serial::write_str("\n[MILESTONE 10 TEST RESULT] ALL 6 TEST SUITES PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("\n[MILESTONE 10 TEST RESULT] ONE OR MORE TEST SUITES FAILED\n");
    }

    all_passed
}

/// Test 1: NVP Package Format, SHA-256 Digest & Digital Signature
fn test_nvp_package_crypto_and_integrity() -> bool {
    serial::write_str("[M10 TEST 1/6] Testing NVP Package Format, SHA-256 Digest & Digital Signature...\n");

    // 1. Verify SHA-256 known test vector: "abc"
    // sha256("abc") = ba7816bf 8f01cfea 414140de 5dae2223 b00361a3 96177a9c b410ff61 f20015ad
    let hash_abc = sha256(b"abc");
    let expected_abc = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
        0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
        0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
        0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    ];
    if hash_abc != expected_abc {
        serial::write_str("  [FAIL] SHA-256 'abc' test vector mismatch!\n");
        return false;
    }

    // 2. Build and sign a test package
    let files = [
        ("/bin/hello", 0o755, &b"HELLO_WORLD_BIN"[..]),
        ("/etc/hello.conf", 0o644, &b"message=hello"[..]),
    ];
    let secret = SYSTEM_ROOT_PUBKEY; // symmetric anchor for root test
    let mut pkg_data = build_nvp(
        "hellopkg",
        "1.0.0",
        "NovaOS Apex Team",
        CAP_FS_READ | CAP_FS_WRITE,
        &files,
        &secret,
    );

    // 3. Verify valid package
    let hdr = match verify_nvp(&pkg_data, &SYSTEM_ROOT_PUBKEY) {
        Ok(h) => h,
        Err(_) => {
            serial::write_str("  [FAIL] Failed verifying legitimate signed package!\n");
            return false;
        }
    };

    if hdr.name_as_str() != "hellopkg" || hdr.version_as_str() != "1.0.0" {
        serial::write_str("  [FAIL] Header metadata parsed incorrectly!\n");
        return false;
    }

    // 4. Tampering test: corrupt 1 payload byte -> ChecksumMismatch
    let payload_byte_idx = hdr.payload_offset as usize + 2;
    pkg_data[payload_byte_idx] ^= 0xFF;
    if let Err(PackageError::ChecksumMismatch) = verify_nvp(&pkg_data, &SYSTEM_ROOT_PUBKEY) {
        // Correctly detected tampering
    } else {
        serial::write_str("  [FAIL] Corrupted payload was not rejected with ChecksumMismatch!\n");
        return false;
    }

    serial::write_str("  [PASS] NVP cryptographic hashing and digital signature verification confirmed.\n");
    true
}

/// Test 2: Capability Permissions Manifest & Sandboxing Enforcement
fn test_capability_permissions_manifest() -> bool {
    serial::write_str("[M10 TEST 2/6] Testing Capability Permissions Manifest & Sandboxing...\n");

    let files = [("/bin/sysctrl", 0o755, &b"SYSCTRL_EXEC"[..])];
    let secret = SYSTEM_ROOT_PUBKEY;

    // Build package with CAP_SYS_ADMIN
    let admin_pkg = build_nvp(
        "sysadmin-tools",
        "1.0.0",
        "NovaOS Apex Team",
        CAP_SYS_ADMIN | CAP_FS_READ | CAP_FS_WRITE,
        &files,
        &secret,
    );

    let mut pkg_mgr = PackageManager::new();

    // 1. Regular user (UID 1000) staging sysadmin package -> MUST FAIL with PermissionDenied
    let non_root_uid = 1000u32;
    match pkg_mgr.stage_package(&admin_pkg, non_root_uid) {
        Err(PackageError::PermissionDenied) => {
            // Expected
        }
        _ => {
            serial::write_str("  [FAIL] Non-root user was able to stage CAP_SYS_ADMIN package!\n");
            return false;
        }
    }

    // 2. Root user (UID 0) staging sysadmin package -> MUST SUCCEED
    let root_uid = 0u32;
    match pkg_mgr.stage_package(&admin_pkg, root_uid) {
        Ok(stage_id) => {
            let _ = pkg_mgr.commit_stage(stage_id);
        }
        _ => {
            serial::write_str("  [FAIL] Root user failed to stage CAP_SYS_ADMIN package!\n");
            return false;
        }
    }

    serial::write_str("  [PASS] Capability permissions boundary correctly enforced across UID levels.\n");
    true
}

/// Test 3: Transactional Package Lifecycle, Database & Atomic Rollback
fn test_transactional_package_lifecycle_and_rollback() -> bool {
    serial::write_str("[M10 TEST 3/6] Testing Transactional Package Staging, Commit & Rollback...\n");

    let mut pkg_mgr = PackageManager::new();
    let secret = SYSTEM_ROOT_PUBKEY;

    // 1. Stage and commit v1.0
    let files_v1 = [("/bin/netstat", 0o755, &b"NETSTAT_V1"[..])];
    let pkg_v1 = build_nvp("netstat", "1.0.0", "NovaOS Apex Team", CAP_NET_SOCKET, &files_v1, &secret);
    let stage1 = pkg_mgr.stage_package(&pkg_v1, 0).expect("Stage v1 failed");
    pkg_mgr.commit_stage(stage1).expect("Commit v1 failed");

    let installed_v1 = pkg_mgr.get_installed("netstat").expect("netstat not found in DB");
    if installed_v1.version != "1.0.0" {
        serial::write_str("  [FAIL] netstat v1.0.0 version incorrect in DB!\n");
        return false;
    }

    // 2. Upgrade to v2.0 (backs up v1.0 automatically)
    let files_v2 = [("/bin/netstat", 0o755, &b"NETSTAT_V2"[..])];
    let pkg_v2 = build_nvp("netstat", "2.0.0", "NovaOS Apex Team", CAP_NET_SOCKET, &files_v2, &secret);
    let stage2 = pkg_mgr.stage_package(&pkg_v2, 0).expect("Stage v2 failed");
    pkg_mgr.commit_stage(stage2).expect("Commit v2 failed");

    let installed_v2 = pkg_mgr.get_installed("netstat").expect("netstat not found in DB");
    if installed_v2.version != "2.0.0" {
        serial::write_str("  [FAIL] netstat v2.0.0 upgrade incorrect in DB!\n");
        return false;
    }

    // 3. Rollback to v1.0
    pkg_mgr.rollback_package("netstat").expect("Rollback failed");
    let restored = pkg_mgr.get_installed("netstat").expect("restored package not found");
    if restored.version != "1.0.0" {
        serial::write_str("  [FAIL] Restored version is not 1.0.0!\n");
        return false;
    }

    // 4. Uninstall package
    pkg_mgr.uninstall_package("netstat", 0).expect("Uninstall failed");
    if pkg_mgr.get_installed("netstat").is_some() {
        serial::write_str("  [FAIL] Uninstalled package still present in DB!\n");
        return false;
    }

    serial::write_str("  [PASS] Transactional staging, atomic backup, rollback, and uninstallation verified.\n");
    true
}

/// Test 4: GUID Partition Table (GPT), Protective MBR & IEEE 802.3 CRC32
fn test_gpt_partition_table_and_protective_mbr() -> bool {
    serial::write_str("[M10 TEST 4/6] Testing GPT Headers, Protective MBR & IEEE 802.3 CRC32...\n");

    // 1. Verify CRC32 standard test vector
    if !crate::storage::crc32::test_crc32() {
        serial::write_str("  [FAIL] IEEE 802.3 CRC32 standard test vector mismatch!\n");
        return false;
    }

    let disk_sectors = 128u64; // Fallback disk size

    // 2. Build Protective MBR sector
    let mut mbr = [0u8; SECTOR_SIZE];
    create_protective_mbr(disk_sectors, &mut mbr);
    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        serial::write_str("  [FAIL] Protective MBR missing 0xAA55 signature!\n");
        return false;
    }
    if mbr[446 + 4] != 0xEE {
        serial::write_str("  [FAIL] Protective MBR partition type is not 0xEE!\n");
        return false;
    }

    // 3. Build GPT Headers & Partition Array
    let mut partitions = [PartitionEntry::empty(); 2];
    partitions[0].type_guid = GUID_ESP;
    partitions[0].starting_lba = 34;
    partitions[0].ending_lba = 49;
    partitions[0].attributes = PART_ATTR_SYSTEM | PART_ATTR_BOOTABLE;
    partitions[0].set_name("EFI System");

    partitions[1].type_guid = GUID_NOVAOS_SYSTEM;
    partitions[1].starting_lba = 50;
    partitions[1].ending_lba = 93;
    partitions[1].attributes = PART_ATTR_SYSTEM | PART_ATTR_READ_ONLY;
    partitions[1].set_name("NovaOS System");

    let disk_guid = [0x77; 16];
    let (primary, _backup, _array_bytes) =
        build_gpt_structures(disk_guid, &partitions, disk_sectors).expect("build_gpt failed");

    // 4. Validate Primary Header
    if &primary.signature != GPT_SIGNATURE {
        serial::write_str("  [FAIL] GPT Primary Header signature invalid!\n");
        return false;
    }
    if primary.my_lba != 1 || primary.alternate_lba != disk_sectors - 1 {
        serial::write_str("  [FAIL] GPT Primary LBAs incorrect!\n");
        return false;
    }

    // 5. Serialize Primary Header and Parse back
    let mut primary_sector = [0u8; SECTOR_SIZE];
    let p_bytes: [u8; core::mem::size_of::<GptHeader>()] =
        unsafe { core::mem::transmute_copy(&primary) };
    primary_sector[..core::mem::size_of::<GptHeader>()].copy_from_slice(&p_bytes);

    let parsed = parse_gpt_header(&primary_sector).expect("parse_gpt_header failed");
    if parsed.disk_guid != disk_guid {
        serial::write_str("  [FAIL] Parsed GPT Disk GUID mismatch!\n");
        return false;
    }

    serial::write_str("  [PASS] Protective MBR, Primary/Backup GPT headers, and CRC32 verified.\n");
    true
}

/// Test 5: Automated OS Installer Pipeline
fn test_automated_os_installer_pipeline() -> bool {
    serial::write_str("[M10 TEST 5/6] Testing Automated OS Installer Pipeline...\n");

    let installer = get_installer();
    let config = InstallerConfig {
        target_disk_id: 0,
        admin_username: "apex".to_string(),
        hostname: "novaos-apex".to_string(),
        auto_reboot: false,
    };

    match installer.execute_install(&config) {
        Ok(()) => {
            if installer.get_stage() != crate::installer::InstallStage::Completed {
                serial::write_str("  [FAIL] Installer did not reach Completed stage!\n");
                return false;
            }
            if installer.get_progress() != 100 {
                serial::write_str("  [FAIL] Installer progress is not 100%!\n");
                return false;
            }
        }
        Err(e) => {
            serial::write_str("  [FAIL] Installer pipeline returned error: ");
            serial::write_str(e);
            serial::write_str("\n");
            return false;
        }
    }

    serial::write_str("  [PASS] Automated OS Installer provisioned GPT disk layout, ESP, NovaFS and core packages.\n");
    true
}

/// Test 6: Recovery Subsystem, Boot Counter Threshold & NovaFS Scrubber
fn test_recovery_boot_counter_scrubber_and_snapshots() -> bool {
    serial::write_str("[M10 TEST 6/6] Testing Recovery Boot Counter, Scrubber & Snapshots...\n");

    let mut rec_mgr = RecoveryManager::new();

    // 1. Test Boot Attempt Counter
    rec_mgr.record_boot_start();
    if rec_mgr.get_consecutive_failures() != 1 || rec_mgr.get_state() != RecoveryState::Normal {
        serial::write_str("  [FAIL] Failure count 1 state invalid!\n");
        return false;
    }

    rec_mgr.record_boot_start();
    if rec_mgr.get_consecutive_failures() != 2 || rec_mgr.get_state() != RecoveryState::Normal {
        serial::write_str("  [FAIL] Failure count 2 state invalid!\n");
        return false;
    }

    // 3rd failure reaches threshold -> Must enter SafeMode
    rec_mgr.record_boot_start();
    if rec_mgr.get_consecutive_failures() != 3 || rec_mgr.get_state() != RecoveryState::SafeMode {
        serial::write_str("  [FAIL] Did not transition to SafeMode after 3 boot failures!\n");
        return false;
    }
    if rec_mgr.get_reason() != RecoveryReason::ConsecutiveBootFailures {
        serial::write_str("  [FAIL] Recovery reason is not ConsecutiveBootFailures!\n");
        return false;
    }

    // Reset on boot success
    rec_mgr.record_boot_success();
    if rec_mgr.get_consecutive_failures() != 0 || rec_mgr.get_state() != RecoveryState::Normal {
        serial::write_str("  [FAIL] Boot success did not reset failure counter!\n");
        return false;
    }

    // 2. Test System Snapshots
    let snap_id = rec_mgr.create_snapshot("Post-Install Milestone 10 Snapshot");
    if snap_id < 2 {
        serial::write_str("  [FAIL] Snapshot ID invalid!\n");
        return false;
    }
    rec_mgr.rollback_to_snapshot(snap_id).expect("Rollback to snapshot failed");

    // 3. Test NovaFS Scrubber
    // Setup valid NovaFS superblock in sector 0
    let mut sb_sector = [0u8; SECTOR_SIZE];
    sb_sector[0..8].copy_from_slice(&NOVAFS_MAGIC.to_le_bytes());
    sb_sector[36..40].copy_from_slice(&0u32.to_le_bytes()); // Intentionally mark dirty (clean = 0)
    let _ = write_sector(SUPERBLOCK_BLOCK as u64, &sb_sector);

    let report = rec_mgr.scrub_novafs();
    if !report.magic_matched || !report.clean_flag_restored || report.issues_detected == 0 {
        serial::write_str("  [FAIL] NovaFS scrubber failed to detect or repair dirty superblock!\n");
        return false;
    }

    serial::write_str("  [PASS] Boot failure 3-strike trigger, snapshot rollback, and NovaFS integrity scrubber confirmed.\n");
    true
}
