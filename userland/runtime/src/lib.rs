//! NovaOS Userland Runtime Layer
//! Direct interface to kernel system calls via x86_64 `syscall` instruction

#![no_std]

use core::arch::asm;

pub const SYS_EXIT: usize = 1;
pub const SYS_READ: usize = 2;
pub const SYS_WRITE: usize = 3;
pub const SYS_YIELD: usize = 6;
pub const SYS_GET_TIME: usize = 12;

#[inline]
pub unsafe fn syscall1(num: usize, arg1: usize) -> isize {
    let ret: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline]
pub unsafe fn syscall3(num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as usize);
    }
    loop {}
}

pub fn print(s: &str) {
    unsafe {
        syscall3(SYS_WRITE, 1, s.as_ptr() as usize, s.len());
    }
}
