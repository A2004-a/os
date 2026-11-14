//! NovaOS Apex — Kernel Stress Testing & Memory Leak Diagnostics Engine
//! Rigorously executes high-churn process lifecycles, VFS write-churn sequences,
//! and heap canary integrity audits.

use crate::memory::heap::{get_heap_stats, CANARY_PREFIX, CANARY_SUFFIX};
use crate::serial;
use core::alloc::Layout;

/// Audits kernel heap memory for memory leaks and verifies all canary boundary guards.
pub fn test_memory_leak_and_canary_audit() -> bool {
    serial::write_str("[M12 TEST 3/6] Testing Kernel Heap Canary Walk & Memory Leak Audit...\n");

    let initial_stats = get_heap_stats();
    serial::write_str("  Initial Heap State: ");
    print_u64(initial_stats.allocated_bytes as u64);
    serial::write_str(" bytes allocated, ");
    print_u64(initial_stats.free_bytes as u64);
    serial::write_str(" bytes free\n");

    // 1. Perform 32 sequential allocations of various sizes
    let mut ptrs: [*mut u8; 32] = [core::ptr::null_mut(); 32];
    for i in 0..32 {
        let size = ((i + 1) * 32) + 16;
        let layout = match Layout::from_size_align(size, 16) {
            Ok(l) => l,
            Err(_) => return false,
        };
        unsafe {
            let ptr = core::alloc::GlobalAlloc::alloc(&crate::memory::heap::KERNEL_HEAP, layout);
            if ptr.is_null() {
                serial::write_str("  [FAIL] Allocation failed during heap stress test!\n");
                return false;
            }
            // Fill with deterministic pattern
            core::ptr::write_bytes(ptr, (0x30 + (i % 10)) as u8, size);
            ptrs[i] = ptr;

            // Verify prefix canary at ptr - 16
            let prefix_canary = core::ptr::read_unaligned(ptr.sub(16) as *const u64);
            if prefix_canary != CANARY_PREFIX {
                serial::write_str("  [FAIL] Prefix canary corrupted immediately after allocation!\n");
                return false;
            }
            // Verify suffix canary
            let aligned_size = (size + 7) & !7;
            let suffix_canary = core::ptr::read_unaligned(ptr.add(aligned_size) as *const u64);
            if suffix_canary != CANARY_SUFFIX {
                serial::write_str("  [FAIL] Suffix canary corrupted immediately after allocation!\n");
                return false;
            }
        }
    }
    serial::write_str("  [PASS] 32 canary-guarded dynamic allocations verified intact.\n");

    // 2. Free in reverse (LIFO) order to test heap reclamation
    for i in (0..32).rev() {
        let size = ((i + 1) * 32) + 16;
        let layout = Layout::from_size_align(size, 16).unwrap();
        unsafe {
            core::alloc::GlobalAlloc::dealloc(&crate::memory::heap::KERNEL_HEAP, ptrs[i], layout);
        }
    }

    let final_stats = get_heap_stats();
    serial::write_str("  Final Heap State:   ");
    print_u64(final_stats.allocated_bytes as u64);
    serial::write_str(" bytes allocated, ");
    print_u64(final_stats.free_bytes as u64);
    serial::write_str(" bytes free\n");

    if final_stats.allocated_bytes != initial_stats.allocated_bytes {
        serial::write_str("  [FAIL] Memory leak detected! Allocated bytes did not return to baseline.\n");
        return false;
    }

    serial::write_str("  [PASS] Zero memory leaks detected (100% LIFO heap reclamation verified).\n");
    true
}

