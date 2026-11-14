//! NovaOS Zero — Multitasking Scheduler Subsystem

pub mod round_robin;
pub mod task;
pub mod scheduler_test;

pub use round_robin::{switch_context, PriorityScheduler, SCHEDULER, MAX_TASKS};
pub use task::{Task, TaskContext, TaskId, TaskState};

pub fn schedule_tick() {
    let current_ticks = crate::interrupts::timer::get_ticks();
    let mut sched = SCHEDULER.lock();
    sched.update_sleep_queues(current_ticks);
    let _next = sched.schedule_next();
}
