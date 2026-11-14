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
pub const SYS_STAT: usize = 15;
pub const SYS_READDIR: usize = 16;
pub const SYS_MKDIR: usize = 17;
pub const SYS_UNLINK: usize = 18;
pub const SYS_CHDIR: usize = 19;
pub const SYS_GETCWD: usize = 20;
pub const SYS_SEEK: usize = 21;
pub const SYS_SOCKET: usize = 22;
pub const SYS_BIND: usize = 23;
pub const SYS_LISTEN: usize = 24;
pub const SYS_ACCEPT: usize = 25;
pub const SYS_CONNECT: usize = 26;
pub const SYS_SEND: usize = 27;
pub const SYS_RECV: usize = 28;
pub const SYS_CREATE_WINDOW: usize = 29;
pub const SYS_DESTROY_WINDOW: usize = 30;
pub const SYS_BLIT_WINDOW: usize = 31;
pub const SYS_POLL_EVENT: usize = 32;
pub const SYS_GET_DISPLAY_INFO: usize = 33;
pub const SYS_SET_CURSOR: usize = 34;

pub const WINDOW_FLAG_FRAMELESS: u32 = 1 << 0;
pub const WINDOW_FLAG_TRANSPARENT: u32 = 1 << 1;
pub const WINDOW_FLAG_MODAL: u32 = 1 << 2;
pub const WINDOW_FLAG_RESIZABLE: u32 = 1 << 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserEvent {
    pub event_type: u32,
    pub arg1: i32,
    pub arg2: i32,
    pub arg3: u32,
    pub arg4: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub bpp: u32,
}

pub const AF_INET: u32 = 2;
pub const SOCK_STREAM: u32 = 1;
pub const SOCK_DGRAM: u32 = 2;
pub const SOCK_RAW: u32 = 3;

pub const O_RDONLY: u32 = 0x0001;
pub const O_WRONLY: u32 = 0x0002;
pub const O_RDWR: u32 = 0x0003;
pub const O_CREAT: u32 = 0x0100;
pub const O_TRUNC: u32 = 0x0200;
pub const O_APPEND: u32 = 0x0400;

pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileStat {
    pub inode: u64,
    pub size: u64,
    pub blocks: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub file_type: u32,
    pub nlink: u32,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
}

