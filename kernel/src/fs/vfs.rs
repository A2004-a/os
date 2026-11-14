//! NovaOS Apex — Virtual File System (VFS) Layer
//! Provides unified filesystem abstraction, mount management, path resolution, and file operations.

extern crate alloc;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::fs::path::{normalize_path, split_path};
use crate::fs::types::*;
use crate::scheduler::sync::SpinLock;
use crate::security::{check_permission, Credentials, S_IFDIR, S_IFREG};

/// Trait implemented by all file systems (RAMFS, DevFS, NovaFS).
pub trait FileSystem: Send + Sync {
    fn root_inode(&self) -> InodeId;
    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn getattr(&self, inode: InodeId) -> Result<FileStat, FsError>;
    fn setattr(&mut self, inode: InodeId, stat: &FileStat) -> Result<(), FsError>;
    fn read(&self, inode: InodeId, offset: u64, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&mut self, inode: InodeId, offset: u64, buf: &[u8]) -> Result<usize, FsError>;
    fn create(&mut self, parent: InodeId, name: &str, mode: u32, creds: &Credentials) -> Result<InodeId, FsError>;
    fn mkdir(&mut self, parent: InodeId, name: &str, mode: u32, creds: &Credentials) -> Result<InodeId, FsError>;
    fn unlink(&mut self, parent: InodeId, name: &str) -> Result<(), FsError>;
    fn rmdir(&mut self, parent: InodeId, name: &str) -> Result<(), FsError>;
    fn readdir(&self, inode: InodeId, offset: usize) -> Result<Option<DirEntry>, FsError>;
    fn truncate(&mut self, inode: InodeId, size: u64) -> Result<(), FsError>;
}

/// Represents an active open file or stream in userspace or kernel
pub struct OpenFile {
    pub fs: Arc<SpinLock<dyn FileSystem>>,
    pub inode: InodeId,
    pub flags: u32,
    pub offset: u64,
    pub path: String,
}

impl OpenFile {
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, FsError> {
        let bytes_read = self.fs.lock().read(self.inode, self.offset, buf)?;
        self.offset += bytes_read as u64;
        Ok(bytes_read)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, FsError> {
        let bytes_written = self.fs.lock().write(self.inode, self.offset, buf)?;
        self.offset += bytes_written as u64;
        Ok(bytes_written)
    }

    pub fn seek(&mut self, offset: i64, whence: u32) -> Result<u64, FsError> {
        let size = self.fs.lock().getattr(self.inode)?.size;
        let new_offset = match whence {
            SEEK_SET => {
                if offset < 0 {
                    return Err(FsError::InvalidArgument);
                }
                offset as u64
            }
            SEEK_CUR => {
                let cur = self.offset as i64;
                let target = cur + offset;
                if target < 0 {
                    return Err(FsError::InvalidArgument);
                }
                target as u64
            }
            SEEK_END => {
                let end = size as i64;
                let target = end + offset;
                if target < 0 {
                    return Err(FsError::InvalidArgument);
                }
                target as u64
            }
            _ => return Err(FsError::InvalidArgument),
        };

        self.offset = new_offset;
        Ok(self.offset)
    }

    pub fn stat(&self) -> Result<FileStat, FsError> {
        self.fs.lock().getattr(self.inode)
    }

    pub fn readdir(&mut self) -> Result<Option<DirEntry>, FsError> {
        let entry = self.fs.lock().readdir(self.inode, self.offset as usize)?;
        if entry.is_some() {
            self.offset += 1;
        }
        Ok(entry)
    }
}

pub struct MountEntry {
    pub mount_point: String,
    pub fs: Arc<SpinLock<dyn FileSystem>>,
}

pub struct Vfs {
    mounts: Vec<MountEntry>,
}

static VFS_INSTANCE: SpinLock<Option<Vfs>> = SpinLock::new("vfs", None);

impl Vfs {
    pub fn new() -> Self {
        Self { mounts: Vec::new() }
    }

