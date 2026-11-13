//! Task Control Block (TCB) & Execution State

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub u64);

pub struct Task {
    pub id: TaskId,
    pub stack_pointer: u64,
    pub state: TaskState,
    pub priority: u8,
}

impl Task {
    pub fn new(id: u64, stack_pointer: u64, priority: u8) -> Self {
        Task {
            id: TaskId(id),
            stack_pointer,
            state: TaskState::Ready,
            priority,
        }
    }
}
