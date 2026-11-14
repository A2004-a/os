//! NovaOS Zero — PID 1 Userland Init Process
//! Runs in Ring 3 userspace with lowest privileges, interacting exclusively through system calls.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    novaos_runtime::print("\n=======================================================\n");
    novaos_runtime::print("  NovaOS Zero — Ring 3 Userland Init (PID 1) Running!\n");
    novaos_runtime::print("=======================================================\n");

    let _uptime = novaos_runtime::get_time();
    novaos_runtime::print("[INIT] System uptime query succeeded via int 0x80 syscall.\n");

    novaos_runtime::print("[INIT] Testing cooperative multitasking yield (sys_yield)...\n");
    novaos_runtime::yield_now();

    novaos_runtime::print("[INIT] PID 1 execution verified. Requesting clean shutdown.\n");
    novaos_runtime::exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    novaos_runtime::print("[INIT PANIC] Userland exception triggered!\n");
    novaos_runtime::exit(1);
}
