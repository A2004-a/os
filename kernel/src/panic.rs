//! NovaOS Apex — Kernel Panic Subsystem

use crate::arch::x86_64::instructions::{hlt, read_rsp};
use crate::serial;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — KERNEL PANIC ENCOUNTERED               \n");
    serial::write_str("=======================================================\n");

    if let Some(msg) = info.message().as_str() {
        serial::write_str("Message: ");
        serial::write_str(msg);
        serial::write_str("\n");
    }

    if let Some(loc) = info.location() {
        serial::write_str("Location: ");
        serial::write_str(loc.file());
        serial::write_str(":");
        print_u64(loc.line() as u64);
        serial::write_str("\n");
    }

    let rsp = read_rsp();
    serial::write_str("Stack Pointer (RSP): 0x");
    print_hex_u64(rsp);
    serial::write_str("\n");

    serial::write_str("[HALT] CPU halted in panic state. Controlled halt.\n");

    hlt();
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
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
