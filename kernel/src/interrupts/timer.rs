//! NovaOS Zero — Programmable Interval Timer (PIT 8254) Driver
//! Generates 100 Hz hardware timer ticks on IRQ0 for timekeeping and preemptive scheduling

use core::sync::atomic::{AtomicU64, Ordering};
use crate::arch::x86_64::io::{io_wait, outb};

const PIT_CHANNEL0_DATA: u16 = 0x40;
const PIT_COMMAND_PORT: u16   = 0x43;
const PIT_FREQUENCY_HZ: u32   = 100;
const PIT_BASE_FREQUENCY: u32 = 1193182;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    unsafe {
        let divisor = (PIT_BASE_FREQUENCY / PIT_FREQUENCY_HZ) as u16;

        // Command: Channel 0, Access Mode lobyte/hibyte, Mode 3 (Square Wave Generator), Binary
        outb(PIT_COMMAND_PORT, 0x36);
        io_wait();

        // Send divisor low byte
        outb(PIT_CHANNEL0_DATA, (divisor & 0xFF) as u8);
        io_wait();

        // Send divisor high byte
        outb(PIT_CHANNEL0_DATA, ((divisor >> 8) & 0xFF) as u8);
        io_wait();
    }
}

pub fn on_timer_tick() {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed);

    // Yield every 10 ticks (100 ms) for time-sliced scheduling
    if tick % 10 == 0 {
        crate::scheduler::schedule_tick();
    }

    crate::interrupts::pic::notify_eoi(crate::interrupts::pic::PIC_1_OFFSET);
}

pub fn get_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn get_uptime_ms() -> u64 {
    // At 100 Hz, 1 tick = 10 ms
    get_ticks() * 10
}
