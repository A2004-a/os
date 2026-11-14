//! NovaOS Apex — Native Application Suite

#![no_std]
#![no_main]

extern crate novaos_apex_toolkit;

use core::panic::PanicInfo;
use novaos_apex_runtime::{exit, print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("[APPS] NovaOS Apex Native Applications Suite v0.1.0\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
