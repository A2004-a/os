//! NovaOS Apex — Preemptive Multitasking Scheduler & Context Switch Engine
//! Implements round-robin/priority scheduling, sleep queues, per-thread stacks, and scheduler statistics.

pub mod sync;
pub mod thread;

use crate::arch::x86_64::exceptions::ExceptionContext;
use crate::arch::x86_64::gdt::set_tss_rsp0;
use crate::arch::x86_64::instructions::{read_cr3, write_cr3};
use crate::memory::stack::KernelStack;
use crate::serial;
use core::ptr::addr_of_mut;
use thread::{CpuContext, Thread, ThreadId, ThreadPriority, ThreadState};

pub const MAX_THREADS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerError {
    TableFull,
    ThreadNotFound,
    StackAllocationFailed,
    InvalidPriority,
    AlreadyTerminated,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SchedulerStats {
    pub total_switches: u64,
    pub idle_ticks: u64,
    pub user_ticks: u64,
    pub kernel_ticks: u64,
}

pub struct Scheduler {
    threads: [Option<Thread>; MAX_THREADS],
    current_idx: usize,
    next_tid: u32,
    stats: SchedulerStats,
    pub active: bool,
    kernel_cr3: u64,
}

static mut SCHEDULER: Scheduler = Scheduler {
    threads: [const { None }; MAX_THREADS],
    current_idx: 0,
    next_tid: 1,
    stats: SchedulerStats {
        total_switches: 0,
        idle_ticks: 0,
        user_ticks: 0,
        kernel_ticks: 0,
    },
    active: false,
    kernel_cr3: 0,
};

extern "C" fn idle_thread_entry() {
    loop {
        crate::arch::x86_64::instructions::hlt();
    }
}

pub fn init() {
    unsafe {
        let sched = &mut *addr_of_mut!(SCHEDULER);
        sched.kernel_cr3 = read_cr3();

        // 1. Allocate dedicated 64 KiB kernel stack for Idle thread
        let stack = KernelStack::allocate(16).expect("Failed to allocate idle thread stack");
        let idle_thread = Thread::new_kernel(
            ThreadId::IDLE,
            "idle",
            idle_thread_entry,
            stack.top.as_u64(),
            ThreadPriority::Idle,
        );

        sched.threads[0] = Some(idle_thread);
        sched.current_idx = 0;
        sched.next_tid = 1;
        sched.active = true;
    }

    serial::write_str("[SCHEDULER] Multi-priority preemptive scheduler initialized.\n");
    serial::write_str("  Max concurrent threads: 64, Idle thread TID 0 ready.\n");
}

pub fn spawn_kernel_thread(
    name: &str,
    entry: extern "C" fn(),
    priority: ThreadPriority,
) -> Result<ThreadId, SchedulerError> {
    unsafe {
        let sched = &mut *addr_of_mut!(SCHEDULER);

        let free_idx = sched.threads.iter().position(|t| t.is_none())
            .ok_or(SchedulerError::TableFull)?;

        let stack = KernelStack::allocate(16)
            .map_err(|_| SchedulerError::StackAllocationFailed)?;

        let tid = ThreadId(sched.next_tid);
        sched.next_tid += 1;

        let thread = Thread::new_kernel(
            tid,
            name,
            entry,
            stack.top.as_u64(),
            priority,
        );

        sched.threads[free_idx] = Some(thread);
        Ok(tid)
    }
}

pub fn spawn_user_thread(
    name: &str,
    pid: u32,
    entry: u64,
    user_stack_top: u64,
    cr3: u64,
    priority: ThreadPriority,
) -> Result<ThreadId, SchedulerError> {
    unsafe {
        let sched = &mut *addr_of_mut!(SCHEDULER);

        let free_idx = sched.threads.iter().position(|t| t.is_none())
            .ok_or(SchedulerError::TableFull)?;

        // Kernel stack for Ring 0 interrupts while this user thread runs (64 KiB)
        let kstack = KernelStack::allocate(16)
            .map_err(|_| SchedulerError::StackAllocationFailed)?;

        let tid = ThreadId(sched.next_tid);
        sched.next_tid += 1;

        let thread = Thread::new_user(
            tid,
            pid,
            name,
            entry,
            user_stack_top,
            kstack.top.as_u64(),
            cr3,
            priority,
        );

        sched.threads[free_idx] = Some(thread);
        Ok(tid)
    }
}

pub fn on_timer_tick(ctx: &mut ExceptionContext) {
    let sched = unsafe { &mut *addr_of_mut!(SCHEDULER) };
    if !sched.active {
        return;
    }

    // 1. Process sleep timers
    for slot in sched.threads.iter_mut() {
        if let Some(thread) = slot {
            if let ThreadState::Sleeping(rem) = thread.state {
                if rem <= 1 {
                    thread.state = ThreadState::Ready;
                } else {
                    thread.state = ThreadState::Sleeping(rem - 1);
                }
            }
        }
    }

    // 2. Account ticks for current thread
    let _current_is_user = if let Some(current) = sched.threads[sched.current_idx].as_mut() {
        current.ticks_run += 1;
        if current.is_user {
            sched.stats.user_ticks += 1;
            true
        } else if current.id == ThreadId::IDLE {
            sched.stats.idle_ticks += 1;
            false
        } else {
            sched.stats.kernel_ticks += 1;
            false
        }
    } else {
        false
    };

    // 3. Decrement quantum
    let mut need_reschedule = false;
    if let Some(current) = sched.threads[sched.current_idx].as_mut() {
        if current.state == ThreadState::Running {
            current.quantum_remaining = current.quantum_remaining.saturating_sub(1);
            if current.quantum_remaining == 0 {
                current.state = ThreadState::Ready;
                current.quantum_remaining = current.priority.default_quantum_ms();
                need_reschedule = true;
            }
        } else {
            need_reschedule = true;
        }
    } else {
        need_reschedule = true;
    }

    if need_reschedule {
        reschedule(ctx, false);
    }
}

pub fn yield_current(ctx: &mut ExceptionContext) {
    let sched = unsafe { &mut *addr_of_mut!(SCHEDULER) };
    if let Some(current) = sched.threads[sched.current_idx].as_mut() {
        if current.state == ThreadState::Running {
            current.state = ThreadState::Ready;
            current.quantum_remaining = current.priority.default_quantum_ms();
        }
    }
    reschedule(ctx, false);
}

pub fn sleep_current(ms: u64, ctx: &mut ExceptionContext) {
    let sched = unsafe { &mut *addr_of_mut!(SCHEDULER) };
    if let Some(current) = sched.threads[sched.current_idx].as_mut() {
        current.state = ThreadState::Sleeping(ms);
    }
    reschedule(ctx, false);
}

pub fn exit_current(exit_code: i32, ctx: &mut ExceptionContext) {
    let sched = unsafe { &mut *addr_of_mut!(SCHEDULER) };
    if let Some(current) = sched.threads[sched.current_idx].as_mut() {
        current.state = ThreadState::Terminated(exit_code);
        current.exit_code = Some(exit_code);
    }
    reschedule(ctx, true);
}

fn reschedule(ctx: &mut ExceptionContext, current_terminated: bool) {
    let sched = unsafe { &mut *addr_of_mut!(SCHEDULER) };
    let old_idx = sched.current_idx;

    // Save context of old thread if it wasn't terminated
    if !current_terminated {
        if let Some(old_thread) = sched.threads[old_idx].as_mut() {
            copy_exception_to_context(ctx, &mut old_thread.context);
        }
    }

    // Pick next thread to run: priority scan from Realtime down to Low, then Idle
    let priorities = [
        ThreadPriority::Realtime,
        ThreadPriority::High,
        ThreadPriority::Normal,
        ThreadPriority::Low,
    ];

    let mut next_idx: Option<usize> = None;

    'outer: for prio in priorities {
        // Round-robin within priority band starting after current_idx
        for i in 1..=MAX_THREADS {
            let idx = (old_idx + i) % MAX_THREADS;
            if idx == 0 { continue; } // Skip idle thread in normal scan
            if let Some(t) = &sched.threads[idx] {
                if t.priority == prio && t.state == ThreadState::Ready {
                    next_idx = Some(idx);
                    break 'outer;
                }
            }
        }
    }

    // Fall back to Idle thread if no user/worker thread is ready
    let target_idx = next_idx.unwrap_or(0);

    if target_idx == old_idx && !current_terminated {
        if let Some(curr) = sched.threads[old_idx].as_mut() {
            if curr.state == ThreadState::Ready {
                curr.state = ThreadState::Running;
            }
        }
        return;
    }

    // Perform context switch to target thread
    sched.current_idx = target_idx;
    sched.stats.total_switches += 1;

    let target = sched.threads[target_idx].as_mut().expect("Target thread must exist");
    target.state = ThreadState::Running;
    target.quantum_remaining = target.priority.default_quantum_ms();

    // 1. Update TSS.rsp0 so Ring 3 interrupts switch to this thread's kernel stack
    set_tss_rsp0(target.kernel_stack_top);

    // 2. Switch address space (CR3) if needed
    let target_cr3 = if target.cr3 != 0 { target.cr3 } else { sched.kernel_cr3 };
    let current_cr3 = read_cr3();
    if target_cr3 != current_cr3 {
        unsafe {
            write_cr3(target_cr3);
        }
    }

    // 3. Load target thread context into exception context
    copy_context_to_exception(&target.context, ctx);
}

