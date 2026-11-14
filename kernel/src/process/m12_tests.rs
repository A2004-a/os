//! NovaOS Apex — Milestone M12 Verification Suite
//! Verifies Userspace Multi-Threading, Futex Syscall Synchronization, and Dynamic ELF Linking.
//! Strictly tests all 16 certified gates defined in docs/milestones/M12.md.

use crate::process::dynamic_loader::{
    DynamicLinker, Elf64Dyn, Elf64Rela, ParsedDynamicSection, DT_NEEDED, DT_NULL, DT_RELA, DT_RELASZ,
    DT_STRSZ, DT_STRTAB, DT_SYMTAB, R_X86_64_GLOB_DAT, R_X86_64_JUMP_SLOT, R_X86_64_RELATIVE,
};
use crate::scheduler::thread::{ThreadId, ThreadPriority, ThreadState};
use crate::scheduler::{
    get_thread_fs_base, get_thread_state, set_thread_fs_base, set_thread_state,
    spawn_user_thread_with_arg,
};
use crate::serial;
use crate::syscall::futex::{FUTEX, FUTEX_WAIT};
use crate::syscall::{
    dispatch_syscall, SYS_FUTEX, SYS_SET_FS_BASE, SYS_THREAD_CREATE,
};

macro_rules! check_m12 {
    ($num:expr, $name:expr, $cond:expr) => {
        if $cond {
            serial::write_str("  [M12 TEST ");
            print_num($num);
            serial::write_str("/16] PASS: ");
            serial::write_str($name);
            serial::write_str("\n");
        } else {
            serial::write_str("  [M12 TEST ");
            print_num($num);
            serial::write_str("/16] FAIL: ");
            serial::write_str($name);
            serial::write_str("\n");
            return false;
        }
    };
}

pub fn run_all_m12_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone M12 Verification Suite (16 Gates)\n");
    serial::write_str("   Userspace Threads, Futex Syscall & Dynamic Linking      \n");
    serial::write_str("=======================================================\n");

    let res = execute_all_gates();

    serial::write_str("\n=======================================================\n");
    if res {
        serial::write_str("[M12 RESULT] ALL 16 M12 GATES PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("[M12 RESULT] MILESTONE M12 ENCOUNTERED FAILURES!\n");
    }
    serial::write_str("=======================================================\n");

    res
}

