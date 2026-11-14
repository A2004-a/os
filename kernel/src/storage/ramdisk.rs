//! NovaOS Zero — In-Memory RAM Disk Block Device (512-byte sector abstraction)

use super::BlockDevice;

pub const RAMDISK_SECTORS: usize = 512; // 512 sectors * 512 bytes = 256 KiB
pub const SECTOR_SIZE: usize = 512;

pub struct RamBlockDevice {
    storage: [[u8; SECTOR_SIZE]; RAMDISK_SECTORS],
}

impl RamBlockDevice {
    pub const fn new() -> Self {
        Self {
            storage: [[0u8; SECTOR_SIZE]; RAMDISK_SECTORS],
        }
    }
}

impl BlockDevice for RamBlockDevice {
    fn sector_size(&self) -> usize {
        SECTOR_SIZE
    }

    fn total_sectors(&self) -> u64 {
        RAMDISK_SECTORS as u64
    }

    fn read_sector(&mut self, lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str> {
        let index = lba as usize;
        if index >= RAMDISK_SECTORS {
            return Err("LBA out of bounds");
        }
        buf.copy_from_slice(&self.storage[index]);
        Ok(())
    }

    fn write_sector(&mut self, lba: u64, buf: &[u8; 512]) -> Result<(), &'static str> {
        let index = lba as usize;
        if index >= RAMDISK_SECTORS {
            return Err("LBA out of bounds");
        }
        self.storage[index].copy_from_slice(buf);
        Ok(())
    }
}
