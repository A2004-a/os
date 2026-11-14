//! NovaOS Apex — In-Memory RAM Filesystem (RAMFS)
//! Serves as root filesystem, housing system hierarchy, configuration files, and user profiles.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::fs::types::*;
use crate::fs::vfs::FileSystem;
use crate::security::*;

enum RamNodeData {
    File(Vec<u8>),
    Directory(Vec<(String, InodeId)>),
}

struct RamInode {
    stat: FileStat,
    data: RamNodeData,
}

pub struct RamFs {
    next_inode: u64,
    inodes: BTreeMap<InodeId, RamInode>,
}

impl RamFs {
    pub fn new() -> Self {
        let mut fs = Self {
            next_inode: 2,
            inodes: BTreeMap::new(),
        };

        // Create root inode (Inode 1)
        let mut root_stat = FileStat::empty();
        root_stat.inode = 1;
        root_stat.file_type = FileType::Directory as u32;
        root_stat.mode = S_IFDIR | 0o755;
        root_stat.uid = 0;
        root_stat.gid = 0;
        root_stat.nlink = 2;

        let mut root_entries = Vec::new();
        root_entries.push((".".to_string(), InodeId(1)));
        root_entries.push(("..".to_string(), InodeId(1)));

        fs.inodes.insert(
            InodeId(1),
            RamInode {
                stat: root_stat,
                data: RamNodeData::Directory(root_entries),
            },
        );

        fs
    }

    fn alloc_inode_id(&mut self) -> InodeId {
        let id = InodeId(self.next_inode);
        self.next_inode += 1;
        id
    }

    /// Helper to create a file with static contents
    pub fn write_file_content(
        &mut self,
        parent: InodeId,
        name: &str,
        content: &[u8],
        mode: u32,
        uid: u32,
        gid: u32,
    ) -> Result<InodeId, FsError> {
        let creds = Credentials::user(uid, gid);
        let id = self.create(parent, name, mode, &creds)?;
        self.write(id, 0, content)?;
        Ok(id)
    }
}

impl FileSystem for RamFs {
    fn root_inode(&self) -> InodeId {
        InodeId(1)
    }

    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError> {
        let node = self.inodes.get(&parent).ok_or(FsError::NotFound)?;
        match &node.data {
            RamNodeData::Directory(entries) => {
                for (entry_name, child_id) in entries {
                    if entry_name == name {
                        return Ok(*child_id);
                    }
                }
                Err(FsError::NotFound)
            }
            RamNodeData::File(_) => Err(FsError::NotADirectory),
        }
    }

    fn getattr(&self, inode: InodeId) -> Result<FileStat, FsError> {
        let node = self.inodes.get(&inode).ok_or(FsError::NotFound)?;
        Ok(node.stat)
    }

    fn setattr(&mut self, inode: InodeId, stat: &FileStat) -> Result<(), FsError> {
        let node = self.inodes.get_mut(&inode).ok_or(FsError::NotFound)?;
        node.stat = *stat;
        Ok(())
    }

    fn read(&self, inode: InodeId, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let node = self.inodes.get(&inode).ok_or(FsError::NotFound)?;
        match &node.data {
            RamNodeData::File(data) => {
                let off = offset as usize;
                if off >= data.len() {
                    return Ok(0); // EOF
                }
                let available = data.len() - off;
                let to_read = buf.len().min(available);
                buf[..to_read].copy_from_slice(&data[off..off + to_read]);
                Ok(to_read)
            }
            RamNodeData::Directory(_) => Err(FsError::IsADirectory),
        }
    }

    fn write(&mut self, inode: InodeId, offset: u64, buf: &[u8]) -> Result<usize, FsError> {
        let node = self.inodes.get_mut(&inode).ok_or(FsError::NotFound)?;
        match &mut node.data {
            RamNodeData::File(data) => {
                let off = offset as usize;
                let end = off + buf.len();
                if end > data.len() {
                    data.resize(end, 0);
                }
                data[off..end].copy_from_slice(buf);
                node.stat.size = data.len() as u64;
                Ok(buf.len())
            }
            RamNodeData::Directory(_) => Err(FsError::IsADirectory),
        }
    }

