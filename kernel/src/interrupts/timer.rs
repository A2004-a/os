//! Programmable Interval Timer (PIT) Tick Handling & Scheduler Preemption

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn on_timer_tick() {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed);

    // Yield every 10 ticks for cooperative/preemptive time slicing
    if tick % 10 == 0 {
        crate::scheduler::schedule_tick();
    }

    crate::interrupts::pic::notify_eoi(crate::interrupts::pic::PIC_1_OFFSET);
}

pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
