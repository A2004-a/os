//! NovaOS Zero — Milestone M4 Automated Test Suite
//! Validates Priority Scheduling, Sleep/Wake Queues, 14 Syscalls, Kernel Memory Boundary Protection,
//! and ELF64 Userland Executable Validation.

use crate::scheduler::round_robin::PriorityScheduler;
use crate::scheduler::task::{Task, TaskContext, TaskId, TaskState};
use crate::syscall::abi::*;
use crate::syscall::dispatch_syscall;
use crate::process::elf_loader::{Elf64Header, ElfError, ElfLoader, ELF_MAGIC, ELF_CLASS_64, ELF_DATA_2LSB, ET_EXEC, EM_X86_64};
use crate::serial_println;

pub fn run_m4_tests() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M4 Automated Verification");
    serial_println!("=======================================================");

    test_task_context_structures();
    test_priority_scheduler_ordering();
    test_sleep_wake_queue();
    test_all_14_syscalls();
    test_kernel_pointer_protection_fuzz();
    test_elf_validator_and_security();

    serial_println!("[M4 SUCCESS] All Milestone M4 Verification Tests Passed Cleanly!\n");
}

fn test_task_context_structures() {
    serial_println!("[TEST 1/6] Task Context & Register State...");

    let mut ctx = TaskContext::new(0x0000_0000_0040_0000);
    assert_eq!(ctx.rip, 0x0000_0000_0040_0000);
    assert_eq!(ctx.rbx, 0);

    ctx.rbx = 0xCAFE_BABE;
    ctx.r12 = 0x1234_5678;
    assert_eq!(ctx.rbx, 0xCAFE_BABE);
    assert_eq!(ctx.r12, 0x1234_5678);

    serial_println!("  -> TaskContext register allocation verified.");
}

fn test_priority_scheduler_ordering() {
    serial_println!("[TEST 2/6] Priority-Aware Task Scheduling...");

    let mut sched = PriorityScheduler::new();

    let low_task = Task {
        id: TaskId(1),
        process_id: 1,
        state: TaskState::Ready,
        context: TaskContext::new(0x20000),
        stack_bottom: 0x10000,
        stack_top: 0x11000,
        priority: 1,
        time_slices: 0,
    };

    let high_task = Task {
        id: TaskId(2),
        process_id: 1,
        state: TaskState::Ready,
        context: TaskContext::new(0x40000),
        stack_bottom: 0x30000,
        stack_top: 0x31000,
        priority: 10,
        time_slices: 0,
    };

    let mid_task = Task {
        id: TaskId(3),
        process_id: 1,
        state: TaskState::Ready,
        context: TaskContext::new(0x60000),
        stack_bottom: 0x50000,
        stack_top: 0x51000,
        priority: 5,
        time_slices: 0,
    };

    assert!(sched.add_task(low_task).is_ok());
    assert!(sched.add_task(high_task).is_ok());
    assert!(sched.add_task(mid_task).is_ok());

    // Next scheduled task must be TaskId(2) due to highest priority (10)
    let selected = sched.schedule_next();
    assert_eq!(selected, Some(TaskId(2)));

    serial_println!("  -> Priority selection verified: TaskId(2) chosen with priority 10.");
}

fn test_sleep_wake_queue() {
    serial_println!("[TEST 3/6] Sleep/Wake Queue & Monotonic Timer Synchronization...");

    let mut sched = PriorityScheduler::new();

    let sleeping_task = Task {
        id: TaskId(10),
        process_id: 1,
        state: TaskState::Sleeping(500), // Sleep until tick 500
        context: TaskContext::new(0x20000),
        stack_bottom: 0x10000,
        stack_top: 0x11000,
        priority: 5,
        time_slices: 0,
    };

    assert!(sched.add_task(sleeping_task).is_ok());

    // At tick 100, task must remain sleeping (not scheduled)
    sched.update_sleep_queues(100);
    assert_eq!(sched.schedule_next(), None);

    // At tick 499, task must still remain sleeping
    sched.update_sleep_queues(499);
    assert_eq!(sched.schedule_next(), None);

    // At tick 500, sleep queue wakeups must transition state to Ready
    sched.update_sleep_queues(500);
    assert_eq!(sched.schedule_next(), Some(TaskId(10)));

    serial_println!("  -> Sleep queue wakeups verified: TaskId(10) awoke at tick 500.");
}

