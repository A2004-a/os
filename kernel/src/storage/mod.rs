//! NovaOS Storage Subsystem
//! Block device abstraction and VirtIO Block driver interface

pub trait BlockDevice {
    fn read_sector(&mut self, lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str>;
    fn write_sector(&mut self, lba: u64, buf: &[u8; 512]) -> Result<(), &'static str>;
}

pub struct VirtioBlockDevice {
    pub io_base: u16,
    pub capacity_sectors: u64,
}

impl VirtioBlockDevice {
    pub const fn new(io_base: u16) -> Self {
        VirtioBlockDevice {
            io_base,
            capacity_sectors: 0,
        }
    }
}

impl BlockDevice for VirtioBlockDevice {
    fn read_sector(&mut self, _lba: u64, _buf: &mut [u8; 512]) -> Result<(), &'static str> {
        // VirtIO block read queue operation
        Ok(())
    }

    fn write_sector(&mut self, _lba: u64, _buf: &[u8; 512]) -> Result<(), &'static str> {
        // VirtIO block write queue operation
        Ok(())
    }
}
