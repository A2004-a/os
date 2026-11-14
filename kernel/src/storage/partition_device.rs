//! NovaOS Apex — Partition Device Slice
//! Maps logical partition sectors to physical device LBAs and enforces strict boundary isolation.

use crate::drivers::virtio_block::{read_sector, write_sector, SECTOR_SIZE};
use crate::drivers::DriverError;

#[derive(Clone, Copy, Debug)]
pub struct PartitionDevice {
    pub id: u32,
    pub start_lba: u64,
    pub end_lba: u64,
    pub type_guid: [u8; 16],
    pub read_only: bool,
}

impl PartitionDevice {
    pub fn new(id: u32, start_lba: u64, end_lba: u64, type_guid: [u8; 16], read_only: bool) -> Self {
        Self {
            id,
            start_lba,
            end_lba,
            type_guid,
            read_only,
        }
    }

    pub fn capacity_sectors(&self) -> u64 {
        if self.end_lba >= self.start_lba {
            self.end_lba - self.start_lba + 1
        } else {
            0
        }
    }

    pub fn read_partition_sector(&self, local_lba: u64, buf: &mut [u8; SECTOR_SIZE]) -> Result<(), DriverError> {
        if local_lba >= self.capacity_sectors() {
            return Err(DriverError::IoError);
        }
        let physical_lba = self.start_lba + local_lba;
        read_sector(physical_lba, buf)
    }

    pub fn write_partition_sector(&self, local_lba: u64, buf: &[u8; SECTOR_SIZE]) -> Result<(), DriverError> {
        if self.read_only {
            return Err(DriverError::IoError); // Disallowed on read-only partition
        }
        if local_lba >= self.capacity_sectors() {
            return Err(DriverError::IoError);
        }
        let physical_lba = self.start_lba + local_lba;
        write_sector(physical_lba, buf)
    }
}
