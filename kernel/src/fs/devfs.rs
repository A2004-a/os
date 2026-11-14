//! NovaOS Apex — Device Filesystem (DevFS)
//! Provides character and block device nodes mapped under `/dev`.

extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::drivers::console;
use crate::drivers::virtio_block::{read_sector, write_sector, SECTOR_SIZE};
use crate::fs::types::*;
use crate::fs::vfs::FileSystem;
use crate::security::*;
use crate::serial;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    Null = 2,
    Zero = 3,
    Random = 4,
    Console = 5,
    Serial = 6,
    Vda = 7,
    Fb0 = 8,
}

pub struct DevFs {
    devices: Vec<(&'static str, DeviceKind, FileType, u32)>,
    prng_seed: u64,
}

impl DevFs {
    pub fn new() -> Self {
        let mut dev = Self {
            devices: Vec::new(),
            prng_seed: 0x9E37_79B9_7F4A_7C15,
        };

        dev.devices.push(("null", DeviceKind::Null, FileType::CharDevice, 0o666 | S_IFCHR));
        dev.devices.push(("zero", DeviceKind::Zero, FileType::CharDevice, 0o666 | S_IFCHR));
        dev.devices.push(("random", DeviceKind::Random, FileType::CharDevice, 0o666 | S_IFCHR));
        dev.devices.push(("urandom", DeviceKind::Random, FileType::CharDevice, 0o666 | S_IFCHR));
        dev.devices.push(("console", DeviceKind::Console, FileType::CharDevice, 0o666 | S_IFCHR));
        dev.devices.push(("serial", DeviceKind::Serial, FileType::CharDevice, 0o666 | S_IFCHR));
        dev.devices.push(("vda", DeviceKind::Vda, FileType::BlockDevice, 0o660 | S_IFBLK));
        dev.devices.push(("fb0", DeviceKind::Fb0, FileType::CharDevice, 0o660 | S_IFCHR));

        dev
    }

    #[allow(dead_code)]
    fn next_rand(&mut self) -> u8 {
        let mut x = self.prng_seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.prng_seed = x;
        (x & 0xFF) as u8
    }
}

impl FileSystem for DevFs {
    fn root_inode(&self) -> InodeId {
        InodeId(1)
    }

    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError> {
        if parent != InodeId(1) {
            return Err(FsError::NotFound);
        }
        if name == "." || name == ".." {
            return Ok(InodeId(1));
        }

        for (dev_name, kind, _, _) in &self.devices {
            if *dev_name == name {
                return Ok(InodeId(*kind as u64));
            }
        }
        Err(FsError::NotFound)
    }

    fn getattr(&self, inode: InodeId) -> Result<FileStat, FsError> {
        if inode == InodeId(1) {
            let mut stat = FileStat::empty();
            stat.inode = 1;
            stat.file_type = FileType::Directory as u32;
            stat.mode = S_IFDIR | 0o755;
            stat.nlink = 2;
            return Ok(stat);
        }

        for (_, kind, ftype, mode) in &self.devices {
            if (*kind as u64) == inode.0 {
                let mut stat = FileStat::empty();
                stat.inode = inode.0;
                stat.file_type = *ftype as u32;
                stat.mode = *mode;
                stat.nlink = 1;
                if *kind == DeviceKind::Vda {
                    stat.size = crate::drivers::virtio_block::get_capacity_sectors() * (SECTOR_SIZE as u64);
                    stat.blocks = crate::drivers::virtio_block::get_capacity_sectors();
                }
                return Ok(stat);
            }
        }

        Err(FsError::NotFound)
    }

    fn setattr(&mut self, _inode: InodeId, _stat: &FileStat) -> Result<(), FsError> {
        Ok(())
    }