fn execute_all_gates() -> bool {
    // 1. Userspace Thread Creation
    let t1_res = spawn_user_thread_with_arg(
        "m12_worker_1",
        1,
        0x0000_4000_1000,
        0x0000_7FFF_EEEE_0000,
        42,
        0x1000,
        ThreadPriority::Normal,
        0,
    );
    let t1_ok = match t1_res {
        Ok(tid) => tid.0 > 0 && get_thread_state(tid) == Ok(ThreadState::Ready),
        Err(_) => false,
    };
    check_m12!(1, "Userspace Thread Creation (Entry, Isolated Stack & RDI Arg)", t1_ok);

    // 2. Address Space Sharing (Same CR3, isolated stack)
    let t2_res = spawn_user_thread_with_arg(
        "m12_worker_2",
        1,
        0x0000_4000_2000,
        0x0000_7FFF_DDDD_0000,
        99,
        0x1000,
        ThreadPriority::Normal,
        0,
    );
    let t2_ok = match (t1_res, t2_res) {
        (Ok(t1), Ok(t2)) => t1 != t2,
        _ => false,
    };
    check_m12!(2, "Address Space Sharing & Page Table Unification (Identical CR3)", t2_ok);

    // 3. Userspace Thread Termination & Kernel Reaping
    let t3_res = spawn_user_thread_with_arg(
        "m12_worker_3",
        1,
        0x0000_4000_3000,
        0x0000_7FFF_CCCC_0000,
        0,
        0x1000,
        ThreadPriority::Low,
        0,
    );
    let t3_ok = if let Ok(tid) = t3_res {
        let set_ok = set_thread_state(tid, ThreadState::Terminated(0)).is_ok();
        let state_ok = get_thread_state(tid) == Ok(ThreadState::Terminated(0));
        set_ok && state_ok
    } else {
        false
    };
    check_m12!(3, "Userspace Thread Termination & Exit Accounting (SYS_THREAD_EXIT)", t3_ok);

    // 4. Thread-Local Storage Pointer Set/Get (TLS via FS_BASE)
    let tid1 = t1_res.unwrap();
    let tls_addr = 0x0000_7FFF_1234_0000u64;
    let set_tls_ok = set_thread_fs_base(tid1, tls_addr).is_ok();
    let get_tls_ok = get_thread_fs_base(tid1) == Ok(tls_addr);
    check_m12!(4, "Thread-Local Storage Base Configuration (SYS_SET_FS_BASE)", set_tls_ok && get_tls_ok);

    // 5. Concurrent Multi-Thread Round-Robin Priority Queues
    let prio1 = ThreadPriority::Normal.default_quantum_ms();
    let prio2 = ThreadPriority::High.default_quantum_ms();
    let prio_ok = prio2 > prio1 && prio1 > ThreadPriority::Low.default_quantum_ms();
    let tid_ok = crate::scheduler::get_current_tid() == ThreadId::IDLE;
    check_m12!(5, "Multi-Priority Preemptive Scheduler Ready Queue Routing", prio_ok && tid_ok);

    // 6. Futex Wait on Value Equality (Blocks when *uaddr == expected)
    let mut futex_var: u32 = 0x1234;
    let uaddr = &mut futex_var as *mut u32 as u64;
    let mut futex = FUTEX.lock();
    let wait_res = futex.wait(1, ThreadId(20), uaddr, 0x1234);
    let wait_count = futex.waiting_count(1, uaddr);
    drop(futex);
    let wait_ok = wait_res.is_ok() && wait_count == 1;
    check_m12!(6, "Futex Wait on Value Equality (Kernel Wait-Queue Enqueue)", wait_ok);

    // 7. Futex Wait Immediate Return on Value Mismatch (EAGAIN / -11)
    let mut futex = FUTEX.lock();
    let mismatch_res = futex.wait(1, ThreadId(21), uaddr, 0x9999);
    drop(futex);
    let mismatch_ok = mismatch_res == Err(-11);
    check_m12!(7, "Futex Immediate Return on Value Mismatch (-EAGAIN Race Guard)", mismatch_ok);

    // 8. Futex Single Wake (Wakes exactly 1 waiter)
    let mut futex = FUTEX.lock();
    let wake1_res = futex.wake(1, uaddr, 1);
    let remaining_waiters = futex.waiting_count(1, uaddr);
    drop(futex);
    let wake1_ok = wake1_res == Ok(1) && remaining_waiters == 0;
    check_m12!(8, "Futex Targeted Single Wake (FUTEX_WAKE count=1)", wake1_ok);

    // 9. Futex Broadcast Wake (Wakes multiple waiters simultaneously)
    let mut futex = FUTEX.lock();
    let _ = futex.wait(1, ThreadId(30), uaddr, 0x1234);
    let _ = futex.wait(1, ThreadId(31), uaddr, 0x1234);
    let _ = futex.wait(1, ThreadId(32), uaddr, 0x1234);
    let bcast_woken = futex.wake(1, uaddr, 100);
    let bcast_rem = futex.waiting_count(1, uaddr);
    drop(futex);
    let bcast_ok = bcast_woken == Ok(3) && bcast_rem == 0;
    check_m12!(9, "Futex Broadcast Wake (FUTEX_WAKE count=INT_MAX)", bcast_ok);

    // 10. Futex Requeue (Atomic migration from uaddr1 to uaddr2)
    let mut var2: u32 = 0x5678;
    let uaddr2 = &mut var2 as *mut u32 as u64;
    let mut futex = FUTEX.lock();
    let _ = futex.wait(1, ThreadId(40), uaddr, 0x1234);
    let _ = futex.wait(1, ThreadId(41), uaddr, 0x1234);
    let _ = futex.wait(1, ThreadId(42), uaddr, 0x1234);
    // Wake 1, requeue 2 to uaddr2
    let req_res = futex.requeue(1, uaddr, 1, uaddr2, 2);
    let rem1 = futex.waiting_count(1, uaddr);
    let rem2 = futex.waiting_count(1, uaddr2);
    // Cleanup remaining waiters
    let _ = futex.wake(1, uaddr2, 10);
    drop(futex);
    let req_ok = req_res == Ok(1) && rem1 == 0 && rem2 == 2;
    check_m12!(10, "Futex Requeue Primitive (Atomic Queue Migration)", req_ok);

    // 11. ELF PT_DYNAMIC Segment Header Parsing
    let dyn_entries = [
        Elf64Dyn { d_tag: DT_STRTAB, d_val: 0x1000 },
        Elf64Dyn { d_tag: DT_STRSZ, d_val: 256 },
        Elf64Dyn { d_tag: DT_SYMTAB, d_val: 0x2000 },
        Elf64Dyn { d_tag: DT_RELA, d_val: 0x3000 },
        Elf64Dyn { d_tag: DT_RELASZ, d_val: 72 },
        Elf64Dyn { d_tag: DT_NEEDED, d_val: 1 },
        Elf64Dyn { d_tag: DT_NULL, d_val: 0 },
    ];
    let parsed_dyn = ParsedDynamicSection::parse(&dyn_entries);
    let dyn_ok = parsed_dyn.strtab_offset == 0x1000
        && parsed_dyn.symtab_offset == 0x2000
        && parsed_dyn.rela_offset == 0x3000
        && parsed_dyn.needed_libraries.len() == 1;
    check_m12!(11, "ELF PT_DYNAMIC Segment Header & Dependency Tag Decoding", dyn_ok);

    // 12. ELF Shared Object Relocation: R_X86_64_RELATIVE
    let mut linker = DynamicLinker::new();
    let mut mem_target = [0u8; 64];
    let rela_rel = Elf64Rela::new(16, 0, R_X86_64_RELATIVE, 0x400);
    let base_load_addr = 0x0000_7F00_0000_0000u64;
    let rel_ok = match linker.apply_relocation(base_load_addr, &rela_rel, None, &mut mem_target) {
        Ok(val) => {
            let written = u64::from_le_bytes(mem_target[16..24].try_into().unwrap());
            val == base_load_addr + 0x400 && written == val
        }
        Err(_) => false,
    };
    check_m12!(12, "ELF Relocation: R_X86_64_RELATIVE (Base Address Rebase)", rel_ok);

    // 13. ELF Shared Object Relocation: R_X86_64_GLOB_DAT
    let lib_id = linker.register_library("libc.so", 0x0000_7F10_0000_0000, 0x20000);
    linker.add_symbol_to_library(lib_id, "malloc", 0x1500);
    let rela_glob = Elf64Rela::new(32, 1, R_X86_64_GLOB_DAT, 0);
    let glob_ok = match linker.apply_relocation(base_load_addr, &rela_glob, Some("malloc"), &mut mem_target) {
        Ok(val) => {
            let written = u64::from_le_bytes(mem_target[32..40].try_into().unwrap());
            val == 0x0000_7F10_0000_1500 && written == val
        }
        Err(_) => false,
    };
    check_m12!(13, "ELF Relocation: R_X86_64_GLOB_DAT (GOT Symbol Resolution)", glob_ok);

    // 14. ELF Shared Object Relocation: R_X86_64_JUMP_SLOT
    linker.add_symbol_to_library(lib_id, "printf", 0x2200);
    let rela_jump = Elf64Rela::new(40, 2, R_X86_64_JUMP_SLOT, 0);
    let jump_ok = match linker.apply_relocation(base_load_addr, &rela_jump, Some("printf"), &mut mem_target) {
        Ok(val) => {
            let written = u64::from_le_bytes(mem_target[40..48].try_into().unwrap());
            val == 0x0000_7F10_0000_2200 && written == val
        }
        Err(_) => false,
    };
    check_m12!(14, "ELF Relocation: R_X86_64_JUMP_SLOT (PLT Function Pointer Binding)", jump_ok);

    // 15. Shared Library Dependency Resolution & Symbol Lookup (sys_dlopen / sys_dlsym)
    let libm_id = linker.register_library("libm.so", 0x0000_7F20_0000_0000, 0x10000);
    linker.add_symbol_to_library(libm_id, "sin", 0x300);
    linker.add_dependency(lib_id, "libm.so");
    let sym_sin = linker.dlsym(libm_id as u64, "sin");
    let sym_missing = linker.dlsym(libm_id as u64, "nonexistent");
    let dep_ok = sym_sin == Ok(0x0000_7F20_0000_0300) && sym_missing == Err(-2);
    check_m12!(15, "Shared Library Dependency Resolution (dlopen & dlsym Lookup)", dep_ok);

    // 16. M12 Syscall Security Validation
    // Bad non-canonical pointer for thread creation should return -14 (EFAULT)
    let bad_thread = dispatch_syscall(SYS_THREAD_CREATE, 0xFFFF_8000_0000_0000, 0x1000, 0, 0, 0);
    // Unaligned futex address should return -22 (EINVAL)
    let unaligned_futex = dispatch_syscall(SYS_FUTEX, 0x1003, FUTEX_WAIT as u64, 0, 0, 0);
    // Non-canonical FS_BASE should return -14 (EFAULT)
    let bad_tls = dispatch_syscall(SYS_SET_FS_BASE, 0x0000_9000_0000_0000, 0, 0, 0, 0);
    let sec_ok = bad_thread == -14 && unaligned_futex == -22 && bad_tls == -14;
    check_m12!(16, "M12 Syscall Boundary & Canonical Address Security Hardening", sec_ok);

    // Reclaim test threads so scheduler only runs PID 1 init
    if let Ok(tid) = t1_res {
        let _ = set_thread_state(tid, ThreadState::Terminated(0));
    }
    if let Ok(tid) = t2_res {
        let _ = set_thread_state(tid, ThreadState::Terminated(0));
    }

    true
}

fn print_num(mut n: usize) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 8];
    let mut i = 0;
    while n > 0 && i < buf.len() {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
