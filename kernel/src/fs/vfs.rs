//! NovaOS Zero — Virtual Filesystem (VFS) Abstraction Layer
//! Defines Inodes, Directory Entries, File Operations, and Global Path Resolution.

use crate::sync::Spinlock;

pub const MAX_PATH_LEN: usize = 128;
pub const MAX_NAME_LEN: usize = 32;
pub const MAX_OPEN_FILES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
    BlockDevice,
    CharDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    PermissionDenied,
    NotADirectory,
    IsADirectory,
    DeviceError,
    NoSpace,
    InvalidPath,
    FileTableFull,
    InvalidHandle,
}

#[derive(Debug, Clone, Copy)]
pub struct InodeMetadata {
    pub inode_number: u32,
    pub node_type: NodeType,
    pub size: u32,
    pub permissions: u16,
    pub blocks: u32,
}

pub trait FileOperations: Send + Sync {
    fn read(&mut self, inode: u32, offset: u32, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&mut self, inode: u32, offset: u32, buf: &[u8]) -> Result<usize, FsError>;
    fn create(&mut self, parent_inode: u32, name: &str, node_type: NodeType) -> Result<u32, FsError>;
    fn lookup(&mut self, parent_inode: u32, name: &str) -> Result<u32, FsError>;
    fn get_metadata(&mut self, inode: u32) -> Result<InodeMetadata, FsError>;
    fn list_dir(&mut self, parent_inode: u32, out_names: &mut [[u8; MAX_NAME_LEN]], max_entries: usize) -> Result<usize, FsError>;
}

#[derive(Clone, Copy)]
pub struct OpenFileEntry {
    pub inode: u32,
    pub offset: u32,
    pub is_writable: bool,
    pub is_active: bool,
}

pub struct VirtualFileSystem {
    pub root_ops: Option<&'static mut (dyn FileOperations + 'static)>,
    open_table: [OpenFileEntry; MAX_OPEN_FILES],
}

impl VirtualFileSystem {
    pub const fn new() -> Self {
        VirtualFileSystem {
            root_ops: None,
            open_table: [OpenFileEntry {
                inode: 0,
                offset: 0,
                is_writable: false,
                is_active: false,
            }; MAX_OPEN_FILES],
        }
    }

    pub fn mount_root(&mut self, ops: &'static mut (dyn FileOperations + 'static)) {
        self.root_ops = Some(ops);
    }

    /// Resolve an absolute path to an inode number
    pub fn resolve_path(&mut self, path: &str) -> Result<u32, FsError> {
        let ops = self.root_ops.as_mut().ok_or(FsError::DeviceError)?;

        if path.is_empty() || !path.starts_with('/') {
            return Err(FsError::InvalidPath);
        }

        if path == "/" {
            return Ok(1); // Root directory inode is always 1
        }

        let mut current_inode = 1;
        let segments = path.split('/').filter(|s| !s.is_empty());

        for segment in segments {
            current_inode = ops.lookup(current_inode, segment)?;
        }

        Ok(current_inode)
    }

    /// Open a file and return a virtual file descriptor (index into open_table)
    pub fn open(&mut self, path: &str, writable: bool) -> Result<usize, FsError> {
        let inode = match self.resolve_path(path) {
            Ok(i) => i,
            Err(FsError::NotFound) if writable => {
                // Auto-create file in parent directory
                let (parent_path, filename) = match path.rfind('/') {
                    Some(0) => ("/", &path[1..]),
                    Some(idx) => (&path[..idx], &path[idx + 1..]),
                    None => return Err(FsError::InvalidPath),
                };
                let parent_inode = self.resolve_path(parent_path)?;
                let ops = self.root_ops.as_mut().ok_or(FsError::DeviceError)?;
                ops.create(parent_inode, filename, NodeType::File)?
            }
            Err(e) => return Err(e),
        };

        for (idx, slot) in self.open_table.iter_mut().enumerate() {
            if !slot.is_active {
                *slot = OpenFileEntry {
                    inode,
                    offset: 0,
                    is_writable: writable,
                    is_active: true,
                };
                return Ok(idx);
            }
        }

        Err(FsError::FileTableFull)
    }

    /// Read from an open file descriptor
    pub fn read(&mut self, fd: usize, buf: &mut [u8]) -> Result<usize, FsError> {
        if fd >= MAX_OPEN_FILES || !self.open_table[fd].is_active {
            return Err(FsError::InvalidHandle);
        }

        let entry = &mut self.open_table[fd];
        let ops = self.root_ops.as_mut().ok_or(FsError::DeviceError)?;

        let bytes_read = ops.read(entry.inode, entry.offset, buf)?;
        entry.offset += bytes_read as u32;
        Ok(bytes_read)
    }

    /// Write to an open file descriptor
    pub fn write(&mut self, fd: usize, buf: &[u8]) -> Result<usize, FsError> {
        if fd >= MAX_OPEN_FILES || !self.open_table[fd].is_active {
            return Err(FsError::InvalidHandle);
        }

        let entry = &mut self.open_table[fd];
        if !entry.is_writable {
            return Err(FsError::PermissionDenied);
        }

        let ops = self.root_ops.as_mut().ok_or(FsError::DeviceError)?;
        let bytes_written = ops.write(entry.inode, entry.offset, buf)?;
        entry.offset += bytes_written as u32;
        Ok(bytes_written)
    }

    /// Close an open file descriptor
    pub fn close(&mut self, fd: usize) -> Result<(), FsError> {
        if fd >= MAX_OPEN_FILES || !self.open_table[fd].is_active {
            return Err(FsError::InvalidHandle);
        }

        self.open_table[fd].is_active = false;
        Ok(())
    }
}

pub static VFS: Spinlock<VirtualFileSystem> = Spinlock::new(VirtualFileSystem::new());
