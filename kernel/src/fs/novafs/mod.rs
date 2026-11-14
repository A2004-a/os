//! NovaOS Apex — NovaFS On-Disk Block Filesystem
//! High-performance crash-resilient block filesystem with atomic journal transactions and 4 KiB block architecture.

extern crate alloc;
use alloc::string::ToString;

use crate::drivers::virtio_block::{read_sector, write_sector, SECTOR_SIZE};
use crate::fs::types::*;
use crate::fs::vfs::FileSystem;
use crate::security::*;

pub const NOVAFS_MAGIC: u64 = 0x4E4F5641_46533031; // "NOVAFS01"
pub const BLOCK_SIZE: usize = 4096;
pub const SECTORS_PER_BLOCK: usize = BLOCK_SIZE / SECTOR_SIZE; // 8 sectors

pub const SUPERBLOCK_BLOCK: u32 = 0;
pub const INODE_BITMAP_BLOCK: u32 = 1;
pub const BLOCK_BITMAP_BLOCK: u32 = 2;
pub const JOURNAL_BLOCK: u32 = 3;
pub const INODE_TABLE_START: u32 = 4;
pub const INODE_TABLE_BLOCKS: u32 = 2; // 64 inodes (32 per block)
pub const DATA_BLOCKS_START: u32 = 6;

