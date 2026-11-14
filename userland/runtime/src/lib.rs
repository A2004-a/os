//! NovaOS Apex — Userland Runtime Layer

#![no_std]

use core::arch::asm;

pub fn exit(code: i32) -> ! {
    unsafe {
        asm!("int 0x80", in("rax") 1usize, in("rdi") code as usize, options(noreturn));
    }
}

pub fn print(s: &str) {
    unsafe {
        asm!("int 0x80", in("rax") 3usize, in("rdi") 1usize, in("rsi") s.as_ptr() as usize, in("rdx") s.len());
    }
}
