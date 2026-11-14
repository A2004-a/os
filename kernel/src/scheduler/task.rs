//! NovaOS Zero — Task Control Block (TCB) & Context Definition

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Sleeping(u64), // Wakeup tick target
    Blocked,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub u64);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaskContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rip: u64,
}

impl TaskContext {
    pub const fn new(rip: u64) -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip,
        }
    }
}

pub struct Task {
    pub id: TaskId,
    pub process_id: u64,
    pub context: TaskContext,
    pub stack_bottom: u64,
    pub stack_top: u64,
    pub state: TaskState,
    pub priority: u8,
    pub time_slices: u64,
}

impl Task {
    pub fn new(id: u64, process_id: u64, entry: fn(), stack_base: u64, stack_size: usize, priority: u8) -> Self {
        let stack_top = stack_base + stack_size as u64;

        let mut context = TaskContext::default();
        context.rip = entry as usize as u64;

        Task {
            id: TaskId(id),
            process_id,
            context,
            stack_bottom: stack_base,
            stack_top,
            state: TaskState::Ready,
            priority,
            time_slices: 0,
        }
    }
}
