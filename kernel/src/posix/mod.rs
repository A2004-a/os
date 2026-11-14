//! NovaOS Apex — POSIX Layer & musl libc ABI Compatibility Subsystem (M13)
//! Provides standard POSIX file, process, pipe, socket, thread, and time abstractions
//! along with an x86_64 Linux/musl system call translation table.

pub mod pipe;
pub mod process;
pub mod time;
pub mod m13_tests;

// Standard POSIX file descriptor constants
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;

// Standard POSIX lseek whence
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

// Standard Linux x86_64 System Call Numbers (musl ABI)
pub const LINUX_SYS_READ: u64 = 0;
pub const LINUX_SYS_WRITE: u64 = 1;
pub const LINUX_SYS_OPEN: u64 = 2;
pub const LINUX_SYS_CLOSE: u64 = 3;
pub const LINUX_SYS_STAT: u64 = 4;
pub const LINUX_SYS_FSTAT: u64 = 5;
pub const LINUX_SYS_LSEEK: u64 = 8;
pub const LINUX_SYS_PIPE: u64 = 22;
pub const LINUX_SYS_SCHED_YIELD: u64 = 24;
pub const LINUX_SYS_DUP: u64 = 32;
pub const LINUX_SYS_DUP2: u64 = 33;
pub const LINUX_SYS_NANOSLEEP: u64 = 35;
pub const LINUX_SYS_GETPID: u64 = 39;
pub const LINUX_SYS_SOCKET: u64 = 41;
pub const LINUX_SYS_CONNECT: u64 = 42;
pub const LINUX_SYS_ACCEPT: u64 = 43;
pub const LINUX_SYS_SENDTO: u64 = 44;
pub const LINUX_SYS_RECVFROM: u64 = 45;
pub const LINUX_SYS_BIND: u64 = 49;
pub const LINUX_SYS_LISTEN: u64 = 50;
pub const LINUX_SYS_CLONE: u64 = 56;
pub const LINUX_SYS_FORK: u64 = 57;
pub const LINUX_SYS_EXIT: u64 = 60;
pub const LINUX_SYS_WAIT4: u64 = 61;
pub const LINUX_SYS_GETUID: u64 = 102;
pub const LINUX_SYS_GETGID: u64 = 104;
pub const LINUX_SYS_GETEUID: u64 = 107;
pub const LINUX_SYS_GETEGID: u64 = 108;
pub const LINUX_SYS_GETPPID: u64 = 110;
pub const LINUX_SYS_FUTEX: u64 = 202;
pub const LINUX_SYS_CLOCK_GETTIME: u64 = 228;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PosixStat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_mtime: i64,
    pub st_ctime: i64,
}

pub fn translate_linux_syscall(nr: u64) -> Option<usize> {
    match nr {
        LINUX_SYS_READ => Some(crate::syscall::SYS_READ),
        LINUX_SYS_WRITE => Some(crate::syscall::SYS_WRITE),
        LINUX_SYS_OPEN => Some(crate::syscall::SYS_OPEN),
        LINUX_SYS_CLOSE => Some(crate::syscall::SYS_CLOSE),
        LINUX_SYS_STAT => Some(crate::syscall::SYS_STAT),
        LINUX_SYS_FSTAT => Some(crate::syscall::SYS_FSTAT),
        LINUX_SYS_LSEEK => Some(crate::syscall::SYS_LSEEK),
        LINUX_SYS_PIPE => Some(crate::syscall::SYS_PIPE),
        LINUX_SYS_SCHED_YIELD => Some(crate::syscall::SYS_YIELD),
        LINUX_SYS_DUP => Some(crate::syscall::SYS_DUP),
        LINUX_SYS_DUP2 => Some(crate::syscall::SYS_DUP2),
        LINUX_SYS_NANOSLEEP => Some(crate::syscall::SYS_NANOSLEEP),
        LINUX_SYS_GETPID => Some(crate::syscall::SYS_GETPID),
        LINUX_SYS_SOCKET => Some(crate::syscall::SYS_CONNECT), // BSD socket abstraction
        LINUX_SYS_CONNECT => Some(crate::syscall::SYS_CONNECT),
        LINUX_SYS_ACCEPT => Some(crate::syscall::SYS_ACCEPT),
        LINUX_SYS_SENDTO => Some(crate::syscall::SYS_SEND),
        LINUX_SYS_RECVFROM => Some(crate::syscall::SYS_RECV),
        LINUX_SYS_BIND => Some(crate::syscall::SYS_BIND),
        LINUX_SYS_LISTEN => Some(crate::syscall::SYS_LISTEN),
        LINUX_SYS_CLONE => Some(crate::syscall::SYS_THREAD_CREATE),
        LINUX_SYS_FORK => Some(crate::syscall::SYS_FORK),
        LINUX_SYS_EXIT => Some(crate::syscall::SYS_EXIT),
        LINUX_SYS_WAIT4 => Some(crate::syscall::SYS_WAITPID),
        LINUX_SYS_GETUID => Some(crate::syscall::SYS_GETUID),
        LINUX_SYS_GETEUID => Some(crate::syscall::SYS_GETEUID),
        LINUX_SYS_FUTEX => Some(crate::syscall::SYS_FUTEX),
        LINUX_SYS_CLOCK_GETTIME => Some(crate::syscall::SYS_CLOCK_GETTIME),
        _ => None,
    }
}
