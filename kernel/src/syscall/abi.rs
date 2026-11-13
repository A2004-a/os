//! NovaOS System Call ABI Definitions

pub const SYS_EXIT: usize = 1;
pub const SYS_READ: usize = 2;
pub const SYS_WRITE: usize = 3;
pub const SYS_OPEN: usize = 4;
pub const SYS_CLOSE: usize = 5;
pub const SYS_YIELD: usize = 6;
pub const SYS_SLEEP: usize = 7;
pub const SYS_SPAWN: usize = 8;
pub const SYS_WAIT: usize = 9;
pub const SYS_MAP_MEMORY: usize = 10;
pub const SYS_UNMAP_MEMORY: usize = 11;
pub const SYS_GET_TIME: usize = 12;