    fn create(&mut self, parent: InodeId, name: &str, mode: u32, creds: &Credentials) -> Result<InodeId, FsError> {
        if self.lookup(parent, name).is_ok() {
            return Err(FsError::AlreadyExists);
        }

        let id = self.alloc_inode_id();
        let mut stat = FileStat::empty();
        stat.inode = id.0;
        stat.file_type = FileType::File as u32;
        stat.mode = S_IFREG | (mode & 0o7777);
        stat.uid = creds.euid.0;
        stat.gid = creds.egid.0;
        stat.nlink = 1;

        self.inodes.insert(
            id,
            RamInode {
                stat,
                data: RamNodeData::File(Vec::new()),
            },
        );

        let parent_node = self.inodes.get_mut(&parent).ok_or(FsError::NotFound)?;
        if let RamNodeData::Directory(entries) = &mut parent_node.data {
            entries.push((name.to_string(), id));
            Ok(id)
        } else {
            Err(FsError::NotADirectory)
        }
    }

    fn mkdir(&mut self, parent: InodeId, name: &str, mode: u32, creds: &Credentials) -> Result<InodeId, FsError> {
        if self.lookup(parent, name).is_ok() {
            return Err(FsError::AlreadyExists);
        }

        let id = self.alloc_inode_id();
        let mut stat = FileStat::empty();
        stat.inode = id.0;
        stat.file_type = FileType::Directory as u32;
        stat.mode = S_IFDIR | (mode & 0o7777);
        stat.uid = creds.euid.0;
        stat.gid = creds.egid.0;
        stat.nlink = 2;

        let mut entries = Vec::new();
        entries.push((".".to_string(), id));
        entries.push(("..".to_string(), parent));

        self.inodes.insert(
            id,
            RamInode {
                stat,
                data: RamNodeData::Directory(entries),
            },
        );

        let parent_node = self.inodes.get_mut(&parent).ok_or(FsError::NotFound)?;
        if let RamNodeData::Directory(entries) = &mut parent_node.data {
            entries.push((name.to_string(), id));
            parent_node.stat.nlink += 1;
            Ok(id)
        } else {
            Err(FsError::NotADirectory)
        }
    }

    fn unlink(&mut self, parent: InodeId, name: &str) -> Result<(), FsError> {
        let child_id = self.lookup(parent, name)?;
        let child_stat = self.getattr(child_id)?;
        if (child_stat.mode & S_IFDIR) != 0 {
            return Err(FsError::IsADirectory);
        }

        let parent_node = self.inodes.get_mut(&parent).ok_or(FsError::NotFound)?;
        if let RamNodeData::Directory(entries) = &mut parent_node.data {
            entries.retain(|(n, _)| n != name);
            self.inodes.remove(&child_id);
            Ok(())
        } else {
            Err(FsError::NotADirectory)
        }
    }

    fn rmdir(&mut self, parent: InodeId, name: &str) -> Result<(), FsError> {
        let child_id = self.lookup(parent, name)?;
        let child_node = self.inodes.get(&child_id).ok_or(FsError::NotFound)?;
        if let RamNodeData::Directory(entries) = &child_node.data {
            if entries.len() > 2 {
                // Contains more than . and ..
                return Err(FsError::Busy);
            }
        } else {
            return Err(FsError::NotADirectory);
        }

        let parent_node = self.inodes.get_mut(&parent).ok_or(FsError::NotFound)?;
        if let RamNodeData::Directory(entries) = &mut parent_node.data {
            entries.retain(|(n, _)| n != name);
            parent_node.stat.nlink = parent_node.stat.nlink.saturating_sub(1);
            self.inodes.remove(&child_id);
            Ok(())
        } else {
            Err(FsError::NotADirectory)
        }
    }

    fn readdir(&self, inode: InodeId, offset: usize) -> Result<Option<DirEntry>, FsError> {
        let node = self.inodes.get(&inode).ok_or(FsError::NotFound)?;
        match &node.data {
            RamNodeData::Directory(entries) => {
                if offset >= entries.len() {
                    return Ok(None);
                }
                let (name, child_id) = &entries[offset];
                let child_node = self.inodes.get(child_id).ok_or(FsError::NotFound)?;
                let file_type = if (child_node.stat.mode & S_IFDIR) != 0 {
                    FileType::Directory
                } else {
                    FileType::File
                };
                Ok(Some(DirEntry {
                    inode: *child_id,
                    file_type,
                    name: name.clone(),
                }))
            }
            RamNodeData::File(_) => Err(FsError::NotADirectory),
        }
    }

