//! Physical Frame Allocator (4 KiB page frames)

use crate::boot::info::{MemoryRegion, MAX_MEMORY_REGIONS};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

pub struct BootInfoFrameAllocator {
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    regions_count: usize,
    current_region_idx: usize,
    current_addr: u64,
}

impl BootInfoFrameAllocator {
    pub fn new(regions: [MemoryRegion; MAX_MEMORY_REGIONS], count: usize) -> Self {
        let first_addr = if count > 0 && regions[0].region_type == 1 {
            regions[0].start
        } else {
            0
        };

        BootInfoFrameAllocator {
            regions,
            regions_count: count,
            current_region_idx: 0,
            current_addr: first_addr,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        while self.current_region_idx < self.regions_count {
            let region = &self.regions[self.current_region_idx];

            // 1 = Usable RAM
            if region.region_type == 1 {
                let region_end = region.start + region.length;
                if self.current_addr + 4096 <= region_end {
                    let frame = PhysFrame::containing_address(PhysAddr::new(self.current_addr));
                    self.current_addr += 4096;
                    return Some(frame);
                }
            }

            // Advance to next region
            self.current_region_idx += 1;
            if self.current_region_idx < self.regions_count {
                self.current_addr = self.regions[self.current_region_idx].start;
            }
        }

        None
    }
}