    pub fn mount(&mut self, mount_point: &str, fs: Arc<SpinLock<dyn FileSystem>>) -> Result<(), FsError> {
        let norm_mp = normalize_path("/", mount_point);
        // Remove existing mount if already mounted at this point
        self.mounts.retain(|m| m.mount_point != norm_mp);
        self.mounts.push(MountEntry {
            mount_point: norm_mp,
            fs,
        });
        // Sort descending by mount_point length for longest-prefix match
        self.mounts.sort_by(|a, b| b.mount_point.len().cmp(&a.mount_point.len()));
        Ok(())
    }

    pub fn unmount(&mut self, mount_point: &str) -> Result<(), FsError> {
        let norm_mp = normalize_path("/", mount_point);
        let len_before = self.mounts.len();
        self.mounts.retain(|m| m.mount_point != norm_mp);
        if self.mounts.len() < len_before {
            Ok(())
        } else {
            Err(FsError::NotFound)
        }
    }

    /// Resolves which mounted filesystem owns the path, and returns the FS along with the relative subpath.
    pub fn resolve_mount(&self, path: &str) -> Result<(Arc<SpinLock<dyn FileSystem>>, String), FsError> {
        let norm = normalize_path("/", path);
        for m in &self.mounts {
            if norm == m.mount_point {
                return Ok((m.fs.clone(), String::from("/")));
            }
            if norm.starts_with(&m.mount_point) {
                let sub = if m.mount_point == "/" {
                    &norm[1..]
                } else if norm.as_bytes().get(m.mount_point.len()) == Some(&b'/') {
                    &norm[m.mount_point.len() + 1..]
                } else {
                    continue;
                };
                return Ok((m.fs.clone(), String::from(sub)));
            }
        }
        Err(FsError::NotFound)
    }

    /// Resolves a path to its filesystem and InodeId.
    pub fn resolve_inode(&self, path: &str) -> Result<(Arc<SpinLock<dyn FileSystem>>, InodeId), FsError> {
        let (fs_lock, subpath) = self.resolve_mount(path)?;
        let fs = fs_lock.lock();
        let mut cur_inode = fs.root_inode();

        if subpath == "/" || subpath.is_empty() {
            return Ok((fs_lock.clone(), cur_inode));
        }

        for part in subpath.split('/') {
            if part.is_empty() || part == "." {
                continue;
            }
            cur_inode = fs.lookup(cur_inode, part)?;
        }

        Ok((fs_lock.clone(), cur_inode))
    }

