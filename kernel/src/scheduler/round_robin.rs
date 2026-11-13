//! Round-Robin Task Scheduler

use super::task::{Task, TaskId, TaskState};
use spin::Mutex;

pub const MAX_TASKS: usize = 32;

pub struct RoundRobinScheduler {
    tasks: [Option<Task>; MAX_TASKS],
    current_index: usize,
    task_count: usize,
}

impl RoundRobinScheduler {
    pub const fn new() -> Self {
        const INIT_TASK: Option<Task> = None;
        RoundRobinScheduler {
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

    pub fn schedule_next(&mut self) -> Option<TaskId> {
        if self.task_count == 0 {
            return None;
        }

        let start_index = self.current_index;
        loop {
            self.current_index = (self.current_index + 1) % MAX_TASKS;
            if let Some(ref mut task) = self.tasks[self.current_index] {
                if task.state == TaskState::Ready {
                    task.state = TaskState::Running;
                    return Some(task.id);
                }
            }
            if self.current_index == start_index {
                break;
            }
        }

        None
    }
}

pub static SCHEDULER: Mutex<RoundRobinScheduler> = Mutex::new(RoundRobinScheduler::new());