    fn read(&self, inode: InodeId, offset: u64, buf: &mut [u8]) -> Result<usize, FsError> {
        match inode.0 {
            // /dev/null
            2 => Ok(0), // Immediate EOF

            // /dev/zero
            3 => {
                buf.fill(0);
                Ok(buf.len())
            }

            // /dev/random & /dev/urandom
            4 => {
                let mut state = 0xA5A5_5A5A_1234_5678 ^ offset;
                for b in buf.iter_mut() {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    *b = (state & 0xFF) as u8;
                }
                Ok(buf.len())
            }

            // /dev/console & /dev/serial
            5 | 6 => {
                // Non-blocking read from serial COM1 if available
                Ok(0)
            }

            // /dev/vda (block device sector read)
            7 => {
                let sector_idx = offset / (SECTOR_SIZE as u64);
                let mut temp_sec = [0u8; SECTOR_SIZE];
                read_sector(sector_idx, &mut temp_sec).map_err(|_| FsError::IoError)?;
                let sector_offset = (offset % (SECTOR_SIZE as u64)) as usize;
                let available = SECTOR_SIZE - sector_offset;
                let to_copy = buf.len().min(available);
                buf[..to_copy].copy_from_slice(&temp_sec[sector_offset..sector_offset + to_copy]);
                Ok(to_copy)
            }

            // /dev/fb0
            8 => Ok(0),

            _ => Err(FsError::NotFound),
        }
    }

    fn write(&mut self, inode: InodeId, offset: u64, buf: &[u8]) -> Result<usize, FsError> {
        match inode.0 {
            // /dev/null & /dev/zero
            2 | 3 => Ok(buf.len()), // Discard write

            // /dev/random (seed)
            4 => {
                if !buf.is_empty() {
                    self.prng_seed ^= buf[0] as u64;
                }
                Ok(buf.len())
            }

            // /dev/console
            5 => {
                if let Ok(s) = core::str::from_utf8(buf) {
                    console::write_str(s);
                    serial::write_str(s);
                } else {
                    for &b in buf {
                        serial::write_byte(b);
                    }
                }
                Ok(buf.len())
            }

            // /dev/serial
            6 => {
                if let Ok(s) = core::str::from_utf8(buf) {
                    serial::write_str(s);
                } else {
                    for &b in buf {
                        serial::write_byte(b);
                    }
                }
                Ok(buf.len())
            }

            // /dev/vda (block device sector write)
            7 => {
                let sector_idx = offset / (SECTOR_SIZE as u64);
                let sector_offset = (offset % (SECTOR_SIZE as u64)) as usize;
                let mut temp_sec = [0u8; SECTOR_SIZE];
                if sector_offset != 0 || buf.len() < SECTOR_SIZE {
                    let _ = read_sector(sector_idx, &mut temp_sec);
                }
                let available = SECTOR_SIZE - sector_offset;
                let to_copy = buf.len().min(available);
                temp_sec[sector_offset..sector_offset + to_copy].copy_from_slice(&buf[..to_copy]);
                write_sector(sector_idx, &temp_sec).map_err(|_| FsError::IoError)?;
                Ok(to_copy)
            }

            // /dev/fb0
            8 => Ok(buf.len()),

            _ => Err(FsError::NotFound),
        }
    }

    fn create(&mut self, _parent: InodeId, _name: &str, _mode: u32, _creds: &Credentials) -> Result<InodeId, FsError> {
        Err(FsError::PermissionDenied)
    }

    fn mkdir(&mut self, _parent: InodeId, _name: &str, _mode: u32, _creds: &Credentials) -> Result<InodeId, FsError> {
        Err(FsError::PermissionDenied)
    }

    fn unlink(&mut self, _parent: InodeId, _name: &str) -> Result<(), FsError> {
        Err(FsError::PermissionDenied)
    }

    fn rmdir(&mut self, _parent: InodeId, _name: &str) -> Result<(), FsError> {
        Err(FsError::PermissionDenied)
    }

    fn readdir(&self, inode: InodeId, offset: usize) -> Result<Option<DirEntry>, FsError> {
        if inode != InodeId(1) {
            return Err(FsError::NotADirectory);
        }

        if offset >= self.devices.len() {
            return Ok(None);
        }

        let (name, kind, ftype, _) = &self.devices[offset];
        Ok(Some(DirEntry {
            inode: InodeId(*kind as u64),
            file_type: *ftype,
            name: name.to_string(),
        }))
    }

    fn truncate(&mut self, _inode: InodeId, _size: u64) -> Result<(), FsError> {
        Ok(())
    }
}
