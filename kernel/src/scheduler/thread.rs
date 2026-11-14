//! NovaOS Apex — Kernel Thread Structures & Execution Contexts
//! Defines thread states, priorities, saved CPU registers, address spaces, and stack pointers.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct ThreadId(pub u32);

impl ThreadId {
    pub const IDLE: Self = Self(0);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Sleeping(u64), // Remaining sleep ticks in ms
    Terminated(i32), // Exit code
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum ThreadPriority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Realtime = 4,
}

impl ThreadPriority {
    pub fn default_quantum_ms(&self) -> u32 {
        match self {
            ThreadPriority::Idle => 5,
            ThreadPriority::Low => 10,
            ThreadPriority::Normal => 20,
            ThreadPriority::High => 40,
            ThreadPriority::Realtime => 100,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CpuContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub vector: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl CpuContext {
    pub const fn empty() -> Self {
        Self {
            r15: 0, r14: 0, r13: 0, r12: 0, r11: 0, r10: 0, r9: 0, r8: 0,
            rbp: 0, rdi: 0, rsi: 0, rdx: 0, rcx: 0, rbx: 0, rax: 0,
            vector: 0, error_code: 0, rip: 0, cs: 0x08, rflags: 0x202,
            rsp: 0, ss: 0x10,
        }
    }
}

pub struct Thread {
    pub id: ThreadId,
    pub process_id: u32,
    pub name: [u8; 24],
    pub state: ThreadState,
    pub priority: ThreadPriority,
    pub context: CpuContext,
    pub kernel_stack_top: u64,
    pub user_stack_top: u64,
    pub cr3: u64,
    pub is_user: bool,
    pub quantum_remaining: u32,
    pub ticks_run: u64,
    pub exit_code: Option<i32>,
}

impl Thread {
    pub fn new_kernel(
        id: ThreadId,
        name: &str,
        entry: extern "C" fn(),
        stack_top: u64,
        priority: ThreadPriority,
    ) -> Self {
        let mut context = CpuContext::empty();
        context.rip = entry as *const () as usize as u64;
        context.rsp = stack_top;
        context.cs = 0x08; // Kernel Code Selector
        context.ss = 0x10; // Kernel Data Selector
        context.rflags = 0x202; // Interrupts enabled (IF = 1)

        let mut name_buf = [0u8; 24];
        let bytes = name.as_bytes();
        let len = bytes.len().min(24);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            id,
            process_id: 0, // Kernel
            name: name_buf,
            state: ThreadState::Ready,
            priority,
            context,
            kernel_stack_top: stack_top,
            user_stack_top: 0,
            cr3: 0, // Uses kernel default CR3
            is_user: false,
            quantum_remaining: priority.default_quantum_ms(),
            ticks_run: 0,
            exit_code: None,
        }
    }

    pub fn new_user(
        id: ThreadId,
        process_id: u32,
        name: &str,
        entry: u64,
        user_stack: u64,
        kernel_stack: u64,
        cr3: u64,
        priority: ThreadPriority,
    ) -> Self {
        let mut context = CpuContext::empty();
        context.rip = entry;
        context.rsp = user_stack;
        context.cs = 0x20 | 3; // User Code Selector (0x23)
        context.ss = 0x18 | 3; // User Data Selector (0x1B)
        context.rflags = 0x202; // IF = 1

        let mut name_buf = [0u8; 24];
        let bytes = name.as_bytes();
        let len = bytes.len().min(24);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            id,
            process_id,
            name: name_buf,
            state: ThreadState::Ready,
            priority,
            context,
            kernel_stack_top: kernel_stack,
            user_stack_top: user_stack,
            cr3,
            is_user: true,
            quantum_remaining: priority.default_quantum_ms(),
            ticks_run: 0,
            exit_code: None,
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("<unknown>")
    }
}