fn test_all_14_syscalls() {
    serial_println!("[TEST 4/6] System Call Dispatcher (All 14 Syscalls)...");

    let test_str = "NovaOS Syscall ABI Test Payload\n";
    let user_buf = test_str.as_ptr() as usize;
    let user_len = test_str.len();

    // 1. SYS_EXIT
    assert_eq!(dispatch_syscall(SYS_EXIT, 0, 0, 0), 0);

    // 2. SYS_READ
    assert_eq!(dispatch_syscall(SYS_READ, 0, user_buf, user_len), 0);

    // 3. SYS_WRITE
    assert_eq!(dispatch_syscall(SYS_WRITE, 1, user_buf, user_len), user_len as isize);

    // 4. SYS_OPEN
    let fd = dispatch_syscall(SYS_OPEN, user_buf, 0, 0);
    assert!(fd >= 3);

    // 5. SYS_CLOSE
    assert_eq!(dispatch_syscall(SYS_CLOSE, fd as usize, 0, 0), 0);

    // 6. SYS_SPAWN
    let pid = dispatch_syscall(SYS_SPAWN, user_buf, user_len, 0);
    assert!(pid > 0);

    // 7. SYS_WAIT
    assert_eq!(dispatch_syscall(SYS_WAIT, pid as usize, 0, 0), 0);

    // 8. SYS_YIELD
    assert_eq!(dispatch_syscall(SYS_YIELD, 0, 0, 0), 0);

    // 9. SYS_SLEEP
    assert_eq!(dispatch_syscall(SYS_SLEEP, 10, 0, 0), 0);

    // 10. SYS_GET_TIME
    let uptime = dispatch_syscall(SYS_GET_TIME, 0, 0, 0);
    assert!(uptime >= 0);

    // 11. SYS_MAP_MEMORY
    let user_vaddr = 0x0000_1000_0000usize;
    assert_eq!(dispatch_syscall(SYS_MAP_MEMORY, user_vaddr, 4096, 0), user_vaddr as isize);

    // 12. SYS_UNMAP_MEMORY
    assert_eq!(dispatch_syscall(SYS_UNMAP_MEMORY, user_vaddr, 4096, 0), 0);

    // 13. SYS_SEND_IPC
    assert_eq!(dispatch_syscall(SYS_SEND_IPC, 100, user_buf, user_len), user_len as isize);

    // 14. SYS_RECEIVE_IPC
    assert_eq!(dispatch_syscall(SYS_RECEIVE_IPC, 100, user_buf, user_len), 0);

    // Invalid syscall vector
    assert_eq!(dispatch_syscall(999, 0, 0, 0), ENOSYS);

    serial_println!("  -> All 14 system call dispatches verified cleanly.");
}

