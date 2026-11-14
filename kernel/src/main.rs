//! NovaOS Apex — Bare-Metal Kernel Entrypoint (Milestone 0)

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_apex_kernel::serial;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel_entry();
}

#[no_mangle]
pub extern "C" fn kernel_entry() -> ! {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Bare-Metal Kernel Core (M0)           \n");
    serial::write_str("=======================================================\n");
    serial::write_str("[KERNEL] Apex kernel_entry reached successfully.\n");

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial::write_str("\n[KERNEL PANIC] Fatal kernel exception encountered.\n");
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
