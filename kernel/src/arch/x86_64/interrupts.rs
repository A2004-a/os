//! x86_64 Interrupt Control and CPU Halt Routines

use core::arch::asm;

#[inline]
pub fn enable() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

#[inline]
pub fn disable() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack));
    }
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}
