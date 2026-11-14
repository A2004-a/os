//! NovaOS Apex — Command-Line Shell

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_apex_runtime::{exit, print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("NovaOS Apex Shell v0.1.0\nType 'help' for commands.\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
