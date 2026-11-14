//! NovaOS Apex — POSIX Clocks & High-Resolution Timers (M13)
//! Implements timespec, clock_gettime (CLOCK_REALTIME, CLOCK_MONOTONIC), and nanosleep.

use crate::drivers::timer::get_ticks;

pub const CLOCK_REALTIME: u32 = 0;
pub const CLOCK_MONOTONIC: u32 = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: u32 = 2;
pub const CLOCK_THREAD_CPUTIME_ID: u32 = 3;
pub const CLOCK_MONOTONIC_RAW: u32 = 4;

// Base Epoch: 2026-11-14T00:00:00Z UNIX timestamp
pub const BASE_UNIX_EPOCH_SECS: i64 = 1794614400;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

impl Timespec {
    pub const fn new(sec: i64, nsec: i64) -> Self {
        Self { tv_sec: sec, tv_nsec: nsec }
    }

    pub fn to_millis(&self) -> u64 {
        ((self.tv_sec as u64) * 1000) + ((self.tv_nsec as u64) / 1_000_000)
    }

    pub fn from_millis(ms: u64) -> Self {
        Self {
            tv_sec: (ms / 1000) as i64,
            tv_nsec: ((ms % 1000) * 1_000_000) as i64,
        }
    }
}

pub fn clock_gettime(clk_id: u32) -> Result<Timespec, i32> {
    let uptime_ticks = get_ticks(); // 1 tick = 1 ms (1000 Hz PIT/APIC)
    let mono_sec = (uptime_ticks / 1000) as i64;
    let mono_nsec = ((uptime_ticks % 1000) * 1_000_000) as i64;

    match clk_id {
        CLOCK_MONOTONIC | CLOCK_MONOTONIC_RAW => Ok(Timespec::new(mono_sec, mono_nsec)),
        CLOCK_REALTIME => {
            let real_sec = BASE_UNIX_EPOCH_SECS + mono_sec;
            Ok(Timespec::new(real_sec, mono_nsec))
        }
        CLOCK_PROCESS_CPUTIME_ID | CLOCK_THREAD_CPUTIME_ID => {
            Ok(Timespec::new(mono_sec, mono_nsec))
        }
        _ => Err(-22), // EINVAL
    }
}

pub fn nanosleep(req: &Timespec, rem: Option<&mut Timespec>) -> Result<(), i32> {
    if req.tv_nsec < 0 || req.tv_nsec >= 1_000_000_000 || req.tv_sec < 0 {
        return Err(-22); // EINVAL
    }

    let sleep_ms = req.to_millis();
    if sleep_ms > 0 {
        crate::drivers::timer::sleep_ms(sleep_ms);
    }

    if let Some(r) = rem {
        r.tv_sec = 0;
        r.tv_nsec = 0;
    }
    Ok(())
}