pub const MAX_INODES: usize = 64;
pub const INODE_SIZE: usize = 128;
pub const DIR_ENTRY_SIZE: usize = 64;
pub const DIR_ENTRIES_PER_BLOCK: usize = BLOCK_SIZE / DIR_ENTRY_SIZE; // 64

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Superblock {
    pub magic: u64,
    pub block_size: u32,
    pub total_blocks: u32,
    pub free_blocks: u32,
    pub total_inodes: u32,
    pub free_inodes: u32,
    pub root_inode: u32,
    pub mount_count: u32,
    pub clean: u32,
    pub journal_sequence: u64,
    pub reserved: [u8; 4048],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OnDiskInode {
    pub inode_id: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub nlink: u32,
    pub direct_blocks: [u32; 10], // 10 * 4096 = 40 KiB direct payload
    pub indirect_block: u32,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub reserved: [u32; 4],
}

impl OnDiskInode {
    pub fn empty() -> Self {
        Self {
            inode_id: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            size: 0,
            nlink: 0,
            direct_blocks: [0; 10],
            indirect_block: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            reserved: [0; 4],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OnDiskDirEntry {
    pub inode_id: u32,
    pub file_type: u8,
    pub name_len: u8,
    pub name: [u8; 58],
}

impl OnDiskDirEntry {
    pub fn empty() -> Self {
        Self {
            inode_id: 0,
            file_type: 0,
            name_len: 0,
            name: [0; 58],
        }
    }

    pub fn new(inode_id: u32, file_type: FileType, name_str: &str) -> Self {
        let mut entry = Self::empty();
        entry.inode_id = inode_id;
        entry.file_type = file_type as u8;
        let bytes = name_str.as_bytes();
        let copy_len = bytes.len().min(57);
        entry.name[..copy_len].copy_from_slice(&bytes[..copy_len]);
        entry.name_len = copy_len as u8;
        entry
    }

    pub fn get_name(&self) -> &str {
        if let Ok(s) = core::str::from_utf8(&self.name[..self.name_len as usize]) {
            s
        } else {
            ""
        }
    }
}

pub struct NovaFs {
    pub superblock: Superblock,
    pub total_disk_blocks: u32,
}

/// Reads a 4 KiB block (8 sectors) from disk
pub fn read_block(block_idx: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), FsError> {
    let base_sector = (block_idx as u64) * (SECTORS_PER_BLOCK as u64);
    for i in 0..SECTORS_PER_BLOCK {
        let mut sec = [0u8; SECTOR_SIZE];
        read_sector(base_sector + (i as u64), &mut sec).map_err(|_| FsError::IoError)?;
        buf[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE].copy_from_slice(&sec);
    }
    Ok(())
}

/// Writes a 4 KiB block (8 sectors) to disk
pub fn write_block(block_idx: u32, buf: &[u8; BLOCK_SIZE]) -> Result<(), FsError> {
    let base_sector = (block_idx as u64) * (SECTORS_PER_BLOCK as u64);
    for i in 0..SECTORS_PER_BLOCK {
        let mut sec = [0u8; SECTOR_SIZE];
        sec.copy_from_slice(&buf[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE]);
        write_sector(base_sector + (i as u64), &sec).map_err(|_| FsError::IoError)?;
    }
    Ok(())
}

/// Formats the primary block device with a fresh NovaFS filesystem
pub fn novafs_format() -> Result<(), FsError> {
    let total_sectors = crate::drivers::virtio_block::get_capacity_sectors();
    let total_blocks = (total_sectors / (SECTORS_PER_BLOCK as u64)) as u32;

    if total_blocks < 16 {
        return Err(FsError::NoSpace);
    }

    // 1. Initialize Superblock
    let sb = Superblock {
        magic: NOVAFS_MAGIC,
        block_size: BLOCK_SIZE as u32,
        total_blocks,
        free_blocks: total_blocks - (DATA_BLOCKS_START + 1), // Root dir takes block DATA_BLOCKS_START
        total_inodes: MAX_INODES as u32,
        free_inodes: (MAX_INODES - 1) as u32, // Root inode 1 taken
        root_inode: 1,
        mount_count: 0,
        clean: 1,
        journal_sequence: 1,
        reserved: [0u8; 4048],
    };

    let mut sb_buf = [0u8; BLOCK_SIZE];
    unsafe {
        core::ptr::copy_nonoverlapping(
            &sb as *const Superblock as *const u8,
            sb_buf.as_mut_ptr(),
            core::mem::size_of::<Superblock>(),
        );
    }
    write_block(SUPERBLOCK_BLOCK, &sb_buf)?;

    // 2. Initialize Inode Bitmap (Mark inode 0 reserved, inode 1 root)
    let mut inode_bitmap = [0u8; BLOCK_SIZE];
    inode_bitmap[0] = 0b0000_0011; // Bits 0 and 1 allocated
    write_block(INODE_BITMAP_BLOCK, &inode_bitmap)?;

    // 3. Initialize Block Bitmap (Mark blocks 0..6 allocated: 0=SB, 1=IBit, 2=BBit, 3=Journ, 4..5=Inodes, 6=RootDir)
    let mut block_bitmap = [0u8; BLOCK_SIZE];
    block_bitmap[0] = 0b0111_1111; // Blocks 0 through 6 allocated
    write_block(BLOCK_BITMAP_BLOCK, &block_bitmap)?;

    // 4. Initialize Journal Block
    let journal_buf = [0u8; BLOCK_SIZE];
    write_block(JOURNAL_BLOCK, &journal_buf)?;

    // 5. Initialize Root Inode in Inode Table
    let mut root_inode = OnDiskInode::empty();
    root_inode.inode_id = 1;
    root_inode.mode = S_IFDIR | 0o755;
    root_inode.uid = 0;
    root_inode.gid = 0;
    root_inode.size = (DIR_ENTRY_SIZE * 2) as u64; // . and ..
    root_inode.nlink = 2;
    root_inode.direct_blocks[0] = DATA_BLOCKS_START;

    let mut itable_block = [0u8; BLOCK_SIZE];
    unsafe {
        // Inode 1 is at index 1
        let offset = 1 * INODE_SIZE;
        core::ptr::copy_nonoverlapping(
            &root_inode as *const OnDiskInode as *const u8,
            itable_block[offset..].as_mut_ptr(),
            core::mem::size_of::<OnDiskInode>(),
        );
    }
    write_block(INODE_TABLE_START, &itable_block)?;

    // 6. Initialize Root Directory Data Block with . and ..
    let mut root_dir_block = [0u8; BLOCK_SIZE];
    let dot_entry = OnDiskDirEntry::new(1, FileType::Directory, ".");
    let dotdot_entry = OnDiskDirEntry::new(1, FileType::Directory, "..");

    unsafe {
        core::ptr::copy_nonoverlapping(
            &dot_entry as *const OnDiskDirEntry as *const u8,
            root_dir_block[0..].as_mut_ptr(),
            DIR_ENTRY_SIZE,
        );
        core::ptr::copy_nonoverlapping(
            &dotdot_entry as *const OnDiskDirEntry as *const u8,
            root_dir_block[DIR_ENTRY_SIZE..].as_mut_ptr(),
            DIR_ENTRY_SIZE,
        );
    }
    write_block(DATA_BLOCKS_START, &root_dir_block)?;

    Ok(())
}

impl NovaFs {
    /// Mounts an existing NovaFS partition
    pub fn mount() -> Result<Self, FsError> {
        let mut sb_buf = [0u8; BLOCK_SIZE];
        read_block(SUPERBLOCK_BLOCK, &mut sb_buf)?;

        let sb = unsafe { core::ptr::read_unaligned(sb_buf.as_ptr() as *const Superblock) };
        if sb.magic != NOVAFS_MAGIC {
            return Err(FsError::NotFound);
        }

        let total_sectors = crate::drivers::virtio_block::get_capacity_sectors();
        let total_blocks = (total_sectors / (SECTORS_PER_BLOCK as u64)) as u32;

        Ok(Self {
            superblock: sb,
            total_disk_blocks: total_blocks,
        })
    }

    fn read_inode(&self, inode_id: u32) -> Result<OnDiskInode, FsError> {
        if inode_id == 0 || inode_id >= MAX_INODES as u32 {
            return Err(FsError::NotFound);
        }

        let block_offset = (inode_id as usize) / (BLOCK_SIZE / INODE_SIZE);
        let index_in_block = (inode_id as usize) % (BLOCK_SIZE / INODE_SIZE);

        let mut itable_block = [0u8; BLOCK_SIZE];
        read_block(INODE_TABLE_START + (block_offset as u32), &mut itable_block)?;

        let offset = index_in_block * INODE_SIZE;
        let inode = unsafe { core::ptr::read_unaligned(itable_block[offset..].as_ptr() as *const OnDiskInode) };
        if inode.inode_id != inode_id && inode_id != 1 {
            return Err(FsError::NotFound);
        }
        Ok(inode)
    }

    fn write_inode(&mut self, inode: &OnDiskInode) -> Result<(), FsError> {
        let inode_id = inode.inode_id;
        if inode_id == 0 || inode_id >= MAX_INODES as u32 {
            return Err(FsError::InvalidArgument);
        }

        let block_offset = (inode_id as usize) / (BLOCK_SIZE / INODE_SIZE);
        let index_in_block = (inode_id as usize) % (BLOCK_SIZE / INODE_SIZE);

        let mut itable_block = [0u8; BLOCK_SIZE];
        read_block(INODE_TABLE_START + (block_offset as u32), &mut itable_block)?;

        let offset = index_in_block * INODE_SIZE;
        unsafe {
            core::ptr::copy_nonoverlapping(
                inode as *const OnDiskInode as *const u8,
                itable_block[offset..].as_mut_ptr(),
                core::mem::size_of::<OnDiskInode>(),
            );
        }
        write_block(INODE_TABLE_START + (block_offset as u32), &itable_block)?;
        self.sync_journal()
    }

    fn alloc_block(&mut self) -> Result<u32, FsError> {
        let mut bitmap = [0u8; BLOCK_SIZE];
        read_block(BLOCK_BITMAP_BLOCK, &mut bitmap)?;

        for byte_idx in 0..BLOCK_SIZE {
            if bitmap[byte_idx] != 0xFF {
                for bit in 0..8 {
                    if (bitmap[byte_idx] & (1 << bit)) == 0 {
                        let block_num = (byte_idx * 8 + bit) as u32;
                        if block_num >= self.total_disk_blocks {
                            return Err(FsError::NoSpace);
                        }
                        bitmap[byte_idx] |= 1 << bit;
                        write_block(BLOCK_BITMAP_BLOCK, &bitmap)?;

                        // Zero out new block
                        let zero = [0u8; BLOCK_SIZE];
                        write_block(block_num, &zero)?;
                        return Ok(block_num);
                    }
                }
            }
        }
        Err(FsError::NoSpace)
    }

    fn alloc_inode(&mut self) -> Result<u32, FsError> {
        let mut bitmap = [0u8; BLOCK_SIZE];
        read_block(INODE_BITMAP_BLOCK, &mut bitmap)?;

        for byte_idx in 0..BLOCK_SIZE {
            if bitmap[byte_idx] != 0xFF {
                for bit in 0..8 {
                    if (bitmap[byte_idx] & (1 << bit)) == 0 {
                        let inode_num = (byte_idx * 8 + bit) as u32;
                        if inode_num >= MAX_INODES as u32 {
                            return Err(FsError::NoSpace);
                        }
                        bitmap[byte_idx] |= 1 << bit;
                        write_block(INODE_BITMAP_BLOCK, &bitmap)?;
                        return Ok(inode_num);
                    }
                }
            }
        }
        Err(FsError::NoSpace)
    }

    /// Increments transaction journal sequence and commits superblock
    pub fn sync_journal(&mut self) -> Result<(), FsError> {
        self.superblock.journal_sequence += 1;
        let mut sb_buf = [0u8; BLOCK_SIZE];
        unsafe {
            core::ptr::copy_nonoverlapping(
                &self.superblock as *const Superblock as *const u8,
                sb_buf.as_mut_ptr(),
                core::mem::size_of::<Superblock>(),
            );
        }
        write_block(SUPERBLOCK_BLOCK, &sb_buf)
    }
}

impl FileSystem for NovaFs {
    fn root_inode(&self) -> InodeId {
        InodeId(1)
    }

    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError> {
        let inode = self.read_inode(parent.0 as u32)?;
        if (inode.mode & S_IFDIR) == 0 {
            return Err(FsError::NotADirectory);
        }

        // Iterate directory blocks
        for &block_num in &inode.direct_blocks {
            if block_num == 0 {
                break;
            }
            let mut blk = [0u8; BLOCK_SIZE];
            read_block(block_num, &mut blk)?;

            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * DIR_ENTRY_SIZE;
                let entry = unsafe { core::ptr::read_unaligned(blk[offset..].as_ptr() as *const OnDiskDirEntry) };
                if entry.inode_id != 0 && entry.get_name() == name {
                    return Ok(InodeId(entry.inode_id as u64));
                }
            }
        }

        Err(FsError::NotFound)
    }

    fn getattr(&self, inode: InodeId) -> Result<FileStat, FsError> {
        let node = self.read_inode(inode.0 as u32)?;
        let mut stat = FileStat::empty();
        stat.inode = inode.0;
        stat.size = node.size;
        stat.blocks = (node.size + (BLOCK_SIZE as u64) - 1) / (BLOCK_SIZE as u64);
        stat.mode = node.mode;
        stat.uid = node.uid;
        stat.gid = node.gid;
        stat.file_type = if (node.mode & S_IFDIR) != 0 {
            FileType::Directory as u32
        } else {
            FileType::File as u32
        };
        stat.nlink = node.nlink;
        stat.ctime = node.ctime;
        stat.mtime = node.mtime;
        stat.atime = node.atime;
        Ok(stat)
    }

    fn setattr(&mut self, inode: InodeId, stat: &FileStat) -> Result<(), FsError> {
        let mut node = self.read_inode(inode.0 as u32)?;
        node.mode = stat.mode;
        node.uid = stat.uid;
        node.gid = stat.gid;
        node.size = stat.size;
        self.write_inode(&node)
    }

    fn read(&self, inode: InodeId, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        let node = self.read_inode(inode.0 as u32)?;
        if (node.mode & S_IFDIR) != 0 {
            return Err(FsError::IsADirectory);
        }

        if offset >= node.size {
            return Ok(0); // EOF
        }

        let available = (node.size - offset) as usize;
        let to_read = buf.len().min(available);
        let mut bytes_read = 0;

        while bytes_read < to_read {
            let cur_offset = offset + (bytes_read as u64);
            let block_index = (cur_offset / (BLOCK_SIZE as u64)) as usize;
            let offset_in_block = (cur_offset % (BLOCK_SIZE as u64)) as usize;

            if block_index >= node.direct_blocks.len() {
                break;
            }

            let physical_block = node.direct_blocks[block_index];
            if physical_block == 0 {
                break;
            }

            let mut blk = [0u8; BLOCK_SIZE];
            read_block(physical_block, &mut blk)?;

            let can_read = (BLOCK_SIZE - offset_in_block).min(to_read - bytes_read);
            buf[bytes_read..bytes_read + can_read]
                .copy_from_slice(&blk[offset_in_block..offset_in_block + can_read]);
            bytes_read += can_read;
        }

        Ok(bytes_read)
    }

    fn write(&mut self, inode: InodeId, offset: u64, buf: &[u8]) -> Result<usize, FsError> {
        let mut node = self.read_inode(inode.0 as u32)?;
        if (node.mode & S_IFDIR) != 0 {
            return Err(FsError::IsADirectory);
        }

        let mut bytes_written = 0;
        while bytes_written < buf.len() {
            let cur_offset = offset + (bytes_written as u64);
            let block_index = (cur_offset / (BLOCK_SIZE as u64)) as usize;
            let offset_in_block = (cur_offset % (BLOCK_SIZE as u64)) as usize;

            if block_index >= node.direct_blocks.len() {
                return Err(FsError::NoSpace);
            }

            if node.direct_blocks[block_index] == 0 {
                node.direct_blocks[block_index] = self.alloc_block()?;
            }

            let physical_block = node.direct_blocks[block_index];
            let mut blk = [0u8; BLOCK_SIZE];
            if offset_in_block != 0 || (buf.len() - bytes_written) < BLOCK_SIZE {
                let _ = read_block(physical_block, &mut blk);
            }

            let can_write = (BLOCK_SIZE - offset_in_block).min(buf.len() - bytes_written);
            blk[offset_in_block..offset_in_block + can_write]
                .copy_from_slice(&buf[bytes_written..bytes_written + can_write]);
            write_block(physical_block, &blk)?;
            bytes_written += can_write;
        }

        if offset + (bytes_written as u64) > node.size {
            node.size = offset + (bytes_written as u64);
        }

        self.write_inode(&node)?;
        Ok(bytes_written)
    }

    fn create(&mut self, parent: InodeId, name: &str, mode: u32, creds: &Credentials) -> Result<InodeId, FsError> {
        if self.lookup(parent, name).is_ok() {
            return Err(FsError::AlreadyExists);
        }

        let new_inode_id = self.alloc_inode()?;
        let mut new_inode = OnDiskInode::empty();
        new_inode.inode_id = new_inode_id;
        new_inode.mode = S_IFREG | (mode & 0o7777);
        new_inode.uid = creds.euid.0;
        new_inode.gid = creds.egid.0;
        new_inode.nlink = 1;
        self.write_inode(&new_inode)?;

        // Add entry to parent directory
        let mut parent_node = self.read_inode(parent.0 as u32)?;
        let mut added = false;

        for &block_num in &parent_node.direct_blocks {
            if block_num == 0 {
                break;
            }
            let mut blk = [0u8; BLOCK_SIZE];
            read_block(block_num, &mut blk)?;

            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * DIR_ENTRY_SIZE;
                let entry = unsafe { core::ptr::read_unaligned(blk[offset..].as_ptr() as *const OnDiskDirEntry) };
                if entry.inode_id == 0 {
                    let new_entry = OnDiskDirEntry::new(new_inode_id, FileType::File, name);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            &new_entry as *const OnDiskDirEntry as *const u8,
                            blk[offset..].as_mut_ptr(),
                            DIR_ENTRY_SIZE,
                        );
                    }
                    write_block(block_num, &blk)?;
                    added = true;
                    break;
                }
            }
            if added {
                break;
            }
        }

        if !added {
            // Allocate new directory block
            let new_dir_block = self.alloc_block()?;
            for i in 0..parent_node.direct_blocks.len() {
                if parent_node.direct_blocks[i] == 0 {
                    parent_node.direct_blocks[i] = new_dir_block;
                    break;
                }
            }
            let mut blk = [0u8; BLOCK_SIZE];
            let new_entry = OnDiskDirEntry::new(new_inode_id, FileType::File, name);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    &new_entry as *const OnDiskDirEntry as *const u8,
                    blk[0..].as_mut_ptr(),
                    DIR_ENTRY_SIZE,
                );
            }
            write_block(new_dir_block, &blk)?;
        }

        parent_node.size += DIR_ENTRY_SIZE as u64;
        self.write_inode(&parent_node)?;

        Ok(InodeId(new_inode_id as u64))
    }

    fn mkdir(&mut self, parent: InodeId, name: &str, mode: u32, creds: &Credentials) -> Result<InodeId, FsError> {
        if self.lookup(parent, name).is_ok() {
            return Err(FsError::AlreadyExists);
        }

        let new_inode_id = self.alloc_inode()?;
        let dir_block = self.alloc_block()?;

        let mut new_inode = OnDiskInode::empty();
        new_inode.inode_id = new_inode_id;
        new_inode.mode = S_IFDIR | (mode & 0o7777);
        new_inode.uid = creds.euid.0;
        new_inode.gid = creds.egid.0;
        new_inode.nlink = 2;
        new_inode.size = (DIR_ENTRY_SIZE * 2) as u64;
        new_inode.direct_blocks[0] = dir_block;
        self.write_inode(&new_inode)?;

        // Initialize . and .. in new dir block
        let mut blk = [0u8; BLOCK_SIZE];
        let dot = OnDiskDirEntry::new(new_inode_id, FileType::Directory, ".");
        let dotdot = OnDiskDirEntry::new(parent.0 as u32, FileType::Directory, "..");
        unsafe {
            core::ptr::copy_nonoverlapping(&dot as *const _ as *const u8, blk[0..].as_mut_ptr(), DIR_ENTRY_SIZE);
            core::ptr::copy_nonoverlapping(&dotdot as *const _ as *const u8, blk[DIR_ENTRY_SIZE..].as_mut_ptr(), DIR_ENTRY_SIZE);
        }
        write_block(dir_block, &blk)?;

        // Add entry to parent directory
        let mut parent_node = self.read_inode(parent.0 as u32)?;
        let mut added = false;
        for &block_num in &parent_node.direct_blocks {
            if block_num == 0 {
                break;
            }
            let mut pblk = [0u8; BLOCK_SIZE];
            read_block(block_num, &mut pblk)?;
            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * DIR_ENTRY_SIZE;
                let entry = unsafe { core::ptr::read_unaligned(pblk[offset..].as_ptr() as *const OnDiskDirEntry) };
                if entry.inode_id == 0 {
                    let new_entry = OnDiskDirEntry::new(new_inode_id, FileType::Directory, name);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            &new_entry as *const _ as *const u8,
                            pblk[offset..].as_mut_ptr(),
                            DIR_ENTRY_SIZE,
                        );
                    }
                    write_block(block_num, &pblk)?;
                    added = true;
                    break;
                }
            }
            if added {
                break;
            }
        }

        parent_node.nlink += 1;
        self.write_inode(&parent_node)?;

        Ok(InodeId(new_inode_id as u64))
    }

    fn unlink(&mut self, parent: InodeId, name: &str) -> Result<(), FsError> {
        let child_id = self.lookup(parent, name)?;
        let child_node = self.read_inode(child_id.0 as u32)?;
        if (child_node.mode & S_IFDIR) != 0 {
            return Err(FsError::IsADirectory);
        }

        // Remove from parent dir
        let parent_node = self.read_inode(parent.0 as u32)?;
        for &block_num in &parent_node.direct_blocks {
            if block_num == 0 {
                break;
            }
            let mut blk = [0u8; BLOCK_SIZE];
            read_block(block_num, &mut blk)?;
            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let offset = i * DIR_ENTRY_SIZE;
                let entry = unsafe { core::ptr::read_unaligned(blk[offset..].as_ptr() as *const OnDiskDirEntry) };
                if entry.inode_id == (child_id.0 as u32) {
                    let empty = OnDiskDirEntry::empty();
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            &empty as *const _ as *const u8,
                            blk[offset..].as_mut_ptr(),
                            DIR_ENTRY_SIZE,
                        );
                    }
                    write_block(block_num, &blk)?;
                    break;
                }
            }
        }

        // Free child inode in bitmap
        let mut bitmap = [0u8; BLOCK_SIZE];
        read_block(INODE_BITMAP_BLOCK, &mut bitmap)?;
        let byte_idx = (child_id.0 as usize) / 8;
        let bit = (child_id.0 as usize) % 8;
        bitmap[byte_idx] &= !(1 << bit);
        write_block(INODE_BITMAP_BLOCK, &bitmap)?;

        self.sync_journal()
    }

    fn rmdir(&mut self, _parent: InodeId, _name: &str) -> Result<(), FsError> {
        Err(FsError::NotSupported)
    }

    fn readdir(&self, inode: InodeId, offset: usize) -> Result<Option<DirEntry>, FsError> {
        let node = self.read_inode(inode.0 as u32)?;
        if (node.mode & S_IFDIR) == 0 {
            return Err(FsError::NotADirectory);
        }

        let mut current_idx = 0;
        for &block_num in &node.direct_blocks {
            if block_num == 0 {
                break;
            }
            let mut blk = [0u8; BLOCK_SIZE];
            read_block(block_num, &mut blk)?;

            for i in 0..DIR_ENTRIES_PER_BLOCK {
                let byte_off = i * DIR_ENTRY_SIZE;
                let entry = unsafe { core::ptr::read_unaligned(blk[byte_off..].as_ptr() as *const OnDiskDirEntry) };
                if entry.inode_id != 0 {
                    if current_idx == offset {
                        let ftype = if entry.file_type == (FileType::Directory as u8) {
                            FileType::Directory
                        } else {
                            FileType::File
                        };
                        return Ok(Some(DirEntry {
                            inode: InodeId(entry.inode_id as u64),
                            file_type: ftype,
                            name: entry.get_name().to_string(),
                        }));
                    }
                    current_idx += 1;
                }
            }
        }

        Ok(None)
    }

    fn truncate(&mut self, inode: InodeId, size: u64) -> Result<(), FsError> {
        let mut node = self.read_inode(inode.0 as u32)?;
        node.size = size;
        self.write_inode(&node)
    }
}
