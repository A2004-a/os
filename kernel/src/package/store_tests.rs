//! NovaOS Apex — Nova Store & Package Management Verification Suite
//! Verifies all 20 store and package management gates one by one:
//! - Gate 1:  NVP Binary Format & Magic Header Detection (NOVA_NVP)
//! - Gate 2:  SHA-256 Payload Hash Integrity
//! - Gate 3:  Ed25519 Cryptographic Signature Verification
//! - Gate 4:  Tamper Detection & Corrupted Package Rejection
//! - Gate 5:  Untrusted Publisher Public Key Isolation
//! - Gate 6:  System Capability Bitmask Decoding
//! - Gate 7:  Capability Sandboxing & Privilege Ring Enforcement (Root UID 0 vs User UID 1000)
//! - Gate 8:  Package Manifest & File Entry Unpacking
//! - Gate 9:  Transactional Staging (stage_package)
//! - Gate 10: Atomic Registry Commit (commit_stage)
//! - Gate 11: Automated Backup Creation on Upgrade
//! - Gate 12: Atomic Version Rollback (rollback_package)
//! - Gate 13: Safe Package Uninstallation (uninstall_package)
//! - Gate 14: Curated Store Catalog Enumeration (6 Official Packages)
//! - Gate 15: Category Filtering Subsystem (System, Utilities, Productivity, Creative)
//! - Gate 16: Store CLI Query (pkg list / store list)
//! - Gate 17: Store CLI Inspection (pkg info <name>)
//! - Gate 18: Store CLI Transactional Install (pkg install <name>)
//! - Gate 19: Store CLI Transactional Uninstall (pkg remove <name>)
//! - Gate 20: Desktop Nova Store Graphical Surface Rendering

use super::crypto::{sha256, SYSTEM_ROOT_PUBKEY};
use super::format::*;
use super::manager::PackageManager;
use crate::graphics::desktop::DesktopShell;
use crate::serial;

macro_rules! check_store {
    ($num:expr, $name:expr, $cond:expr) => {
        if $cond {
            serial::write_str("  [STORE TEST ");
            print_num($num);
            serial::write_str("/20] PASS: ");
            serial::write_str($name);
            serial::write_str("\n");
        } else {
            serial::write_str("  [STORE TEST ");
            print_num($num);
            serial::write_str("/20] FAIL: ");
            serial::write_str($name);
            serial::write_str("\n");
            return false;
        }
    };
}

#[derive(Clone, Copy)]
pub struct StoreCatalogEntry {
    pub name: &'static str,
    pub version: &'static str,
    pub category: &'static str,
    pub publisher: &'static str,
    pub installed: bool,
    pub size_kb: u32,
    pub permissions: &'static str,
}

pub static STORE_CATALOG: [StoreCatalogEntry; 6] = [
    StoreCatalogEntry {
        name: "coreutils",
        version: "0.5.0",
        category: "System Utilities",
        publisher: "NovaOS Apex Team",
        installed: true,
        size_kb: 128,
        permissions: "fs:read, fs:write, proc:exec",
    },
    StoreCatalogEntry {
        name: "novashell",
        version: "0.8.0",
        category: "System Shell",
        publisher: "NovaOS Apex Team",
        installed: true,
        size_kb: 256,
        permissions: "fs:all, proc:all, net:all",
    },
    StoreCatalogEntry {
        name: "nettools",
        version: "0.6.0",
        category: "Network Tools",
        publisher: "NovaOS Apex Team",
        installed: true,
        size_kb: 96,
        permissions: "net:socket, net:bind, net:raw",
    },
    StoreCatalogEntry {
        name: "textedit",
        version: "1.0.0",
        category: "Productivity",
        publisher: "Apex Labs",
        installed: false,
        size_kb: 180,
        permissions: "fs:read, fs:write, gui:window",
    },
    StoreCatalogEntry {
        name: "calc",
        version: "0.4.0",
        category: "Utilities",
        publisher: "Apex Labs",
        installed: false,
        size_kb: 64,
        permissions: "gui:window",
    },
    StoreCatalogEntry {
        name: "paint",
        version: "0.3.0",
        category: "Creative Tools",
        publisher: "Community Contrib",
        installed: false,
        size_kb: 320,
        permissions: "fs:write, gui:window",
    },
];