pub fn get_current_tid() -> ThreadId {
    let sched = unsafe { &*addr_of_mut!(SCHEDULER) };
    sched.threads[sched.current_idx].as_ref().map(|t| t.id).unwrap_or(ThreadId::IDLE)
}

pub fn get_current_pid() -> u32 {
    let sched = unsafe { &*addr_of_mut!(SCHEDULER) };
    sched.threads[sched.current_idx].as_ref().map(|t| t.process_id).unwrap_or(0)
}

pub fn get_stats() -> SchedulerStats {
    let sched = unsafe { &*addr_of_mut!(SCHEDULER) };
    sched.stats
}

fn copy_exception_to_context(src: &ExceptionContext, dst: &mut CpuContext) {
    dst.r15 = src.r15;
    dst.r14 = src.r14;
    dst.r13 = src.r13;
    dst.r12 = src.r12;
    dst.r11 = src.r11;
    dst.r10 = src.r10;
    dst.r9  = src.r9;
    dst.r8  = src.r8;
    dst.rbp = src.rbp;
    dst.rdi = src.rdi;
    dst.rsi = src.rsi;
    dst.rdx = src.rdx;
    dst.rcx = src.rcx;
    dst.rbx = src.rbx;
    dst.rax = src.rax;
    dst.vector = src.vector;
    dst.error_code = src.error_code;
    dst.rip = src.rip;
    dst.cs = src.cs;
    dst.rflags = src.rflags;
    dst.rsp = src.rsp;
    dst.ss = src.ss;
}

fn copy_context_to_exception(src: &CpuContext, dst: &mut ExceptionContext) {
    dst.r15 = src.r15;
    dst.r14 = src.r14;
    dst.r13 = src.r13;
    dst.r12 = src.r12;
    dst.r11 = src.r11;
    dst.r10 = src.r10;
    dst.r9  = src.r9;
    dst.r8  = src.r8;
    dst.rbp = src.rbp;
    dst.rdi = src.rdi;
    dst.rsi = src.rsi;
    dst.rdx = src.rdx;
    dst.rcx = src.rcx;
    dst.rbx = src.rbx;
    dst.rax = src.rax;
    dst.vector = src.vector;
    dst.error_code = src.error_code;
    dst.rip = src.rip;
    dst.cs = src.cs;
    dst.rflags = src.rflags;
    dst.rsp = src.rsp;
    dst.ss = src.ss;
}
