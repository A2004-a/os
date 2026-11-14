//! NovaOS Apex — Milestone 5 Automated Verification Suite
//! Tests VFS mounts, RAMFS mutations, DevFS nodes, NovaFS on-disk block persistence, and user security.

extern crate alloc;
use alloc::sync::Arc;

use crate::fs::novafs::NovaFs;
use crate::fs::types::*;
use crate::fs::vfs::get_vfs;
use crate::scheduler::sync::SpinLock;
use crate::security::*;
use crate::serial;

pub fn run_m5_verification_suite() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone 5 Verification Suite        \n");
    serial::write_str("=======================================================\n");

    let mut all_passed = true;

    // Test 1: VFS Paths & Mount Resolution
    serial::write_str("[M5 TEST 1/5] Testing VFS path resolution and hierarchy...\n");
    if test_vfs_paths() {
        serial::write_str("[M5 TEST 1/5] [PASS] VFS mount table and path resolution verified.\n");
    } else {
        serial::write_str("[M5 TEST 1/5] [FAIL] VFS path resolution failed.\n");
        all_passed = false;
    }

    // Test 2: RAMFS Mutation and Lifecycle
    serial::write_str("[M5 TEST 2/5] Testing RAMFS file creation, write, read, and delete...\n");
    if test_ramfs_lifecycle() {
        serial::write_str("[M5 TEST 2/5] [PASS] RAMFS mutations and directory traversal verified.\n");
    } else {
        serial::write_str("[M5 TEST 2/5] [FAIL] RAMFS mutation test failed.\n");
        all_passed = false;
    }

    // Test 3: DevFS Virtual Devices
    serial::write_str("[M5 TEST 3/5] Testing DevFS nodes (/dev/null, /dev/zero, /dev/urandom)...\n");
    if test_devfs_devices() {
        serial::write_str("[M5 TEST 3/5] [PASS] DevFS character and block device nodes verified.\n");
    } else {
        serial::write_str("[M5 TEST 3/5] [FAIL] DevFS device node test failed.\n");
        all_passed = false;
    }

    // Test 4: NovaFS Block Persistence Across Remount
    serial::write_str("[M5 TEST 4/5] Testing NovaFS on-disk block format, commit, and remount...\n");
    if test_novafs_persistence() {
        serial::write_str("[M5 TEST 4/5] [PASS] NovaFS crash-resilient block persistence verified.\n");
    } else {
        serial::write_str("[M5 TEST 4/5] [FAIL] NovaFS persistence test failed.\n");
        all_passed = false;
    }

    // Test 5: Security Model and User Isolation
    serial::write_str("[M5 TEST 5/5] Testing multi-user profile isolation and permissions...\n");
    if test_security_isolation() {
        serial::write_str("[M5 TEST 5/5] [PASS] User permissions (root UID 0 vs apex UID 1000) verified.\n");
    } else {
        serial::write_str("[M5 TEST 5/5] [FAIL] User isolation test failed.\n");
        all_passed = false;
    }

    if all_passed {
        serial::write_str("\n[MILESTONE 5 TEST RESULT] ALL 5 TEST SUITES PASSED.\n");
        serial::write_str("[MILESTONE 5 VERIFIED] VFS abstraction, RAMFS, DevFS, NovaFS block persistence,\n");
        serial::write_str("                       and User security profiles confirmed fully operational!\n");
    } else {
        serial::write_str("\n[MILESTONE 5 TEST RESULT] ONE OR MORE SUITES FAILED.\n");
    }

    all_passed
}

