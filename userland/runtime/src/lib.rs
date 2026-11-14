//! NovaOS Apex — Userland Runtime Library & System Call Interface
//! Provides ergonomic, type-safe Rust abstractions over the 14 NovaOS Apex system calls (int 0x80).

#![no_std]

use core::arch::asm;

pub const SYS_EXIT: usize = 1;
pub const SYS_READ: usize = 2;
pub const SYS_WRITE: usize = 3;
pub const SYS_OPEN: usize = 4;
pub const SYS_CLOSE: usize = 5;
pub const SYS_SPAWN: usize = 6;
pub const SYS_WAIT: usize = 7;
pub const SYS_YIELD: usize = 8;
pub const SYS_SLEEP: usize = 9;
pub const SYS_GET_TIME: usize = 10;
pub const SYS_MAP_MEMORY: usize = 11;
pub const SYS_UNMAP_MEMORY: usize = 12;
pub const SYS_SEND_IPC: usize = 13;
pub const SYS_RECEIVE_IPC: usize = 14;

#[inline(always)]
unsafe fn syscall0(num: usize) -> i64 {
    let ret: i64;
    asm!(
        "int 0x80",
        inout("rax") num as u64 => ret,
        out("rcx") _,
        out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
unsafe fn syscall1(num: usize, arg1: u64) -> i64 {
    let ret: i64;
    asm!(
        "int 0x80",
        inout("rax") num as u64 => ret,
        in("rdi") arg1,
        out("rcx") _,
        out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
unsafe fn syscall2(num: usize, arg1: u64, arg2: u64) -> i64 {
    let ret: i64;
    asm!(
        "int 0x80",
        inout("rax") num as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        out("rcx") _,
        out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
unsafe fn syscall3(num: usize, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let ret: i64;
    asm!(
        "int 0x80",
        inout("rax") num as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rcx") _,
        out("r11") _,
        options(nostack, preserves_flags)
    );
    ret
}

// 1. exit
pub fn exit(code: i32) -> ! {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") SYS_EXIT as u64,
            in("rdi") code as u64,
            options(noreturn)
        );
    }
}

// 2. read
pub fn read(handle: u32, buf: &mut [u8]) -> Result<usize, i32> {
    let ret = unsafe {
        syscall3(SYS_READ, handle as u64, buf.as_mut_ptr() as u64, buf.len() as u64)
    };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret as i32)
    }
}

// 3. write
pub fn write(handle: u32, buf: &[u8]) -> Result<usize, i32> {
    let ret = unsafe {
        syscall3(SYS_WRITE, handle as u64, buf.as_ptr() as u64, buf.len() as u64)
    };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret as i32)
    }
}

pub fn print(s: &str) {
    let _ = write(1, s.as_bytes());
}

// 4. open
pub fn open(path: &str, flags: u32) -> Result<u32, i32> {
    // Null-terminated buffer
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall2(SYS_OPEN, buf.as_ptr() as u64, flags as u64)
    };
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(ret as i32)
    }
}

// 5. close
pub fn close(handle: u32) -> Result<(), i32> {
    let ret = unsafe { syscall1(SYS_CLOSE, handle as u64) };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 6. spawn
pub fn spawn(elf_bytes: &[u8], name: &str) -> Result<u32, i32> {
    let mut name_buf = [0u8; 64];
    let nlen = name.len().min(63);
    name_buf[..nlen].copy_from_slice(&name.as_bytes()[..nlen]);
    name_buf[nlen] = 0;

    let ret = unsafe {
        syscall3(
            SYS_SPAWN,
            elf_bytes.as_ptr() as u64,
            elf_bytes.len() as u64,
            name_buf.as_ptr() as u64,
        )
    };
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(ret as i32)
    }
}

// 7. wait
pub fn wait(pid: u32) -> Result<i32, i32> {
    let mut status: i32 = 0;
    let ret = unsafe {
        syscall2(SYS_WAIT, pid as u64, (&mut status as *mut i32) as u64)
    };
    if ret >= 0 {
        Ok(status)
    } else {
        Err(ret as i32)
    }
}

// 8. yield
pub fn yield_now() {
    unsafe {
        syscall0(SYS_YIELD);
    }
}

// 9. sleep
pub fn sleep(ms: u64) {
    unsafe {
        syscall1(SYS_SLEEP, ms);
    }
}

// 10. get_time
pub fn get_time() -> u64 {
    unsafe { syscall0(SYS_GET_TIME) as u64 }
}

// 11. map_memory
pub fn map_memory(virt: u64, size: usize, flags: u32) -> Result<u64, i32> {
    let ret = unsafe {
        syscall3(SYS_MAP_MEMORY, virt, size as u64, flags as u64)
    };
    if ret >= 0 {
        Ok(ret as u64)
    } else {
        Err(ret as i32)
    }
}

// 12. unmap_memory
pub fn unmap_memory(virt: u64, size: usize) -> Result<(), i32> {
    let ret = unsafe {
        syscall2(SYS_UNMAP_MEMORY, virt, size as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 13. send_ipc
pub fn send_ipc(dest_pid: u32, data: &[u8]) -> Result<(), i32> {
    let ret = unsafe {
        syscall3(SYS_SEND_IPC, dest_pid as u64, data.as_ptr() as u64, data.len() as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 14. receive_ipc
pub fn receive_ipc(src_pid: u32, buf: &mut [u8]) -> Result<usize, i32> {
    let ret = unsafe {
        syscall3(SYS_RECEIVE_IPC, src_pid as u64, buf.as_mut_ptr() as u64, buf.len() as u64)
    };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret as i32)
    }
}