    fn truncate(&mut self, inode: InodeId, size: u64) -> Result<(), FsError> {
        let node = self.inodes.get_mut(&inode).ok_or(FsError::NotFound)?;
        match &mut node.data {
            RamNodeData::File(data) => {
                data.resize(size as usize, 0);
                node.stat.size = size;
                Ok(())
            }
            RamNodeData::Directory(_) => Err(FsError::IsADirectory),
        }
    }
}

/// Populates the initial system directory layout and default configuration files
pub fn populate_default_hierarchy(ramfs: &mut RamFs) {
    let root = InodeId(1);
    let root_creds = Credentials::root();
    let user_creds = Credentials::user(1000, 1000);

    // 1. Create standard root folders
    let _bin_dir = ramfs.mkdir(root, "bin", 0o755, &root_creds).unwrap();
    let etc_dir = ramfs.mkdir(root, "etc", 0o755, &root_creds).unwrap();
    let _dev_dir = ramfs.mkdir(root, "dev", 0o755, &root_creds).unwrap();
    let _tmp_dir = ramfs.mkdir(root, "tmp", 0o777, &root_creds).unwrap();
    let var_dir = ramfs.mkdir(root, "var", 0o755, &root_creds).unwrap();
    let users_dir = ramfs.mkdir(root, "users", 0o755, &root_creds).unwrap();
    let _storage_dir = ramfs.mkdir(root, "storage", 0o755, &root_creds).unwrap();

    // 2. Populate /etc files
    let os_release = b"NAME=\"NovaOS Apex\"\nVERSION=\"0.5.0-alpha\"\nID=novaos\nID_LIKE=independent\nPRETTY_NAME=\"NovaOS Apex 0.5.0-alpha (Bare-Metal x86_64)\"\nANSI_COLOR=\"0;36\"\nHOME_URL=\"https://novaos.org\"\n";
    let _ = ramfs.write_file_content(etc_dir, "os-release", os_release, 0o644, 0, 0);

    let hostname = b"novaos-apex\n";
    let _ = ramfs.write_file_content(etc_dir, "hostname", hostname, 0o644, 0, 0);

    let passwd = b"root:x:0:0:System Administrator:/root:/bin/sh\nsystem:x:1:1:System Daemon:/var/empty:/bin/nologin\napex:x:1000:1000:NovaOS User:/users/apex:/bin/sh\n";
    let _ = ramfs.write_file_content(etc_dir, "passwd", passwd, 0o644, 0, 0);

    let group = b"root:x:0:\nsystem:x:1:\nusers:x:1000:apex\n";
    let _ = ramfs.write_file_content(etc_dir, "group", group, 0o644, 0, 0);

    let motd = b"===========================================================\n  Welcome to NovaOS Apex Desktop Platform (Milestone 5)\n  Original x86_64 Bare-Metal Operating System\n===========================================================\n";
    let _ = ramfs.write_file_content(etc_dir, "motd", motd, 0o644, 0, 0);

    // 3. Populate /users/apex profile
    let apex_home = ramfs.mkdir(users_dir, "apex", 0o750, &user_creds).unwrap();
    let _ = ramfs.mkdir(apex_home, "Desktop", 0o750, &user_creds);
    let _ = ramfs.mkdir(apex_home, "Documents", 0o750, &user_creds);
    let _ = ramfs.mkdir(apex_home, "Downloads", 0o750, &user_creds);
    let _ = ramfs.mkdir(apex_home, ".config", 0o700, &user_creds);

    let welcome = b"Welcome to NovaOS Apex!\nNovaOS Apex is an original, bare-metal operating system engineered\nfor modern PC hardware with pure Rust kernel safety, capability-based\nisolation, and a resilient multi-tier filesystem.\n";
    let _ = ramfs.write_file_content(apex_home, "welcome.txt", welcome, 0o644, 1000, 1000);

    // 4. Create /var subdirectories
    let _ = ramfs.mkdir(var_dir, "log", 0o755, &root_creds);
    let _ = ramfs.mkdir(var_dir, "run", 0o755, &root_creds);
}
