//! NovaOS Apex — 8254 Programmable Interval Timer (PIT) & APIC Driver (#4)
//! Provides 1000 Hz (1 ms) system clock tick, uptime tracking, and calibrated sleeps.

use super::{Driver, DriverError, DriverState};
use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::serial;

pub const PIT_CHANNEL_0: u16 = 0x40;
pub const PIT_COMMAND: u16 = 0x43;
pub const PIT_BASE_FREQUENCY: u32 = 1_193_182; // 1.193182 MHz
pub const TIMER_TARGET_HZ: u32 = 1000;          // 1 ms tick period

pub static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

pub struct PitTimer {
    frequency_hz: u32,
    has_apic: bool,
    apic_base: u64,
    state: DriverState,
}

static mut TIMER_DRIVER: PitTimer = PitTimer {
    frequency_hz: TIMER_TARGET_HZ,
    has_apic: false,
    apic_base: 0,
    state: DriverState::Uninitialized,
};

impl Driver for PitTimer {
    fn name(&self) -> &'static str {
        "8254 PIT / Local APIC Timer"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        // Probe CPUID for APIC feature (Leaf 1, EDX bit 9)
        let has_apic = probe_apic();
        self.has_apic = has_apic;

        if has_apic {
            self.apic_base = read_apic_base();
        }

        // 8254 PIT is always present on x86 platforms
        Ok(true)
    }

    fn init(&mut self) -> Result<(), DriverError> {
        self.probe()?;

        let divisor = (PIT_BASE_FREQUENCY / self.frequency_hz) as u16;

        unsafe {
            // Command: Channel 0, Access Mode Lobyle/Hibyte, Mode 3 (Square Wave Generator), Binary
            asm!("out dx, al", in("dx") PIT_COMMAND, in("al") 0x36u8, options(nomem, nostack, preserves_flags));
            // Send low byte
            asm!("out dx, al", in("dx") PIT_CHANNEL_0, in("al") (divisor & 0xFF) as u8, options(nomem, nostack, preserves_flags));
            // Send high byte
            asm!("out dx, al", in("dx") PIT_CHANNEL_0, in("al") ((divisor >> 8) & 0xFF) as u8, options(nomem, nostack, preserves_flags));
        }

        self.state = DriverState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Stopped;
        Ok(())
    }

    fn state(&self) -> DriverState {
        self.state
    }
}

fn probe_apic() -> bool {
    let edx: u32;
    unsafe {
        asm!(
            "push rbx",
            "mov eax, 1",
            "cpuid",
            "pop rbx",
            out("edx") edx,
            out("eax") _,
            out("ecx") _,
            options(nomem, preserves_flags)
        );
    }
    (edx & (1 << 9)) != 0
}

fn read_apic_base() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") 0x1Bu32, // IA32_APIC_BASE MSR
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64 & 0xFFFF_F000)
}

pub fn init() {
    let timer = unsafe { &mut *core::ptr::addr_of_mut!(TIMER_DRIVER) };
    if let Ok(_) = timer.init() {
        let _ = timer.start();
        serial::write_str("[DRIVER 4/10] 8254 PIT Timer active at 1000 Hz (1 ms tick). ");
        if timer.has_apic {
            serial::write_str("Local APIC detected at 0x");
            print_hex_u64(timer.apic_base);
            serial::write_str("\n");
        } else {
            serial::write_str("Legacy PIC/PIT mode.\n");
        }
    }
}

pub fn on_timer_tick() {
    SYSTEM_TICKS.fetch_add(1, Ordering::Relaxed);
    super::pic::send_eoi(0);
}

pub fn get_ticks() -> u64 {
    SYSTEM_TICKS.load(Ordering::Relaxed)
}

pub fn get_uptime_ms() -> u64 {
    get_ticks() // 1 tick = 1 ms at 1000 Hz
}

pub fn sleep_ms(ms: u64) {
    let start = get_ticks();
    while get_ticks().saturating_sub(start) < ms {
        core::hint::spin_loop();
    }
}

fn print_hex_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        buf[i] = hex_chars[(n & 0x0F) as usize];
        n >>= 4;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
