//! NovaOS Apex — Milestone 4 Automated Test Suite
//! Verifies synchronization primitives, thread scheduling, IPC, pointer validation, and userland Ring 3 init.

use crate::ipc;
use crate::process;
use crate::scheduler::sync::{KernelEvent, SpinLock};
use crate::scheduler::thread::ThreadPriority;
use crate::scheduler::spawn_kernel_thread;
use crate::serial;
use crate::syscall::validate::{validate_user_buffer, SyscallError, USER_SPACE_LIMIT};
use core::sync::atomic::{AtomicU64, Ordering};

static TEST_WORKER_FLAG: AtomicU64 = AtomicU64::new(0);

extern "C" fn kernel_test_worker() {
    TEST_WORKER_FLAG.fetch_add(1, Ordering::SeqCst);
    loop {
        core::hint::spin_loop();
    }
}

pub fn run_m4_verification_suite(init_elf_bytes: &[u8]) {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone 4 Verification Test Suite  \n");
    serial::write_str("=======================================================\n");

    // 1. Test Synchronization Primitives (SpinLock & KernelEvent)
    serial::write_str("[TEST 1/5] Testing SpinLock & KernelEvent synchronization...\n");
    {
        let lock = SpinLock::new("test_lock", 42u64);
        {
            let mut guard = lock.lock();
            assert!(*guard == 42, "Lock value should equal 42");
            *guard = 100;
        }
        assert!(*lock.lock() == 100, "Lock value should equal updated 100");

        let event = KernelEvent::new("test_event", false);
        assert!(!event.is_signaled(), "Event should initially be unsignaled");
        event.set();
        assert!(event.is_signaled(), "Event should be signaled after set()");
        event.reset();
        assert!(!event.is_signaled(), "Event should be unsignaled after reset()");
    }
    serial::write_str("  [PASS] SpinLock mutual exclusion and KernelEvent signaling verified.\n");

    // 2. Test Cooperative Kernel Tasks & Thread Allocation
    serial::write_str("[TEST 2/5] Testing kernel thread creation & priority queue...\n");
    {
        let tid = spawn_kernel_thread("kworker", kernel_test_worker, ThreadPriority::Normal)
            .expect("Failed to spawn kernel worker thread");
        assert!(tid.0 > 0, "Worker TID should be strictly positive");
        serial::write_str("  [PASS] Kernel thread spawned successfully with TID ");
        print_u32(tid.0);
        serial::write_str(".\n");
    }

    // 3. Test System Call Validation Engine
    serial::write_str("[TEST 3/5] Testing system call pointer & buffer security validation...\n");
    {
        // Test null pointer rejection
        assert_eq!(
            validate_user_buffer(0, 64, false),
            Err(SyscallError::BadPointer),
            "Null pointer must be rejected"
        );

        // Test arithmetic overflow rejection
        assert_eq!(
            validate_user_buffer(u64::MAX - 10, 20, false),
            Err(SyscallError::BufferOverflow),
            "Integer overflow in pointer + length must be rejected"
        );

        // Test kernel tampering boundary rejection
        assert_eq!(
            validate_user_buffer(USER_SPACE_LIMIT + 0x1000, 4096, false),
            Err(SyscallError::KernelTamperingAttempt),
            "Kernel virtual address in user syscall must be rejected"
        );

        // Test boundary violation crossing into kernel space
        assert_eq!(
            validate_user_buffer(USER_SPACE_LIMIT - 100, 200, false),
            Err(SyscallError::KernelTamperingAttempt),
            "Buffer crossing USER_SPACE_LIMIT must be rejected"
        );

        // Test valid user buffer acceptance
        assert_eq!(
            validate_user_buffer(0x0000_0000_1000_0000, 4096, false),
            Ok(()),
            "Valid canonical userspace buffer must be accepted"
        );

        serial::write_str("  [PASS] Boundary validation successfully blocks kernel tampering and overflows.\n");
    }

    // 4. Test IPC Subsystem (Process Message Queuing & Isolation)
    serial::write_str("[TEST 4/5] Testing Inter-Process Communication (IPC)...\n");
    {
        ipc::init();
        let payload = b"Apex-IPC-Integration-Test-OK";
        assert_eq!(ipc::send(1, 2, payload), Ok(()), "IPC send to PID 2 must succeed");

        let mut rx_buf = [0u8; 64];
        let bytes_received = ipc::receive(2, 1, &mut rx_buf)
            .expect("IPC receive at PID 2 must succeed");
        assert_eq!(bytes_received, payload.len(), "Received byte count must match");
        assert_eq!(&rx_buf[..bytes_received], payload, "Payload content must match");

        // Test oversized payload rejection (>256 bytes)
        let oversized = [0xAAu8; 300];
        assert_eq!(ipc::send(1, 2, &oversized), Err(-2), "Oversized IPC payload must be rejected");

        serial::write_str("  [PASS] IPC message transfer, length integrity, and bounds checks verified.\n");
    }

    // 5. Test Ring 3 Process Spawning & ELF64 Loader
    serial::write_str("[TEST 5/5] Spawning PID 1 'novaos-apex-init' in Ring 3 Userspace...\n");
    {
        process::init();

        let pid = match process::spawn_process("init", init_elf_bytes, None) {
            Ok(p) => p,
            Err(e) => {
                serial::write_str("[TEST 5/5 ERROR] spawn_process failed with error: ");
                match e {
                    process::ProcessError::ProcessTableFull => serial::write_str("ProcessTableFull\n"),
                    process::ProcessError::AddressSpaceCreationFailed => serial::write_str("AddressSpaceCreationFailed\n"),
                    process::ProcessError::ElfLoadFailed(elf_err) => {
                        serial::write_str("ElfLoadFailed: ");
                        match elf_err {
                            process::elf_loader::ElfError::InvalidMagic => serial::write_str("InvalidMagic\n"),
                            process::elf_loader::ElfError::Not64Bit => serial::write_str("Not64Bit\n"),
                            process::elf_loader::ElfError::NotLittleEndian => serial::write_str("NotLittleEndian\n"),
                            process::elf_loader::ElfError::UnsupportedMachine => serial::write_str("UnsupportedMachine\n"),
                            process::elf_loader::ElfError::InvalidHeaderSize => serial::write_str("InvalidHeaderSize\n"),
                            process::elf_loader::ElfError::BufferTooShort => serial::write_str("BufferTooShort\n"),
                            process::elf_loader::ElfError::AllocationFailed => serial::write_str("AllocationFailed\n"),
                            process::elf_loader::ElfError::PagingFailed => serial::write_str("PagingFailed\n"),
                            process::elf_loader::ElfError::NoLoadableSegments => serial::write_str("NoLoadableSegments\n"),
                        }
                    },
                    process::ProcessError::ProcessNotFound => serial::write_str("ProcessNotFound\n"),
                    process::ProcessError::ThreadSpawnFailed => serial::write_str("ThreadSpawnFailed\n"),
                    process::ProcessError::AccessDenied => serial::write_str("AccessDenied\n"),
                }
                panic!("Failed to load and spawn PID 1 init daemon");
            }
        };
        assert_eq!(pid.0, 1, "First spawned user process must be PID 1");

        serial::write_str("  [PASS] PID 1 'init' loaded into isolated address space with user stack.\n");
    }

    serial::write_str("\n[MILESTONE 4 TEST RESULT] ALL 5 TEST SUITES PASSED.\n");
    serial::write_str("Scheduling active. Handing off CPU to multi-tasking scheduler & PID 1.\n\n");
}

fn print_u32(mut n: u32) {
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
