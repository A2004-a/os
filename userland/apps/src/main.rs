#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_runtime::{exit, print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("NovaOS Userland Application Suite\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
