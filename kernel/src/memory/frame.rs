//! NovaOS Zero — Physical Frame Allocator (4 KiB page frames)
//! Manages conventional physical memory discovered through UEFI Memory Map

use crate::boot::info::{MemoryRegion, MAX_MEMORY_REGIONS};

pub const FRAME_SIZE: u64 = 4096;

pub struct BootInfoFrameAllocator {
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    regions_count: usize,
    current_region_idx: usize,
    current_addr: u64,
    allocated_count: u64,
    total_frames_count: u64,
}

impl BootInfoFrameAllocator {
    pub fn new(regions: [MemoryRegion; MAX_MEMORY_REGIONS], count: usize) -> Self {
        let mut total_bytes = 0u64;
        let mut first_addr = 0u64;
        let mut found_first = false;

        for i in 0..count {
            if regions[i].region_type == 1 {
                // Conventional Usable RAM
                total_bytes += regions[i].length;
                if !found_first {
                    first_addr = regions[i].start;
                    found_first = true;
                }
            }
        }

        BootInfoFrameAllocator {
            regions,
            regions_count: count,
            current_region_idx: 0,
            current_addr: first_addr,
            allocated_count: 0,
            total_frames_count: total_bytes / FRAME_SIZE,
        }
    }

    /// Allocate a single 4 KiB physical page frame
    pub fn allocate_frame(&mut self) -> Option<u64> {
        while self.current_region_idx < self.regions_count {
            let region = &self.regions[self.current_region_idx];

            if region.region_type == 1 {
                let region_end = region.start + region.length;
                if self.current_addr + FRAME_SIZE <= region_end {
                    let frame = self.current_addr;
                    self.current_addr += FRAME_SIZE;
                    self.allocated_count += 1;
                    return Some(frame);
                }
            }

            self.current_region_idx += 1;
            if self.current_region_idx < self.regions_count {
                self.current_addr = self.regions[self.current_region_idx].start;
            }
        }

        None
    }

    /// Deallocate a physical frame (recorded in statistics)
    pub fn deallocate_frame(&mut self, _frame_addr: u64) {
        if self.allocated_count > 0 {
            self.allocated_count -= 1;
        }
    }

    pub fn total_frames(&self) -> u64 {
        self.total_frames_count
    }

    pub fn allocated_frames(&self) -> u64 {
        self.allocated_count
    }

    pub fn free_frames(&self) -> u64 {
        self.total_frames_count.saturating_sub(self.allocated_count)
    }
}
