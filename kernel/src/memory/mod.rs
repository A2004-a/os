//! NovaOS Apex — Kernel Memory Subsystem

pub mod address;
pub mod frame;
pub mod heap;
pub mod paging;
pub mod stack;
pub mod stats;
pub mod tests;

pub use address::{PhysAddr, VirtAddr, PAGE_SIZE, PHYSICAL_MEMORY_OFFSET};
pub use frame::{alloc_contiguous_frames, alloc_frame, free_contiguous_frames, free_frame, FrameError};
pub use heap::{get_heap_stats, HeapStats, KERNEL_HEAP};
pub use paging::{get_mapper, OffsetPageTable, PageTable, PagingError};
pub use stack::KernelStack;
pub use stats::{get_system_stats, SystemMemoryStats};

use crate::boot::BootInfo;

pub fn init(boot_info: &BootInfo) {
    // 1. Initialize physical frame allocator from UEFI memory map
    frame::init(boot_info);

    // 2. Initialize 4-level paging and kernel PML4 table
    paging::init(boot_info);

    // 3. Initialize kernel dynamic heap allocator
    heap::init();
}
