//! NovaOS System Call ABI Definitions & Constants

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

// Standard POSIX-compatible error numbers
pub const EPERM: isize  = -1;
pub const ENOENT: isize = -2;
pub const ESRCH: isize  = -3;
pub const EBADF: isize  = -9;
pub const ENOMEM: isize = -12;
pub const EACCES: isize = -13;
pub const EFAULT: isize = -14;
pub const EEXIST: isize = -17;
pub const EINVAL: isize = -22;
pub const ENOSYS: isize = -38;
pub const EOVERFLOW: isize = -75;
