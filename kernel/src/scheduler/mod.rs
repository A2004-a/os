pub mod round_robin;
pub mod task;

pub use round_robin::{SCHEDULER, MAX_TASKS};
pub use task::{Task, TaskId, TaskState};

pub fn schedule_tick() {
    let mut sched = SCHEDULER.lock();
    let _next = sched.schedule_next();
}