impl FileStat {
    pub const fn empty() -> Self {
        Self {
            inode: 0,
            size: 0,
            blocks: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            file_type: 0,
            nlink: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UserDirEntry {
    pub inode: u64,
    pub file_type: u32,
    pub name_len: u32,
    pub name: [u8; 128],
}

impl UserDirEntry {
    pub const fn empty() -> Self {
        Self {
            inode: 0,
            file_type: 0,
            name_len: 0,
            name: [0u8; 128],
        }
    }

    pub fn name_str(&self) -> &str {
        let len = (self.name_len as usize).min(128);
        core::str::from_utf8(&self.name[..len]).unwrap_or("")
    }
}

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

#[inline(always)]
unsafe fn syscall4(num: usize, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i64 {
    let ret: i64;
    asm!(
        "int 0x80",
        inout("rax") num as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
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

// 15. open_with_mode
pub fn open_with_mode(path: &str, flags: u32, mode: u32) -> Result<u32, i32> {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall3(SYS_OPEN, buf.as_ptr() as u64, flags as u64, mode as u64)
    };
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(ret as i32)
    }
}

// 16. stat
pub fn stat(path: &str, stat_buf: &mut FileStat) -> Result<(), i32> {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall2(SYS_STAT, buf.as_ptr() as u64, stat_buf as *mut FileStat as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 17. readdir
pub fn readdir(path: &str, offset: usize, entry: &mut UserDirEntry) -> Result<bool, i32> {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall3(SYS_READDIR, buf.as_ptr() as u64, offset as u64, entry as *mut UserDirEntry as u64)
    };
    if ret > 0 {
        Ok(true) // Entry found
    } else if ret == 0 {
        Ok(false) // End of directory
    } else {
        Err(ret as i32)
    }
}

// 18. mkdir
pub fn mkdir(path: &str, mode: u32) -> Result<(), i32> {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall2(SYS_MKDIR, buf.as_ptr() as u64, mode as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 19. unlink
pub fn unlink(path: &str) -> Result<(), i32> {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall1(SYS_UNLINK, buf.as_ptr() as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 20. chdir
pub fn chdir(path: &str) -> Result<(), i32> {
    let mut buf = [0u8; 256];
    let len = path.len().min(255);
    buf[..len].copy_from_slice(&path.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall1(SYS_CHDIR, buf.as_ptr() as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 21. getcwd
pub fn getcwd(buf: &mut [u8]) -> Result<usize, i32> {
    let ret = unsafe {
        syscall2(SYS_GETCWD, buf.as_mut_ptr() as u64, buf.len() as u64)
    };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret as i32)
    }
}

// 22. seek
pub fn seek(handle: u32, offset: i64, whence: u32) -> Result<u64, i32> {
    let ret = unsafe {
        syscall3(SYS_SEEK, handle as u64, offset as u64, whence as u64)
    };
    if ret >= 0 {
        Ok(ret as u64)
    } else {
        Err(ret as i32)
    }
}

// 23. socket
pub fn socket(domain: u32, sock_type: u32, protocol: u32) -> Result<u32, i32> {
    let ret = unsafe {
        syscall3(SYS_SOCKET, domain as u64, sock_type as u64, protocol as u64)
    };
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(ret as i32)
    }
}

// 24. bind
pub fn bind(handle: u32, ip: [u8; 4], port: u16) -> Result<(), i32> {
    let packed_ip = ((ip[0] as u32) << 24) | ((ip[1] as u32) << 16) | ((ip[2] as u32) << 8) | (ip[3] as u32);
    let ret = unsafe {
        syscall3(SYS_BIND, handle as u64, packed_ip as u64, port as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 25. listen
pub fn listen(handle: u32, backlog: usize) -> Result<(), i32> {
    let ret = unsafe {
        syscall2(SYS_LISTEN, handle as u64, backlog as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 26. accept
pub fn accept(handle: u32) -> Result<u32, i32> {
    let ret = unsafe {
        syscall1(SYS_ACCEPT, handle as u64)
    };
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(ret as i32)
    }
}

// 27. connect
pub fn connect(handle: u32, ip: [u8; 4], port: u16) -> Result<(), i32> {
    let packed_ip = ((ip[0] as u32) << 24) | ((ip[1] as u32) << 16) | ((ip[2] as u32) << 8) | (ip[3] as u32);
    let ret = unsafe {
        syscall3(SYS_CONNECT, handle as u64, packed_ip as u64, port as u64)
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 28. send
pub fn send(handle: u32, data: &[u8]) -> Result<usize, i32> {
    let ret = unsafe {
        syscall3(SYS_SEND, handle as u64, data.as_ptr() as u64, data.len() as u64)
    };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret as i32)
    }
}

// 29. recv
pub fn recv(handle: u32, buf: &mut [u8]) -> Result<usize, i32> {
    let ret = unsafe {
        syscall3(SYS_RECV, handle as u64, buf.as_mut_ptr() as u64, buf.len() as u64)
    };
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(ret as i32)
    }
}

// 30. create_window
pub fn create_window(title: &str, width: u32, height: u32, flags: u32) -> Result<u32, i32> {
    let mut buf = [0u8; 64];
    let len = title.len().min(63);
    buf[..len].copy_from_slice(&title.as_bytes()[..len]);
    buf[len] = 0;

    let ret = unsafe {
        syscall4(
            SYS_CREATE_WINDOW,
            buf.as_ptr() as u64,
            width as u64,
            height as u64,
            flags as u64,
        )
    };
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(ret as i32)
    }
}

// 31. destroy_window
pub fn destroy_window(window_id: u32) -> Result<(), i32> {
    let ret = unsafe { syscall1(SYS_DESTROY_WINDOW, window_id as u64) };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 32. blit_window
pub fn blit_window(window_id: u32, pixels: &[u32]) -> Result<(), i32> {
    let ret = unsafe {
        syscall3(
            SYS_BLIT_WINDOW,
            window_id as u64,
            pixels.as_ptr() as u64,
            (pixels.len() * 4) as u64,
        )
    };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}

// 33. poll_event
pub fn poll_event(window_id: u32) -> Option<UserEvent> {
    let mut ev = UserEvent {
        event_type: 0,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        arg4: 0,
    };
    let ret = unsafe {
        syscall2(
            SYS_POLL_EVENT,
            window_id as u64,
            &mut ev as *mut UserEvent as u64,
        )
    };
    if ret == 1 {
        Some(ev)
    } else {
        None
    }
}

// 34. get_display_info
pub fn get_display_info() -> Result<DisplayInfo, i32> {
    let mut info = DisplayInfo {
        width: 0,
        height: 0,
        stride: 0,
        bpp: 0,
    };
    let ret = unsafe {
        syscall1(
            SYS_GET_DISPLAY_INFO,
            &mut info as *mut DisplayInfo as u64,
        )
    };
    if ret >= 0 {
        Ok(info)
    } else {
        Err(ret as i32)
    }
}

// 35. set_cursor
pub fn set_cursor(x: i32, y: i32) -> Result<(), i32> {
    let ret = unsafe { syscall2(SYS_SET_CURSOR, x as u64, y as u64) };
    if ret >= 0 {
        Ok(())
    } else {
        Err(ret as i32)
    }
}