/// Tests high-churn process lifecycle (creation, resource allocation, and teardown).
pub fn test_process_lifecycle_stress() -> bool {
    serial::write_str("[M12 TEST 4/6] Testing Process Lifecycle & Address Space Stress Test...\n");

    use crate::process::{create_test_process, get_process_count, reap_terminated_processes, terminate_process, ProcessId};

    let initial_count = get_process_count();

    // Spawn 10 ephemeral processes
    let mut spawned_pids = [ProcessId(0); 10];

    for i in 0..10 {
        match create_test_process("stress_worker") {
            Ok(pid) => {
                spawned_pids[i] = pid;
            }
            Err(_) => {
                serial::write_str("  [FAIL] Process creation failed under stress!\n");
                return false;
            }
        }
    }

    let peak_count = get_process_count();
    if peak_count != initial_count + 10 {
        serial::write_str("  [FAIL] Process table count mismatch during spawn phase!\n");
        return false;
    }
    serial::write_str("  [PASS] 10 concurrent address spaces and user processes provisioned.\n");

    // Terminate and clean up all 10 processes
    for &pid in &spawned_pids {
        if pid.0 != 0 {
            terminate_process(pid, 0);
        }
    }

    reap_terminated_processes();

    let final_count = get_process_count();
    if final_count != initial_count {
        serial::write_str("  [FAIL] Process table leak! Count did not return to baseline.\n");
        return false;
    }

    serial::write_str("  [PASS] 10 processes terminated, address spaces reclaimed, 0 zombie leaks.\n");
    true
}

/// Tests high-churn VFS file creation, writing, verification, and unlinking.
pub fn test_vfs_storage_stress() -> bool {
    serial::write_str("[M12 TEST 5/6] Testing VFS High-Churn Transactional Resilience Stress Test...\n");

    use crate::fs::types::{O_CREAT, O_RDWR};
    use crate::fs::vfs::get_vfs;
    use crate::security::user::Credentials;

    let creds = Credentials::root();
    let vfs_opt = get_vfs();
    if vfs_opt.is_none() {
        serial::write_str("  [FAIL] VFS not accessible!\n");
        return false;
    }
    let vfs_lock = vfs_opt.unwrap();
    let vfs = vfs_lock.as_ref().unwrap();

    // Create 10 temporary files with unique content, verify contents, then unlink
    for i in 0..10 {
        let mut path_buf = [0u8; 32];
        let p_str = b"/tmp/stress_file_";
        path_buf[..p_str.len()].copy_from_slice(p_str);
        path_buf[p_str.len()] = b'0' + (i / 10) as u8;
        path_buf[p_str.len() + 1] = b'0' + (i % 10) as u8;
        let path = core::str::from_utf8(&path_buf[..p_str.len() + 2]).unwrap();

        // 1. Create & Write
        let mut node = match vfs.open(path, O_CREAT | O_RDWR, 0o644, &creds) {
            Ok(n) => n,
            Err(_) => {
                serial::write_str("  [FAIL] Failed to create file: ");
                serial::write_str(path);
                serial::write_str("\n");
                return false;
            }
        };

        let mut payload = [0u8; 64];
        for b in payload.iter_mut() {
            *b = 0x41 + (i % 26) as u8;
        }

        if let Err(_) = node.write(&payload) {
            serial::write_str("  [FAIL] Failed to write payload to: ");
            serial::write_str(path);
            serial::write_str("\n");
            return false;
        }

        // Seek back to beginning
        if let Err(_) = node.seek(0, 0) {
            serial::write_str("  [FAIL] Failed to seek in stress file!\n");
            return false;
        }

        // 2. Read back & verify
        let mut verify_buf = [0u8; 64];
        let bytes_read = match node.read(&mut verify_buf) {
            Ok(n) => n,
            Err(_) => return false,
        };

        if bytes_read != 64 || verify_buf != payload {
            serial::write_str("  [FAIL] Data corruption during VFS stress readback!\n");
            return false;
        }

        // 3. Unlink
        if let Err(_) = vfs.unlink(path, &creds) {
            serial::write_str("  [FAIL] Failed to unlink stress file!\n");
            return false;
        }
    }

    serial::write_str("  [PASS] 10 VFS files created, written, verified byte-for-byte, and unlinked.\n");
    true
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
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
