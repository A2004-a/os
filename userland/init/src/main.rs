//! NovaOS Apex — PID 1 Init Service

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_apex_runtime::{exit, print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("[INIT] NovaOS Apex PID 1 Init Online.\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
