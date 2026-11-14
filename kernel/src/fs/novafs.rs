//! NovaOS Zero — Native Read/Write Disk Filesystem (NovaFS)
//! Implements on-disk superblock, block allocation bitmap, 64-byte inode table, and directory indexing.

use crate::storage::BlockDevice;
use super::vfs::{FileOperations, FsError, InodeMetadata, NodeType, MAX_NAME_LEN};

pub const NOVA_MAGIC: [u8; 4] = [0x4E, 0x4F, 0x56, 0x41]; // "NOVA"
pub const NOVA_VERSION: u32 = 1;
pub const BLOCK_SIZE: usize = 512;
pub const INODE_COUNT: usize = 32;
pub const INODES_PER_BLOCK: usize = BLOCK_SIZE / 64; // 8
pub const INODE_TABLE_BLOCKS: usize = INODE_COUNT / INODES_PER_BLOCK; // 4 blocks
pub const DENTRY_SIZE: usize = 32;
pub const DENTRIES_PER_BLOCK: usize = BLOCK_SIZE / DENTRY_SIZE; // 16 entries
pub const DIRECT_BLOCKS: usize = 12;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Superblock {
    pub magic: [u8; 4],
    pub version: u32,
    pub block_size: u32,
    pub total_blocks: u32,
    pub inode_count: u32,
    pub bitmap_lba: u32,
    pub inode_table_lba: u32,
    pub data_lba: u32,
    pub free_blocks: u32,
    pub free_inodes: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct DiskInode {
    pub mode: u16,        // 0 = free, 1 = file, 2 = directory
    pub permissions: u16,
    pub size: u32,
    pub direct_blocks: [u32; DIRECT_BLOCKS],
    pub indirect_block: u32,
    pub pad: [u8; 8],     // 2 + 2 + 4 + 48 + 4 + 8 = 68 -> adjust for exactly 64 bytes
}

// Exactly 64-byte on-disk Inode definition
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Inode64 {
    pub mode: u16,        // 0 = Unused, 1 = File, 2 = Directory
    pub permissions: u16,
    pub size: u32,
    pub direct_blocks: [u32; DIRECT_BLOCKS], // 48 bytes
    pub indirect_block: u32,                 // 4 bytes
    pub reserved: u32,                       // 4 bytes -> total 64 bytes
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct DirEntry {
    pub inode: u32,
    pub file_type: u8,
    pub name_len: u8,
    pub name: [u8; 26],
}

pub struct NovaFs<B: BlockDevice> {
    pub device: B,
    pub superblock: Superblock,
}

impl<B: BlockDevice> NovaFs<B> {
    /// Format a raw block device with a fresh NovaFS volume
    pub fn format(mut device: B) -> Result<Self, FsError> {
        let total_blocks = device.total_sectors() as u32;
        if total_blocks < 32 {
            return Err(FsError::DeviceError);
        }

        let bitmap_lba = 1u32;
        let inode_table_lba = 2u32;
        let data_lba = inode_table_lba + INODE_TABLE_BLOCKS as u32;
        let free_blocks = total_blocks - data_lba - 1; // 1 block allocated for root dir

        let sb = Superblock {
            magic: NOVA_MAGIC,
            version: NOVA_VERSION,
            block_size: BLOCK_SIZE as u32,
            total_blocks,
            inode_count: INODE_COUNT as u32,
            bitmap_lba,
            inode_table_lba,
            data_lba,
            free_blocks,
            free_inodes: (INODE_COUNT - 1) as u32, // Inode 1 reserved for root
        };

        // 1. Write Superblock to LBA 0
        let mut block = [0u8; BLOCK_SIZE];
        unsafe {
            core::ptr::copy_nonoverlapping(
                &sb as *const Superblock as *const u8,
                block.as_mut_ptr(),
                core::mem::size_of::<Superblock>(),
            );
        }
        device.write_sector(0, &block).map_err(|_| FsError::DeviceError)?;

        // 2. Initialize Block Allocation Bitmap (LBA 1)
        // Mark metadata blocks (0..data_lba) as allocated
        block = [0u8; BLOCK_SIZE];
        let reserved_blocks = data_lba + 1; // +1 for root directory data block
        for i in 0..reserved_blocks {
            let byte_idx = (i / 8) as usize;
            let bit_idx = i % 8;
            block[byte_idx] |= 1 << bit_idx;
        }
        device.write_sector(bitmap_lba as u64, &block).map_err(|_| FsError::DeviceError)?;

        // 3. Initialize Inode Table (LBAs 2..5)
        for lba in inode_table_lba..(inode_table_lba + INODE_TABLE_BLOCKS as u32) {
            block = [0u8; BLOCK_SIZE];
            device.write_sector(lba as u64, &block).map_err(|_| FsError::DeviceError)?;
        }

        // 4. Initialize Root Directory Inode (Inode 1)
        let root_inode = Inode64 {
            mode: 2, // Directory
            permissions: 0o755,
            size: (DENTRY_SIZE * 2) as u32, // '.' and '..'
            direct_blocks: {
                let mut db = [0u32; DIRECT_BLOCKS];
                db[0] = data_lba;
                db
            },
            indirect_block: 0,
            reserved: 0,
        };

        // Write Root Inode into Inode Table Block 0 (Inode 1 is index 1)
        device.read_sector(inode_table_lba as u64, &mut block).map_err(|_| FsError::DeviceError)?;
        unsafe {
            let dest = block.as_mut_ptr().add(core::mem::size_of::<Inode64>());
            core::ptr::copy_nonoverlapping(
                &root_inode as *const Inode64 as *const u8,
                dest,
                core::mem::size_of::<Inode64>(),
            );
        }
        device.write_sector(inode_table_lba as u64, &block).map_err(|_| FsError::DeviceError)?;

        // 5. Initialize Root Directory Data Block with '.' and '..'
        block = [0u8; BLOCK_SIZE];
        let mut dot = DirEntry {
            inode: 1,
            file_type: 2,
            name_len: 1,
            name: [0u8; 26],
        };
        dot.name[0] = b'.';

        let mut dotdot = DirEntry {
            inode: 1,
            file_type: 2,
            name_len: 2,
            name: [0u8; 26],
        };
        dotdot.name[0] = b'.';
        dotdot.name[1] = b'.';

        unsafe {
            core::ptr::copy_nonoverlapping(
                &dot as *const DirEntry as *const u8,
                block.as_mut_ptr(),
                DENTRY_SIZE,
            );
            core::ptr::copy_nonoverlapping(
                &dotdot as *const DirEntry as *const u8,
                block.as_mut_ptr().add(DENTRY_SIZE),
                DENTRY_SIZE,
            );
        }
        device.write_sector(data_lba as u64, &block).map_err(|_| FsError::DeviceError)?;

        Ok(NovaFs { device, superblock: sb })
    }

    /// Read an inode from the inode table
    pub fn read_inode(&mut self, inode_num: u32) -> Result<Inode64, FsError> {
        if inode_num == 0 || inode_num as usize >= INODE_COUNT {
            return Err(FsError::NotFound);
        }

        let block_idx = (inode_num as usize) / INODES_PER_BLOCK;
        let entry_idx = (inode_num as usize) % INODES_PER_BLOCK;
        let lba = self.superblock.inode_table_lba as u64 + block_idx as u64;

        let mut buf = [0u8; BLOCK_SIZE];
        self.device.read_sector(lba, &mut buf).map_err(|_| FsError::DeviceError)?;

        let inode = unsafe {
            let src = buf.as_ptr().add(entry_idx * core::mem::size_of::<Inode64>());
            &*(src as *const Inode64)
        };

        if inode.mode == 0 {
            return Err(FsError::NotFound);
        }

        Ok(*inode)
    }

    /// Write an inode back into the inode table
    pub fn write_inode(&mut self, inode_num: u32, inode: &Inode64) -> Result<(), FsError> {
        if inode_num == 0 || inode_num as usize >= INODE_COUNT {
            return Err(FsError::NotFound);
        }

        let block_idx = (inode_num as usize) / INODES_PER_BLOCK;
        let entry_idx = (inode_num as usize) % INODES_PER_BLOCK;
        let lba = self.superblock.inode_table_lba as u64 + block_idx as u64;

        let mut buf = [0u8; BLOCK_SIZE];
        self.device.read_sector(lba, &mut buf).map_err(|_| FsError::DeviceError)?;

        unsafe {
            let dest = buf.as_mut_ptr().add(entry_idx * core::mem::size_of::<Inode64>());
            core::ptr::copy_nonoverlapping(
                inode as *const Inode64 as *const u8,
                dest,
                core::mem::size_of::<Inode64>(),
            );
        }

        self.device.write_sector(lba, &buf).map_err(|_| FsError::DeviceError)?;
        Ok(())
    }

    /// Allocate a free block from the bitmap
    pub fn alloc_block(&mut self) -> Result<u32, FsError> {
        let mut bitmap = [0u8; BLOCK_SIZE];
        self.device.read_sector(self.superblock.bitmap_lba as u64, &mut bitmap)
            .map_err(|_| FsError::DeviceError)?;

        for block_id in self.superblock.data_lba..self.superblock.total_blocks {
            let byte_idx = (block_id / 8) as usize;
            let bit_idx = block_id % 8;

            if (bitmap[byte_idx] & (1 << bit_idx)) == 0 {
                bitmap[byte_idx] |= 1 << bit_idx;
                self.device.write_sector(self.superblock.bitmap_lba as u64, &bitmap)
                    .map_err(|_| FsError::DeviceError)?;
                
                // Zero out the newly allocated block
                let zero = [0u8; BLOCK_SIZE];
                self.device.write_sector(block_id as u64, &zero)
                    .map_err(|_| FsError::DeviceError)?;

                return Ok(block_id);
            }
        }

        Err(FsError::NoSpace)
    }

    /// Allocate a free inode number
    pub fn alloc_inode(&mut self, mode: u16) -> Result<u32, FsError> {
        for inode_num in 2..INODE_COUNT as u32 {
            let block_idx = (inode_num as usize) / INODES_PER_BLOCK;
            let entry_idx = (inode_num as usize) % INODES_PER_BLOCK;
            let lba = self.superblock.inode_table_lba as u64 + block_idx as u64;

            let mut buf = [0u8; BLOCK_SIZE];
            self.device.read_sector(lba, &mut buf).map_err(|_| FsError::DeviceError)?;

            let inode = unsafe {
                let src = buf.as_ptr().add(entry_idx * core::mem::size_of::<Inode64>());
                &*(src as *const Inode64)
            };

            if inode.mode == 0 {
                let new_inode = Inode64 {
                    mode,
                    permissions: 0o644,
                    size: 0,
                    direct_blocks: [0; DIRECT_BLOCKS],
                    indirect_block: 0,
                    reserved: 0,
                };
                self.write_inode(inode_num, &new_inode)?;
                return Ok(inode_num);
            }
        }

        Err(FsError::NoSpace)
    }
}

impl<B: BlockDevice + Send + Sync> FileOperations for NovaFs<B> {
    fn read(&mut self, inode: u32, offset: u32, buf: &mut [u8]) -> Result<usize, FsError> {
        self.read_file_data(inode, offset, buf)
    }

    fn write(&mut self, inode: u32, offset: u32, buf: &[u8]) -> Result<usize, FsError> {
        let mut in_node = self.read_inode(inode)?;
        if in_node.mode != 1 {
            return Err(FsError::IsADirectory);
        }

        let mut bytes_written = 0;
        let mut cur_offset = offset as usize;

        while bytes_written < buf.len() {
            let block_idx = cur_offset / BLOCK_SIZE;
            let block_offset = cur_offset % BLOCK_SIZE;
            let chunk_len = (buf.len() - bytes_written).min(BLOCK_SIZE - block_offset);

            if block_idx >= DIRECT_BLOCKS {
                return Err(FsError::NoSpace);
            }

            // Allocate block if not present
            let block_lba = if in_node.direct_blocks[block_idx] == 0 {
                let new_blk = self.alloc_block()?;
                in_node.direct_blocks[block_idx] = new_blk;
                new_blk
            } else {
                in_node.direct_blocks[block_idx]
            };

            let mut data_buf = [0u8; BLOCK_SIZE];
            self.device.read_sector(block_lba as u64, &mut data_buf)
                .map_err(|_| FsError::DeviceError)?;

            data_buf[block_offset..block_offset + chunk_len]
                .copy_from_slice(&buf[bytes_written..bytes_written + chunk_len]);

            self.device.write_sector(block_lba as u64, &data_buf)
                .map_err(|_| FsError::DeviceError)?;

            bytes_written += chunk_len;
            cur_offset += chunk_len;
        }

        if cur_offset as u32 > in_node.size {
            in_node.size = cur_offset as u32;
        }
        self.write_inode(inode, &in_node)?;

        Ok(bytes_written)
    }

    fn create(&mut self, parent_inode: u32, name: &str, node_type: NodeType) -> Result<u32, FsError> {
        let mut parent = self.read_inode(parent_inode)?;
        if parent.mode != 2 {
            return Err(FsError::NotADirectory);
        }

        // Check if name already exists
        if self.lookup(parent_inode, name).is_ok() {
            return Err(FsError::AlreadyExists);
        }

        let mode = match node_type {
            NodeType::File => 1,
            NodeType::Directory => 2,
            _ => 1,
        };

        let new_inode_num = self.alloc_inode(mode)?;

        // If directory, allocate a data block for '.' and '..'
        if mode == 2 {
            let dir_blk = self.alloc_block()?;
            let mut new_dir_node = self.read_inode(new_inode_num)?;
            new_dir_node.direct_blocks[0] = dir_blk;
            new_dir_node.size = (DENTRY_SIZE * 2) as u32;
            self.write_inode(new_inode_num, &new_dir_node)?;

            let mut block = [0u8; BLOCK_SIZE];
            let mut dot = DirEntry { inode: new_inode_num, file_type: 2, name_len: 1, name: [0; 26] };
            dot.name[0] = b'.';
            let mut dotdot = DirEntry { inode: parent_inode, file_type: 2, name_len: 2, name: [0; 26] };
            dotdot.name[0] = b'.'; dotdot.name[1] = b'.';

            unsafe {
                core::ptr::copy_nonoverlapping(&dot as *const _ as *const u8, block.as_mut_ptr(), DENTRY_SIZE);
                core::ptr::copy_nonoverlapping(&dotdot as *const _ as *const u8, block.as_mut_ptr().add(DENTRY_SIZE), DENTRY_SIZE);
            }
            self.device.write_sector(dir_blk as u64, &block).map_err(|_| FsError::DeviceError)?;
        }

        // Insert into parent directory entries
        let mut dentry_blk = parent.direct_blocks[0];
        if dentry_blk == 0 {
            dentry_blk = self.alloc_block()?;
            parent.direct_blocks[0] = dentry_blk;
        }

        let mut block = [0u8; BLOCK_SIZE];
        self.device.read_sector(dentry_blk as u64, &mut block).map_err(|_| FsError::DeviceError)?;

        let mut inserted = false;
        for i in 0..DENTRIES_PER_BLOCK {
            let offset = i * DENTRY_SIZE;
            let dentry = unsafe { &*(block.as_ptr().add(offset) as *const DirEntry) };
            if dentry.inode == 0 {
                let mut new_entry = DirEntry {
                    inode: new_inode_num,
                    file_type: mode as u8,
                    name_len: name.len() as u8,
                    name: [0u8; 26],
                };
                let copy_len = name.len().min(26);
                new_entry.name[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);

                unsafe {
                    core::ptr::copy_nonoverlapping(
                        &new_entry as *const _ as *const u8,
                        block.as_mut_ptr().add(offset),
                        DENTRY_SIZE,
                    );
                }
                inserted = true;
                break;
            }
        }

        if !inserted {
            return Err(FsError::NoSpace);
        }

        self.device.write_sector(dentry_blk as u64, &block).map_err(|_| FsError::DeviceError)?;
        parent.size += DENTRY_SIZE as u32;
        self.write_inode(parent_inode, &parent)?;

        Ok(new_inode_num)
    }

    fn lookup(&mut self, parent_inode: u32, name: &str) -> Result<u32, FsError> {
        let parent = self.read_inode(parent_inode)?;
        if parent.mode != 2 {
            return Err(FsError::NotADirectory);
        }

        let dentry_blk = parent.direct_blocks[0];
        if dentry_blk == 0 {
            return Err(FsError::NotFound);
        }

        let mut block = [0u8; BLOCK_SIZE];
        self.device.read_sector(dentry_blk as u64, &mut block).map_err(|_| FsError::DeviceError)?;

        for i in 0..DENTRIES_PER_BLOCK {
            let offset = i * DENTRY_SIZE;
            let dentry = unsafe { &*(block.as_ptr().add(offset) as *const DirEntry) };
            if dentry.inode != 0 {
                let len = dentry.name_len as usize;
                if len == name.len() && &dentry.name[..len] == name.as_bytes() {
                    return Ok(dentry.inode);
                }
            }
        }

        Err(FsError::NotFound)
    }

    fn get_metadata(&mut self, inode: u32) -> Result<InodeMetadata, FsError> {
        let node = self.read_inode(inode)?;
        Ok(InodeMetadata {
            inode_number: inode,
            node_type: match node.mode {
                1 => NodeType::File,
                2 => NodeType::Directory,
                _ => NodeType::File,
            },
            size: node.size,
            permissions: node.permissions,
            blocks: (node.size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32,
        })
    }

    fn list_dir(&mut self, parent_inode: u32, out_names: &mut [[u8; MAX_NAME_LEN]], max_entries: usize) -> Result<usize, FsError> {
        let parent = self.read_inode(parent_inode)?;
        if parent.mode != 2 {
            return Err(FsError::NotADirectory);
        }

        let dentry_blk = parent.direct_blocks[0];
        if dentry_blk == 0 {
            return Ok(0);
        }

        let mut block = [0u8; BLOCK_SIZE];
        self.device.read_sector(dentry_blk as u64, &mut block).map_err(|_| FsError::DeviceError)?;

        let mut count = 0;
        for i in 0..DENTRIES_PER_BLOCK {
            if count >= max_entries {
                break;
            }
            let offset = i * DENTRY_SIZE;
            let dentry = unsafe { &*(block.as_ptr().add(offset) as *const DirEntry) };
            if dentry.inode != 0 {
                let len = (dentry.name_len as usize).min(MAX_NAME_LEN);
                out_names[count][..len].copy_from_slice(&dentry.name[..len]);
                count += 1;
            }
        }

        Ok(count)
    }
}

impl<B: BlockDevice> NovaFs<B> {
    /// Read data from a file inode
    pub fn read_file_data(&mut self, inode: u32, offset: u32, buf: &mut [u8]) -> Result<usize, FsError> {
        let in_node = self.read_inode(inode)?;
        if in_node.mode != 1 {
            return Err(FsError::IsADirectory);
        }

        if offset >= in_node.size {
            return Ok(0); // EOF
        }

        let max_read = (in_node.size - offset) as usize;
        let to_read = buf.len().min(max_read);
        let mut bytes_read = 0;
        let mut cur_offset = offset as usize;

        while bytes_read < to_read {
            let block_idx = cur_offset / BLOCK_SIZE;
            let block_offset = cur_offset % BLOCK_SIZE;
            let chunk_len = (to_read - bytes_read).min(BLOCK_SIZE - block_offset);

            if block_idx >= DIRECT_BLOCKS {
                break;
            }

            let block_lba = in_node.direct_blocks[block_idx];
            if block_lba == 0 {
                // Sparse hole: return zeroes
                buf[bytes_read..bytes_read + chunk_len].fill(0);
            } else {
                let mut data_buf = [0u8; BLOCK_SIZE];
                self.device.read_sector(block_lba as u64, &mut data_buf)
                    .map_err(|_| FsError::DeviceError)?;
                buf[bytes_read..bytes_read + chunk_len]
                    .copy_from_slice(&data_buf[block_offset..block_offset + chunk_len]);
            }

            bytes_read += chunk_len;
            cur_offset += chunk_len;
        }

        Ok(bytes_read)
    }
}