fn test_kernel_pointer_protection_fuzz() {
    serial_println!("[TEST 5/6] Kernel Memory Space & Boundary Protection Fuzzing...");

    let kernel_ptr = 0xFFFF_8000_0000_0000usize;
    let null_ptr = 0x0000_0000_0000_0000usize;
    let zero_page_ptr = 0x0000_0000_0000_0500usize;
    let overflow_ptr = 0x0000_7FFF_FFFF_FFFFusize;

    // Fuzz SYS_WRITE: Kernel pointer must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_WRITE, 1, kernel_ptr, 16), EFAULT);

    // Fuzz SYS_WRITE: Null pointer must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_WRITE, 1, null_ptr, 16), EFAULT);

    // Fuzz SYS_WRITE: Zero-page pointer (< 0x1000) must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_WRITE, 1, zero_page_ptr, 16), EFAULT);

    // Fuzz SYS_WRITE: Arithmetic overflow beyond canonical address space must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_WRITE, 1, overflow_ptr, 100), EFAULT);

    // Fuzz SYS_READ: Kernel pointer must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_READ, 0, kernel_ptr, 64), EFAULT);

    // Fuzz SYS_SPAWN: Kernel pointer must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_SPAWN, kernel_ptr, 128, 0), EFAULT);

    // Fuzz SYS_MAP_MEMORY: Mapping kernel space must trigger EFAULT
    assert_eq!(dispatch_syscall(SYS_MAP_MEMORY, kernel_ptr, 4096, 0), EFAULT);

    // Fuzz SYS_CLOSE: Attempting to close stdout (1) or stdin (0) must trigger EBADF
    assert_eq!(dispatch_syscall(SYS_CLOSE, 1, 0, 0), EBADF);
    assert_eq!(dispatch_syscall(SYS_CLOSE, 2, 0, 0), EBADF);

    // Fuzz SYS_WRITE: Writing to unopened descriptor (fd 99) must trigger EBADF
    let test_buf = "Safe String".as_ptr() as usize;
    assert_eq!(dispatch_syscall(SYS_WRITE, 99, test_buf, 11), EBADF);

    serial_println!("  -> Kernel pointer boundary isolation verified. All fuzz attempts rejected.");
}

fn test_elf_validator_and_security() {
    serial_println!("[TEST 6/6] ELF64 Executable Header & Boundary Validation...");

    // 1. Buffer too small
    let small = [0u8; 10];
    assert!(matches!(ElfLoader::parse_header(&small), Err(ElfError::BufferTooSmall)));

    // 2. Synthesize valid AMD64 ELF Header
    let mut valid_header = Elf64Header {
        magic: ELF_MAGIC,
        class: ELF_CLASS_64,
        endian: ELF_DATA_2LSB,
        version: 1,
        os_abi: 0,
        abi_version: 0,
        pad: [0; 7],
        elf_type: ET_EXEC,
        machine: EM_X86_64,
        version2: 1,
        entry: 0x0000_0000_0040_0000, // Valid user space entry point
        phoff: 64,
        shoff: 0,
        flags: 0,
        ehsize: 64,
        phentsize: 56,
        phnum: 0,
        shentsize: 64,
        shnum: 0,
        shstrndx: 0,
    };

    let header_bytes = unsafe {
        core::slice::from_raw_parts(
            &valid_header as *const _ as *const u8,
            core::mem::size_of::<Elf64Header>(),
        )
    };

    let parsed = ElfLoader::parse_header(header_bytes);
    assert!(parsed.is_ok());
    let entry_point = parsed.unwrap().entry;
    assert_eq!(entry_point, 0x0000_0000_0040_0000);

    // 3. Reject Kernel Space Entry Point (Address Space Violation)
    valid_header.entry = 0xFFFF_8000_0000_0000;
    let kernel_bytes = unsafe {
        core::slice::from_raw_parts(
            &valid_header as *const _ as *const u8,
            core::mem::size_of::<Elf64Header>(),
        )
    };
    assert!(matches!(ElfLoader::parse_header(kernel_bytes), Err(ElfError::KernelAddressViolation)));

    // 4. Reject Non-AMD64 Machine Architecture
    valid_header.entry = 0x0000_0000_0040_0000;
    valid_header.machine = 0x03; // x86 32-bit
    let bad_arch_bytes = unsafe {
        core::slice::from_raw_parts(
            &valid_header as *const _ as *const u8,
            core::mem::size_of::<Elf64Header>(),
        )
    };
    assert!(matches!(ElfLoader::parse_header(bad_arch_bytes), Err(ElfError::NotX86_64)));

    // 5. Reject Invalid Magic
    valid_header.machine = EM_X86_64;
    valid_header.magic = [0x00, 0x00, 0x00, 0x00];
    let bad_magic_bytes = unsafe {
        core::slice::from_raw_parts(
            &valid_header as *const _ as *const u8,
            core::mem::size_of::<Elf64Header>(),
        )
    };
    assert!(matches!(ElfLoader::parse_header(bad_magic_bytes), Err(ElfError::InvalidMagic)));

    serial_println!("  -> ELF64 parser correctly validates AMD64 format and rejects kernel address breaches.");
}
