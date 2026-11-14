//! NovaOS Apex — VFS Data Structures and Error Types

extern crate alloc;
use alloc::string::String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    File = 1,
    Directory = 2,
    CharDevice = 3,
    BlockDevice = 4,
    Fifo = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FsError {
    NotFound = -2,          // ENOENT
    IoError = -5,           // EIO
    BadFileDescriptor = -9, // EBADF
    NoMemory = -12,         // ENOMEM
    PermissionDenied = -13, // EACCES
    BadAddress = -14,       // EFAULT
    Busy = -16,             // EBUSY
    AlreadyExists = -17,    // EEXIST
    NotADirectory = -20,    // ENOTDIR
    IsADirectory = -21,     // EISDIR
    InvalidArgument = -22,  // EINVAL
    TooManyOpenFiles = -24, // EMFILE
    NoSpace = -28,          // ENOSPC
    NotSupported = -38,     // ENOSYS
}

impl FsError {
    pub fn to_errno(self) -> i64 {
        self as i32 as i64
    }
}

pub const O_RDONLY: u32 = 0x0001;
pub const O_WRONLY: u32 = 0x0002;
pub const O_RDWR: u32 = 0x0003;
pub const O_CREAT: u32 = 0x0100;
pub const O_TRUNC: u32 = 0x0200;
pub const O_APPEND: u32 = 0x0400;
pub const O_DIRECTORY: u32 = 0x10000;

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
    pub fn empty() -> Self {
        Self {
            inode: 0,
            size: 0,
            blocks: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            file_type: 0,
            nlink: 1,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    pub inode: InodeId,
    pub file_type: FileType,
    pub name: String,
}

/// ABI structure for directory listings passed across syscall boundary
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UserDirEntry {
    pub inode: u64,
    pub file_type: u32,
    pub name_len: u32,
    pub name: [u8; 128],
}

impl UserDirEntry {
    pub fn new(inode: u64, file_type: FileType, name_str: &str) -> Self {
        let mut entry = Self {
            inode,
            file_type: file_type as u32,
            name_len: 0,
            name: [0u8; 128],
        };
        let bytes = name_str.as_bytes();
        let copy_len = bytes.len().min(127);
        entry.name[..copy_len].copy_from_slice(&bytes[..copy_len]);
        entry.name_len = copy_len as u32;
        entry
    }
}