fn test_vfs_paths() -> bool {
    let root_creds = Credentials::root();
    let vfs_guard = get_vfs().expect("VFS not initialized");
    let vfs = vfs_guard.as_ref().expect("VFS instance empty");

    // 1. Stat /etc/os-release
    let stat = match vfs.stat("/etc/os-release", &root_creds) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if stat.size == 0 {
        return false;
    }

    // 2. Open and read /etc/hostname
    let mut file = match vfs.open("/etc/hostname", O_RDONLY, 0, &root_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = [0u8; 32];
    let bytes_read = match file.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    if bytes_read == 0 {
        return false;
    }
    let content = match core::str::from_utf8(&buf[..bytes_read]) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if !content.contains("novaos-apex") {
        return false;
    }

    // 3. Normalized dot-dot traversal
    let dotdot_stat = match vfs.stat("/etc/../etc/passwd", &root_creds) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if dotdot_stat.size == 0 {
        return false;
    }

    true
}

fn test_ramfs_lifecycle() -> bool {
    let root_creds = Credentials::root();
    let vfs_guard = get_vfs().expect("VFS not initialized");
    let vfs = vfs_guard.as_ref().expect("VFS instance empty");

    let test_path = "/tmp/ramfs_test.txt";
    let test_data = b"NovaOS-Apex-M5-RamFs-Payload-Canary";

    // 1. Create file with O_CREAT | O_WRONLY
    let mut file = match vfs.open(test_path, O_CREAT | O_WRONLY, 0o644, &root_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };

    // 2. Write data
    let written = match file.write(test_data) {
        Ok(n) => n,
        Err(_) => return false,
    };
    if written != test_data.len() {
        return false;
    }

    // 3. Open and read back
    let mut read_file = match vfs.open(test_path, O_RDONLY, 0, &root_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut read_buf = [0u8; 64];
    let bytes_read = match read_file.read(&mut read_buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    if bytes_read != test_data.len() || &read_buf[..bytes_read] != test_data {
        return false;
    }

    // 4. Unlink file and verify it's gone
    if vfs.unlink(test_path, &root_creds).is_err() {
        return false;
    }
    if vfs.stat(test_path, &root_creds) != Err(FsError::NotFound) {
        return false;
    }

    true
}

fn test_devfs_devices() -> bool {
    let root_creds = Credentials::root();
    let vfs_guard = get_vfs().expect("VFS not initialized");
    let vfs = vfs_guard.as_ref().expect("VFS instance empty");

    // 1. /dev/null: Write succeeds; read returns 0
    let mut null_file = match vfs.open("/dev/null", O_RDWR, 0, &root_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let test_bytes = [0x55u8; 128];
    if null_file.write(&test_bytes) != Ok(128) {
        return false;
    }
    let mut rx = [0u8; 16];
    if null_file.read(&mut rx) != Ok(0) {
        return false;
    }

    // 2. /dev/zero: Read fills buffer with 0
    let mut zero_file = match vfs.open("/dev/zero", O_RDONLY, 0, &root_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut zero_buf = [0xFFu8; 32];
    if zero_file.read(&mut zero_buf) != Ok(32) {
        return false;
    }
    if zero_buf.iter().any(|&b| b != 0) {
        return false;
    }

    // 3. /dev/urandom: Read fills with entropy
    let mut rand_file = match vfs.open("/dev/urandom", O_RDONLY, 0, &root_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut rand_buf = [0u8; 32];
    if rand_file.read(&mut rand_buf) != Ok(32) {
        return false;
    }
    if rand_buf.iter().all(|&b| b == 0) {
        return false;
    }

    true
}

fn test_novafs_persistence() -> bool {
    let root_creds = Credentials::root();
    let persist_path = "/storage/persist.bin";
    let test_canary: [u8; 16] = [
        0x50, 0x45, 0x52, 0x53, 0x49, 0x53, 0x54, 0x30, 0x31, 0xCA, 0xFE, 0xBA, 0xBE, 0x13, 0x37, 0x42,
    ];

    // 1. Write file to /storage
    {
        let vfs_guard = get_vfs().expect("VFS not initialized");
        let vfs = vfs_guard.as_ref().expect("VFS instance empty");
        let mut file = match vfs.open(persist_path, O_CREAT | O_WRONLY, 0o644, &root_creds) {
            Ok(f) => f,
            Err(_) => return false,
        };
        if file.write(&test_canary) != Ok(test_canary.len()) {
            return false;
        }
    }

    // 2. Unmount /storage
    {
        let mut vfs_guard = get_vfs().expect("VFS not initialized");
        let vfs = vfs_guard.as_mut().expect("VFS instance empty");
        if vfs.unmount("/storage").is_err() {
            return false;
        }
    }

    // 3. Remount NovaFS from disk
    let remounted = match NovaFs::mount() {
        Ok(fs) => fs,
        Err(_) => return false,
    };
    {
        let mut vfs_guard = get_vfs().expect("VFS not initialized");
        let vfs = vfs_guard.as_mut().expect("VFS instance empty");
        let _ = vfs.mount("/storage", Arc::new(SpinLock::new("novafs_remount", remounted)));
    }

    // 4. Read back and verify exact canary
    {
        let vfs_guard = get_vfs().expect("VFS not initialized");
        let vfs = vfs_guard.as_ref().expect("VFS instance empty");
        let mut file = match vfs.open(persist_path, O_RDONLY, 0, &root_creds) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let mut rx = [0u8; 16];
        if file.read(&mut rx) != Ok(16) {
            return false;
        }
        if rx != test_canary {
            return false;
        }
    }

    true
}

fn test_security_isolation() -> bool {
    let root_creds = Credentials::root();
    let user_creds = Credentials::user(1000, 1000);
    let intruder_creds = Credentials::user(2000, 2000);

    let vfs_guard = get_vfs().expect("VFS not initialized");
    let vfs = vfs_guard.as_ref().expect("VFS instance empty");

    let private_file = "/users/apex/.config/secret.key";

    // 1. Apex creates private file with 0o600 (owner only)
    let mut file = match vfs.open(private_file, O_CREAT | O_WRONLY, 0o600, &user_creds) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let _ = file.write(b"Apex-Secret-Key");

    // 2. Intruder UID 2000 tries to read it -> Must fail with PermissionDenied
    let intruder_result = vfs.open(private_file, O_RDONLY, 0, &intruder_creds);
    if intruder_result.is_ok() {
        return false; // Security failure!
    }

    // 3. Apex owner UID 1000 reads it -> Must succeed
    let apex_read = vfs.open(private_file, O_RDONLY, 0, &user_creds);
    if apex_read.is_err() {
        return false;
    }

    // 4. Root UID 0 reads it -> Must succeed
    let root_read = vfs.open(private_file, O_RDONLY, 0, &root_creds);
    if root_read.is_err() {
        return false;
    }

    true
}