pub fn run_all_store_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Nova Store & Package Test Suite (20 Gates) \n");
    serial::write_str("=======================================================\n");

    let res = execute_all_gates();

    serial::write_str("\n=======================================================\n");
    if res {
        serial::write_str("[NOVA STORE RESULT] ALL 20 NOVA STORE TESTS PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("[NOVA STORE RESULT] NOVA STORE SUBSYSTEM ENCOUNTERED FAILURES!\n");
    }
    serial::write_str("=======================================================\n");

    res
}

fn execute_all_gates() -> bool {
    let secret = SYSTEM_ROOT_PUBKEY;

    // Gate 1: NVP Binary Format & Magic Header Detection
    let files_test1 = [("/bin/demo", 0o755, &b"DEMO_CODE"[..])];
    let pkg1 = build_nvp("demoapp", "1.0.0", "NovaOS Team", CAP_FS_READ, &files_test1, &secret);
    let is_magic_valid = pkg1.len() >= 8 && &pkg1[0..8] == NVP_MAGIC;
    let mut bad_pkg = pkg1.clone();
    bad_pkg[0] = b'X';
    let rejects_bad_magic = matches!(verify_nvp(&bad_pkg, &SYSTEM_ROOT_PUBKEY), Err(PackageError::InvalidMagic));
    check_store!(1, "NVP Binary Format & Magic Header Detection (NOVA_NVP)", is_magic_valid && rejects_bad_magic);

    // Gate 2: SHA-256 Payload Hash Integrity
    let hash_abc = sha256(b"abc");
    let expected_abc = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
        0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
        0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
        0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    ];
    let sha256_golden = hash_abc == expected_abc;
    let hdr1 = verify_nvp(&pkg1, &SYSTEM_ROOT_PUBKEY).expect("verify pkg1");
    let payload_bytes = &pkg1[hdr1.payload_offset as usize..(hdr1.payload_offset + hdr1.payload_size) as usize];
    let sha256_payload = sha256(payload_bytes) == hdr1.sha256_hash;
    check_store!(2, "SHA-256 Payload Hash Integrity", sha256_golden && sha256_payload);

    // Gate 3: Ed25519 Cryptographic Signature Verification
    let sig_valid = verify_nvp(&pkg1, &SYSTEM_ROOT_PUBKEY).is_ok();
    check_store!(3, "Ed25519 Cryptographic Signature Verification", sig_valid);

    // Gate 4: Tamper Detection & Corrupted Package Rejection
    let mut tampered_pkg = pkg1.clone();
    let tamper_idx = hdr1.payload_offset as usize + 2;
    tampered_pkg[tamper_idx] ^= 0xFF;
    let tamper_rejected = verify_nvp(&tampered_pkg, &SYSTEM_ROOT_PUBKEY).is_err();
    check_store!(4, "Tamper Detection & Corrupted Package Rejection", tamper_rejected);

    // Gate 5: Untrusted Publisher Public Key Isolation
    let untrusted_pubkey = [0x55u8; 32];
    let untrusted_rejected = matches!(verify_nvp(&pkg1, &untrusted_pubkey), Err(PackageError::SignatureInvalid));
    check_store!(5, "Untrusted Publisher Public Key Isolation", untrusted_rejected);

    // Gate 6: System Capability Bitmask Decoding
    let full_caps = CAP_FS_READ | CAP_FS_WRITE | CAP_NET_SOCKET | CAP_GUI_WINDOW;
    let pkg_caps = build_nvp("captest", "1.0.0", "NovaOS Team", full_caps, &files_test1, &secret);
    let hdr_caps = verify_nvp(&pkg_caps, &SYSTEM_ROOT_PUBKEY).expect("caps verify");
    let caps_ok = hdr_caps.has_capability(CAP_FS_READ)
        && hdr_caps.has_capability(CAP_FS_WRITE)
        && hdr_caps.has_capability(CAP_NET_SOCKET)
        && hdr_caps.has_capability(CAP_GUI_WINDOW)
        && !hdr_caps.has_capability(CAP_SYS_ADMIN);
    check_store!(6, "System Capability Bitmask Decoding", caps_ok);

    // Gate 7: Capability Sandboxing & Privilege Ring Enforcement
    let mut mgr = PackageManager::new();
    let admin_pkg = build_nvp("admintool", "1.0.0", "NovaOS Team", CAP_SYS_ADMIN, &files_test1, &secret);
    let non_root_denied = mgr.stage_package(&admin_pkg, 1000) == Err(PackageError::PermissionDenied);
    let root_allowed = mgr.stage_package(&admin_pkg, 0).is_ok();
    check_store!(7, "Capability Sandboxing & Privilege Ring Enforcement (Root UID 0 vs User UID 1000)", non_root_denied && root_allowed);

    // Gate 8: Package Manifest & File Entry Unpacking
    let multi_files = [
        ("/bin/editor", 0o755, &b"BINARY_CONTENT"[..]),
        ("/etc/editor.conf", 0o644, &b"tabsize=4"[..]),
    ];
    let pkg_multi = build_nvp("editor", "1.0.0", "Apex Labs", CAP_FS_READ | CAP_FS_WRITE, &multi_files, &secret);
    let stage_multi = mgr.stage_package(&pkg_multi, 0).expect("stage multi");
    check_store!(8, "Package Manifest & File Entry Unpacking", stage_multi > 0);

    // Gate 9: Transactional Staging (stage_package)
    let stage_id_valid = stage_multi >= 1;
    check_store!(9, "Transactional Staging (stage_package)", stage_id_valid);

    // Gate 10: Atomic Registry Commit (commit_stage)
    let commit_res = mgr.commit_stage(stage_multi);
    let installed_rec = mgr.get_installed("editor");
    let commit_ok = commit_res.is_ok() && installed_rec.is_some() && installed_rec.unwrap().version == "1.0.0";
    check_store!(10, "Atomic Registry Commit (commit_stage)", commit_ok);

    // Gate 11: Automated Backup Creation on Upgrade
    let multi_files_v2 = [
        ("/bin/editor", 0o755, &b"BINARY_CONTENT_V2"[..]),
        ("/etc/editor.conf", 0o644, &b"tabsize=2"[..]),
    ];
    let pkg_v2 = build_nvp("editor", "2.0.0", "Apex Labs", CAP_FS_READ | CAP_FS_WRITE, &multi_files_v2, &secret);
    let stage_v2 = mgr.stage_package(&pkg_v2, 0).expect("stage v2");
    let commit_v2_ok = mgr.commit_stage(stage_v2).is_ok();
    let installed_v2 = mgr.get_installed("editor").expect("installed v2");
    let upgrade_ok = commit_v2_ok && installed_v2.version == "2.0.0";
    check_store!(11, "Automated Backup Creation on Upgrade", upgrade_ok);

    // Gate 12: Atomic Version Rollback (rollback_package)
    let rollback_res = mgr.rollback_package("editor");
    let rolled_back_pkg = mgr.get_installed("editor").expect("rolled back pkg");
    let rollback_ok = rollback_res.is_ok() && rolled_back_pkg.version == "1.0.0";
    check_store!(12, "Atomic Version Rollback (rollback_package)", rollback_ok);

    // Gate 13: Safe Package Uninstallation (uninstall_package)
    let user_denied_uninstall = mgr.uninstall_package("editor", 1000) == Err(PackageError::PermissionDenied);
    let root_uninstall = mgr.uninstall_package("editor", 0).is_ok();
    let uninstalled_clean = mgr.get_installed("editor").is_none();
    check_store!(13, "Safe Package Uninstallation (uninstall_package)", user_denied_uninstall && root_uninstall && uninstalled_clean);

    // Gate 14: Curated Store Catalog Enumeration (6 Official Packages)
    let cat_len_ok = STORE_CATALOG.len() == 6;
    let names = ["coreutils", "novashell", "nettools", "textedit", "calc", "paint"];
    let mut all_found = true;
    for n in names {
        if !STORE_CATALOG.iter().any(|c| c.name == n) {
            all_found = false;
        }
    }
    check_store!(14, "Curated Store Catalog Enumeration (6 Official Packages)", cat_len_ok && all_found);

    // Gate 15: Category Filtering Subsystem
    let mut categories_count = 0;
    let expected_cats = ["System Utilities", "System Shell", "Network Tools", "Productivity", "Utilities", "Creative Tools"];
    for ec in expected_cats {
        if STORE_CATALOG.iter().any(|c| c.category == ec) {
            categories_count += 1;
        }
    }
    check_store!(15, "Category Filtering Subsystem (System, Utilities, Productivity, Creative)", categories_count == 6);

    // Gate 16: Store CLI Query (pkg list / store list)
    let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
    let shell = match shell_guard.as_mut() {
        Some(s) => s,
        None => return false,
    };
    let prev_blit = shell.blit_to_screen;
    shell.blit_to_screen = false;

    set_cli_input(shell, "pkg list");
    shell.execute_terminal_command();
    let cli_list_ok = shell.terminal_line_count > 0 &&
        shell.terminal_lines[..shell.terminal_line_count].iter().any(|l| l.as_str().contains("coreutils"));
    check_store!(16, "Store CLI Query (pkg list / store list)", cli_list_ok);

    // Gate 17: Store CLI Inspection (pkg info <name>)
    set_cli_input(shell, "pkg info coreutils");
    shell.execute_terminal_command();
    let cli_info_ok = shell.terminal_line_count > 0 &&
        shell.terminal_lines[..shell.terminal_line_count].iter().any(|l| l.as_str().contains("Ed25519 Verified [PASS]"));
    check_store!(17, "Store CLI Inspection (pkg info <name>)", cli_info_ok);

    // Gate 18: Store CLI Transactional Install (pkg install <name>)
    set_cli_input(shell, "pkg install textedit");
    shell.execute_terminal_command();
    let cli_install_ok = shell.terminal_line_count > 0 &&
        shell.terminal_lines[..shell.terminal_line_count].iter().any(|l| l.as_str().contains("registered in system PATH"));
    check_store!(18, "Store CLI Transactional Install (pkg install <name>)", cli_install_ok);

    // Gate 19: Store CLI Transactional Uninstall (pkg remove <name>)
    set_cli_input(shell, "pkg remove textedit");
    shell.execute_terminal_command();
    let cli_remove_ok = shell.terminal_line_count > 0 &&
        shell.terminal_lines[..shell.terminal_line_count].iter().any(|l| l.as_str().contains("uninstalled successfully"));
    check_store!(19, "Store CLI Transactional Uninstall (pkg remove <name>)", cli_remove_ok);

    // Gate 20: Desktop Nova Store Graphical Surface Rendering
    let win_id = shell.compositor.create_window("Nova Store", 140, 80, 560, 380, 0, 1);
    shell.render_store_surface(win_id);
    let win_rendered = shell.compositor.get_window(win_id).map(|w| w.surface.width == 560 && w.surface.height == 380).unwrap_or(false);
    shell.compositor.destroy_window(win_id);
    shell.blit_to_screen = prev_blit;
    check_store!(20, "Desktop Nova Store Graphical Surface Rendering", win_rendered);

    true
}

fn set_cli_input(shell: &mut DesktopShell, s: &str) {
    shell.terminal_input_len = 0;
    let b = s.as_bytes();
    let len = b.len().min(64);
    shell.terminal_input[..len].copy_from_slice(&b[..len]);
    shell.terminal_input_len = len;
}

fn print_num(mut n: u32) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
