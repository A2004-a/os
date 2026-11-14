//! NovaOS Apex — Core Utilities Entrypoint

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_apex_runtime::{exit, print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("NovaOS Apex Coreutils v0.1.0\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
