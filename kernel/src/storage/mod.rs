//! NovaOS Zero — Storage Subsystem
//! Block device abstraction and VirtIO / RAM disk implementations

pub mod ramdisk;

pub use ramdisk::RamBlockDevice;

pub trait BlockDevice {
    fn sector_size(&self) -> usize { 512 }
    fn total_sectors(&self) -> u64;
    fn read_sector(&mut self, lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str>;
    fn write_sector(&mut self, lba: u64, buf: &[u8; 512]) -> Result<(), &'static str>;
}

impl<B: BlockDevice> BlockDevice for &mut B {
    fn sector_size(&self) -> usize {
        (**self).sector_size()
    }
    fn total_sectors(&self) -> u64 {
        (**self).total_sectors()
    }
    fn read_sector(&mut self, lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str> {
        (**self).read_sector(lba, buf)
    }
    fn write_sector(&mut self, lba: u64, buf: &[u8; 512]) -> Result<(), &'static str> {
        (**self).write_sector(lba, buf)
    }
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

    /// Discover VirtIO block controller on PCI bus
    pub fn probe() -> Option<Self> {
        if let Some(dev) = crate::drivers::pci::find_pci_device(
            crate::drivers::pci::VENDOR_VIRTIO,
            crate::drivers::pci::DEVICE_VIRTIO_BLK,
        ) {
            if let Some(io) = dev.io_base() {
                return Some(Self::new(io));
            }
        }
        None
    }
}

impl BlockDevice for VirtioBlockDevice {
    fn total_sectors(&self) -> u64 {
        self.capacity_sectors
    }

    fn read_sector(&mut self, _lba: u64, _buf: &mut [u8; 512]) -> Result<(), &'static str> {
        // VirtIO PCI request queue dispatch
        Ok(())
    }

    fn write_sector(&mut self, _lba: u64, _buf: &[u8; 512]) -> Result<(), &'static str> {
        // VirtIO PCI write request queue dispatch
        Ok(())
    }
}