    /// Opens a file with the given flags and mode.
    pub fn open(&self, path: &str, flags: u32, mode: u32, creds: &Credentials) -> Result<OpenFile, FsError> {
        let norm = normalize_path("/", path);
        let (dir_part, base_name) = split_path(&norm);

        let want_read = (flags & O_RDWR) != 0 || (flags & O_WRONLY) == 0;
        let want_write = (flags & O_WRONLY) != 0 || (flags & O_RDWR) != 0 || (flags & O_TRUNC) != 0 || (flags & O_APPEND) != 0;

        // Try direct lookup
        match self.resolve_inode(&norm) {
            Ok((fs_arc, inode)) => {
                // Inode exists
                let stat = fs_arc.lock().getattr(inode)?;
                let is_dir = (stat.mode & S_IFDIR) != 0;

                if (flags & O_DIRECTORY) != 0 && !is_dir {
                    return Err(FsError::NotADirectory);
                }
                if is_dir && want_write {
                    return Err(FsError::IsADirectory);
                }

                // Check access permission
                if !check_permission(stat.mode, crate::security::Uid(stat.uid), crate::security::Gid(stat.gid), creds, want_read, want_write, false) {
                    return Err(FsError::PermissionDenied);
                }

                if (flags & O_TRUNC) != 0 && want_write {
                    fs_arc.lock().truncate(inode, 0)?;
                }

                let initial_offset = if (flags & O_APPEND) != 0 {
                    stat.size
                } else {
                    0
                };

                Ok(OpenFile {
                    fs: fs_arc,
                    inode,
                    flags,
                    offset: initial_offset,
                    path: norm,
                })
            }
            Err(FsError::NotFound) => {
                // File does not exist; check if O_CREAT was specified
                if (flags & O_CREAT) == 0 {
                    return Err(FsError::NotFound);
                }

                let (parent_fs, parent_inode) = self.resolve_inode(dir_part)?;
                let new_inode = parent_fs.lock().create(parent_inode, base_name, mode | S_IFREG, creds)?;

                Ok(OpenFile {
                    fs: parent_fs,
                    inode: new_inode,
                    flags,
                    offset: 0,
                    path: norm,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Queries file metadata by path.
    pub fn stat(&self, path: &str, creds: &Credentials) -> Result<FileStat, FsError> {
        let (fs_arc, inode) = self.resolve_inode(path)?;
        let stat = fs_arc.lock().getattr(inode)?;
        if !check_permission(stat.mode, crate::security::Uid(stat.uid), crate::security::Gid(stat.gid), creds, true, false, false) {
            return Err(FsError::PermissionDenied);
        }
        Ok(stat)
    }

    /// Creates a new directory at the specified path.
    pub fn mkdir(&self, path: &str, mode: u32, creds: &Credentials) -> Result<(), FsError> {
        let norm = normalize_path("/", path);
        let (dir_part, base_name) = split_path(&norm);
        let (parent_fs, parent_inode) = self.resolve_inode(dir_part)?;

        let parent_stat = parent_fs.lock().getattr(parent_inode)?;
        if !check_permission(parent_stat.mode, crate::security::Uid(parent_stat.uid), crate::security::Gid(parent_stat.gid), creds, false, true, true) {
            return Err(FsError::PermissionDenied);
        }

        parent_fs.lock().mkdir(parent_inode, base_name, mode | S_IFDIR, creds)?;
        Ok(())
    }

    /// Unlinks (removes) a file from its parent directory.
    pub fn unlink(&self, path: &str, creds: &Credentials) -> Result<(), FsError> {
        let norm = normalize_path("/", path);
        let (dir_part, base_name) = split_path(&norm);
        let (parent_fs, parent_inode) = self.resolve_inode(dir_part)?;

        let parent_stat = parent_fs.lock().getattr(parent_inode)?;
        if !check_permission(parent_stat.mode, crate::security::Uid(parent_stat.uid), crate::security::Gid(parent_stat.gid), creds, false, true, true) {
            return Err(FsError::PermissionDenied);
        }

        let res = parent_fs.lock().unlink(parent_inode, base_name);
        res
    }

    /// Reads directory entries sequentially.
    pub fn readdir(&self, path: &str, offset: usize, creds: &Credentials) -> Result<Option<DirEntry>, FsError> {
        let (fs_arc, inode) = self.resolve_inode(path)?;
        let stat = fs_arc.lock().getattr(inode)?;
        if (stat.mode & S_IFDIR) == 0 {
            return Err(FsError::NotADirectory);
        }
        if !check_permission(stat.mode, crate::security::Uid(stat.uid), crate::security::Gid(stat.gid), creds, true, false, true) {
            return Err(FsError::PermissionDenied);
        }

        let res = fs_arc.lock().readdir(inode, offset);
        res
    }
}

pub fn init_vfs(root_fs: Arc<SpinLock<dyn FileSystem>>) {
    let mut vfs = Vfs::new();
    let _ = vfs.mount("/", root_fs);
    *VFS_INSTANCE.lock() = Some(vfs);
}

pub fn get_vfs() -> Option<crate::scheduler::sync::SpinLockGuard<'static, Option<Vfs>>> {
    Some(VFS_INSTANCE.lock())
}
