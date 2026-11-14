//! NovaOS Zero — Priority Round-Robin Scheduler with Sleep/Wake Queues

use super::task::{Task, TaskContext, TaskId, TaskState};
use crate::sync::Spinlock;

pub const MAX_TASKS: usize = 32;

pub struct PriorityScheduler {
    tasks: [Option<Task>; MAX_TASKS],
    current_index: usize,
    task_count: usize,
}

impl PriorityScheduler {
    pub const fn new() -> Self {
        const INIT_TASK: Option<Task> = None;
        PriorityScheduler {
            tasks: [INIT_TASK; MAX_TASKS],
            current_index: 0,
            task_count: 0,
        }
    }

    pub fn spawn(&mut self, task: Task) -> Result<TaskId, &'static str> {
        for slot in self.tasks.iter_mut() {
            if slot.is_none() {
                let id = task.id;
                *slot = Some(task);
                self.task_count += 1;
                return Ok(id);
            }
        }
        Err("Task queue full")
    }

    #[inline]
    pub fn add_task(&mut self, task: Task) -> Result<TaskId, &'static str> {
        self.spawn(task)
    }

    /// Wake up any sleeping tasks whose target tick has arrived
    pub fn update_sleep_queues(&mut self, current_tick: u64) {
        for slot in self.tasks.iter_mut().flatten() {
            if let TaskState::Sleeping(target) = slot.state {
                if current_tick >= target {
                    slot.state = TaskState::Ready;
                }
            }
        }
    }

    /// Select next ready task considering priority
    pub fn schedule_next(&mut self) -> Option<TaskId> {
        if self.task_count == 0 {
            return None;
        }

        let start_index = self.current_index;
        let mut best_index = None;
        let mut highest_priority = 0;

        for offset in 1..=MAX_TASKS {
            let idx = (start_index + offset) % MAX_TASKS;
            if let Some(ref task) = self.tasks[idx] {
                if task.state == TaskState::Ready && task.priority >= highest_priority {
                    highest_priority = task.priority;
                    best_index = Some(idx);
                }
            }
        }

        if let Some(idx) = best_index {
            self.current_index = idx;
            let task = self.tasks[idx].as_mut().unwrap();
            task.state = TaskState::Running;
            task.time_slices += 1;
            Some(task.id)
        } else {
            None
        }
    }

    /// Put currently running task to sleep until specified tick count
    pub fn sleep_current(&mut self, wake_tick: u64) {
        if let Some(ref mut task) = self.tasks[self.current_index] {
            task.state = TaskState::Sleeping(wake_tick);
        }
    }

    /// Terminate a task by ID and reclaim its slot
    pub fn terminate(&mut self, id: TaskId) {
        for slot in self.tasks.iter_mut() {
            if let Some(ref task) = slot {
                if task.id == id {
                    *slot = None;
                    self.task_count -= 1;
                    break;
                }
            }
        }
    }

    pub fn active_task_count(&self) -> usize {
        self.task_count
    }
}

pub static SCHEDULER: Spinlock<PriorityScheduler> = Spinlock::new(PriorityScheduler::new());

/// Assembly context switch between two task contexts
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(
    _old_context: *mut TaskContext,
    _new_context: *const TaskContext,
) {
    core::arch::naked_asm!(
        // Save callee-preserved registers into old_context (pointed by rdi)
        "mov [rdi + 0x00], r15",
        "mov [rdi + 0x08], r14",
        "mov [rdi + 0x10], r13",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], rbx",
        "mov [rdi + 0x28], rbp",
        // Save return address from stack into old_context.rip
        "mov rax, [rsp]",
        "mov [rdi + 0x30], rax",

        // Load callee-preserved registers from new_context (pointed by rsi)
        "mov r15, [rsi + 0x00]",
        "mov r14, [rsi + 0x08]",
        "mov r13, [rsi + 0x10]",
        "mov r12, [rsi + 0x18]",
        "mov rbx, [rsi + 0x20]",
        "mov rbp, [rsi + 0x28]",
        // Jump to new task's saved RIP
        "mov rax, [rsi + 0x30]",
        "mov [rsp], rax",
        "ret",
    );
}
