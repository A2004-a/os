//! NovaOS Apex — Kernel Library

#![no_std]

pub mod serial {
    use core::arch::asm;
    pub const COM1: u16 = 0x3F8;

    pub fn write_byte(b: u8) {
        unsafe {
            let mut ready: u8;
            loop {
                asm!("in al, dx", in("dx") COM1 + 5, out("al") ready, options(nomem, nostack, preserves_flags));
                if (ready & 0x20) != 0 {
                    break;
                }
                asm!("pause", options(nomem, nostack));
            }
            asm!("out dx, al", in("dx") COM1, in("al") b, options(nomem, nostack, preserves_flags));
        }
    }

    pub fn write_str(s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                write_byte(b'\r');
            }
            write_byte(b);
        }
    }
}
