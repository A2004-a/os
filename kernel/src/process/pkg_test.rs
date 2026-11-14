//! NovaOS Zero — Milestone M9 Automated Verification Suite
//! Validates NPK binary framing, Adler-32 integrity verification, sandbox capability
//! enforcement, and the native package manager registry lifecycle.

use crate::process::pkg_mgr::{
    verify_package_bytes, PackageError, PackageHeader, PackageManifest, PackageManager,
    CAP_ADMIN, CAP_ALL, CAP_AUDIO, CAP_GRAPHICS, CAP_IPC, CAP_NET, CAP_NONE, CAP_STORAGE,
    HEADER_SIZE, MAX_INSTALLED_PACKAGES, NPKG_MAGIC, NPKG_VERSION,
};
use crate::serial_println;

pub fn run_pkg_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M9 Automated Verification");
    serial_println!("=======================================================");

    test_npk_header_encoding_and_checksum();
    test_corrupted_package_rejection();
    test_sandbox_capability_enforcement();
    test_package_manager_lifecycle();
    test_package_registry_capacity();

    serial_println!("[M9 SUCCESS] All Milestone M9 Verification Tests Passed Cleanly!\n");
}

fn test_npk_header_encoding_and_checksum() {
    serial_println!("[TEST 1/5] NPK Binary Encoding, Offsets, and Integrity Checksum...");

    let manifest_bytes = b"app_id=org.novaos.files\nname=FileManager\nversion=1.0.0\n";
    let payload_bytes = b"\x7FELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00";

    // Calculate sequential checksum
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in manifest_bytes.iter() {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    for &byte in payload_bytes.iter() {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    let checksum = (b << 16) | a;

    let m_off = HEADER_SIZE as u32;
    let m_sz = manifest_bytes.len() as u32;
    let p_off = m_off + m_sz;
    let p_sz = payload_bytes.len() as u32;

    let header = PackageHeader::new(
        CAP_STORAGE | CAP_GRAPHICS,
        m_off,
        m_sz,
        p_off,
        p_sz,
        checksum,
    );

    let mut buffer = [0u8; 256];
    unsafe {
        let hdr_ptr = buffer.as_mut_ptr() as *mut PackageHeader;
        core::ptr::write_unaligned(hdr_ptr, header);
    }
    buffer[m_off as usize..(m_off + m_sz) as usize].copy_from_slice(manifest_bytes);
    buffer[p_off as usize..(p_off + p_sz) as usize].copy_from_slice(payload_bytes);

    let total_len = (p_off + p_sz) as usize;
    let result = verify_package_bytes(&buffer[..total_len]);
    assert!(result.is_ok());

    let (parsed_hdr, parsed_manifest, parsed_payload) = result.unwrap();
    assert_eq!(parsed_hdr.get_magic(), NPKG_MAGIC);
    assert_eq!(parsed_hdr.get_version(), NPKG_VERSION);
    assert_eq!(parsed_hdr.get_capabilities(), CAP_STORAGE | CAP_GRAPHICS);
    assert_eq!(parsed_manifest, manifest_bytes);
    assert_eq!(parsed_payload, payload_bytes);

    serial_println!("  -> NPK framing, byte alignment, and checksum verification verified.");
}

fn test_corrupted_package_rejection() {
    serial_println!("[TEST 2/5] Package Integrity & Tamper Protection Validation...");

    let manifest = b"app_id=org.novaos.test\n";
    let payload = b"PAYLOAD_DATA";

    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in manifest.iter().chain(payload.iter()) {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    let checksum = (b << 16) | a;

    let m_off = HEADER_SIZE as u32;
    let m_sz = manifest.len() as u32;
    let p_off = m_off + m_sz;
    let p_sz = payload.len() as u32;

    let header = PackageHeader::new(CAP_NONE, m_off, m_sz, p_off, p_sz, checksum);

    let mut buffer = [0u8; 128];
    unsafe {
        let hdr_ptr = buffer.as_mut_ptr() as *mut PackageHeader;
        core::ptr::write_unaligned(hdr_ptr, header);
    }
    buffer[m_off as usize..(m_off + m_sz) as usize].copy_from_slice(manifest);
    buffer[p_off as usize..(p_off + p_sz) as usize].copy_from_slice(payload);

    let total_len = (p_off + p_sz) as usize;

    // 1. Truncated buffer
    assert_eq!(verify_package_bytes(&buffer[..32]).err(), Some(PackageError::BufferTooSmall));

    // 2. Corrupted Magic
    let mut bad_magic_buffer = buffer;
    bad_magic_buffer[0] = 0xFF;
    assert_eq!(verify_package_bytes(&bad_magic_buffer[..total_len]).err(), Some(PackageError::InvalidMagic));

    // 3. Corrupted Payload bit (Checksum mismatch)
    let mut tampered_buffer = buffer;
    tampered_buffer[p_off as usize] ^= 0x01; // flip 1 bit
    assert_eq!(verify_package_bytes(&tampered_buffer[..total_len]).err(), Some(PackageError::CorruptedChecksum));

    serial_println!("  -> Buffer size bounds, magic checking, and checksum tamper rejection verified.");
}

fn test_sandbox_capability_enforcement() {
    serial_println!("[TEST 3/5] Sandbox Capability Ceilings & Security Boundaries...");

    // Standard unprivileged desktop user ceiling
    let user_ceiling = CAP_NET | CAP_STORAGE | CAP_GRAPHICS | CAP_AUDIO | CAP_IPC;
    let mut pkg_mgr = PackageManager::new(user_ceiling);

    // 1. Safe package with standard user capabilities
    let safe_manifest = PackageManifest::new(
        "org.novaos.calc",
        "Calculator",
        "1.0.0",
        "NovaOS Community",
        "/bin/calc",
        CAP_GRAPHICS,
    );
    assert_eq!(pkg_mgr.install(safe_manifest, 4096), Ok(()));

    // 2. Malicious / unauthorized package requesting CAP_ADMIN without privileges
    let rogue_manifest = PackageManifest::new(
        "org.novaos.rootexploit",
        "RootApp",
        "0.1.0",
        "Unknown",
        "/bin/exploit",
        CAP_ADMIN,
    );
    assert_eq!(pkg_mgr.install(rogue_manifest, 2048), Err(PackageError::CapabilityViolation));

    serial_println!("  -> Capability bitmask checks and admin elevation barriers verified.");
}

fn test_package_manager_lifecycle() {
    serial_println!("[TEST 4/5] Package Manager Registry Lifecycle (Install, Query, Launch, Uninstall)...");

    let mut pkg_mgr = PackageManager::new(CAP_ALL);

    // Install 3 Core Native Applications
    let file_mgr = PackageManifest::new(
        "org.novaos.files",
        "File Manager",
        "1.0.0",
        "NovaOS Platform Engineering",
        "/apps/file_manager",
        CAP_STORAGE | CAP_GRAPHICS,
    );
    let sys_mon = PackageManifest::new(
        "org.novaos.sysmon",
        "System Monitor",
        "1.0.0",
        "NovaOS Platform Engineering",
        "/apps/sys_monitor",
        CAP_IPC | CAP_GRAPHICS,
    );
    let settings = PackageManifest::new(
        "org.novaos.settings",
        "Settings",
        "1.0.0",
        "NovaOS Platform Engineering",
        "/apps/settings",
        CAP_STORAGE | CAP_NET | CAP_GRAPHICS,
    );

    assert_eq!(pkg_mgr.install(file_mgr, 32768), Ok(()));
    assert_eq!(pkg_mgr.install(sys_mon, 24576), Ok(()));
    assert_eq!(pkg_mgr.install(settings, 18432), Ok(()));
    assert_eq!(pkg_mgr.count(), 3);

    // Duplicate install must fail
    assert_eq!(pkg_mgr.install(file_mgr, 32768), Err(PackageError::AlreadyInstalled));

    // Lookup installed package
    let looked_up = pkg_mgr.find("org.novaos.sysmon");
    assert!(looked_up.is_some());
    let pkg = looked_up.unwrap();
    assert_eq!(pkg.manifest.name, "System Monitor");
    assert_eq!(pkg.manifest.required_caps, CAP_IPC | CAP_GRAPHICS);

    // Launch application and track launch counter
    assert_eq!(pkg_mgr.launch("org.novaos.files"), Ok(1));
    assert_eq!(pkg_mgr.launch("org.novaos.files"), Ok(2));
    assert_eq!(pkg_mgr.find("org.novaos.files").unwrap().launch_count, 2);

    // Uninstall application
    assert_eq!(pkg_mgr.uninstall("org.novaos.settings"), Ok(()));
    assert_eq!(pkg_mgr.count(), 2);
    assert!(pkg_mgr.find("org.novaos.settings").is_none());

    // Uninstall non-existent
    assert_eq!(pkg_mgr.uninstall("org.novaos.nonexistent"), Err(PackageError::PackageNotFound));

    serial_println!("  -> Package registration, query, launch tracking, and removal verified.");
}

fn test_package_registry_capacity() {
    serial_println!("[TEST 5/5] Package Registry Capacity Limits & Boundary Safety...");

    let mut pkg_mgr = PackageManager::new(CAP_ALL);

    // Install up to MAX_INSTALLED_PACKAGES (16)
    let app_ids = [
        "app.0", "app.1", "app.2", "app.3", "app.4", "app.5", "app.6", "app.7",
        "app.8", "app.9", "app.10", "app.11", "app.12", "app.13", "app.14", "app.15",
    ];

    for &id in app_ids.iter() {
        let manifest = PackageManifest::new(id, "TestApp", "1.0", "NovaOS", "/bin/app", CAP_NONE);
        assert_eq!(pkg_mgr.install(manifest, 1024), Ok(()));
    }
    assert_eq!(pkg_mgr.count(), MAX_INSTALLED_PACKAGES);

    // 17th package must return RegistryFull
    let overflow_manifest = PackageManifest::new("app.overflow", "Overflow", "1.0", "NovaOS", "/bin/app", CAP_NONE);
    assert_eq!(pkg_mgr.install(overflow_manifest, 1024), Err(PackageError::RegistryFull));

    serial_println!("  -> Registry fixed-capacity bounds enforcement verified.");
}
