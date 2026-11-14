//! NovaOS Zero — Freestanding Userland Runtime Layer
//! Direct interface to kernel system calls via x86_64 software interrupt gate (int 0x80)

#![no_std]

use core::arch::asm;

pub const SYS_EXIT: usize           = 1;
pub const SYS_READ: usize           = 2;
pub const SYS_WRITE: usize          = 3;
pub const SYS_OPEN: usize           = 4;
pub const SYS_CLOSE: usize          = 5;
pub const SYS_SPAWN: usize          = 6;
pub const SYS_WAIT: usize           = 7;
pub const SYS_YIELD: usize          = 8;
pub const SYS_SLEEP: usize          = 9;
pub const SYS_GET_TIME: usize       = 10;
pub const SYS_MAP_MEMORY: usize     = 11;
pub const SYS_UNMAP_MEMORY: usize   = 12;
pub const SYS_SEND_IPC: usize       = 13;
pub const SYS_RECEIVE_IPC: usize    = 14;

#[inline]
pub unsafe fn syscall0(num: usize) -> isize {
    let ret: isize;
    asm!(
        "int 0x80",
        in("rax") num,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline]
pub unsafe fn syscall1(num: usize, arg1: usize) -> isize {
    let ret: isize;
    asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg1,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline]
pub unsafe fn syscall2(num: usize, arg1: usize, arg2: usize) -> isize {
    let ret: isize;
    asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline]
pub unsafe fn syscall3(num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: isize;
    asm!(
        "int 0x80",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as usize);
    }
    loop {
        unsafe { asm!("pause", options(nomem, nostack)); }
    }
}

pub fn print(s: &str) {
    unsafe {
        syscall3(SYS_WRITE, 1, s.as_ptr() as usize, s.len());
    }
}

pub fn get_time() -> u64 {
    unsafe {
        let res = syscall0(SYS_GET_TIME);
        if res >= 0 { res as u64 } else { 0 }
    }
}

pub fn yield_now() {
    unsafe {
        syscall0(SYS_YIELD);
    }
}

pub fn sleep_ms(ms: u64) {
    unsafe {
        syscall1(SYS_SLEEP, ms as usize);
    }
}
