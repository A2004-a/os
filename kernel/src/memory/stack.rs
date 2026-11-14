//! NovaOS Apex — Kernel Stack Allocator with Hardware Guard Pages
//! Protects against stack overflow corruptions by placing an unmapped guard page at the stack bottom.

use super::address::{VirtAddr, PAGE_SIZE};
use super::frame::alloc_frame;
use super::paging::{get_mapper, PagingError, PAGE_GLOBAL, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_WRITABLE};
use core::sync::atomic::{AtomicU64, Ordering};

pub const KERNEL_STACK_VIRTUAL_BASE: u64 = 0xFFFF_8001_0000_0000;
static NEXT_STACK_ADDR: AtomicU64 = AtomicU64::new(KERNEL_STACK_VIRTUAL_BASE);

pub struct KernelStack {
    pub guard_page: VirtAddr,
    pub bottom: VirtAddr,
    pub top: VirtAddr,
    pub page_count: usize,
}

impl KernelStack {
    pub fn allocate(page_count: usize) -> Result<Self, PagingError> {
        let total_pages = page_count + 1; // 1 Guard Page + N data pages
        let total_size = (total_pages * PAGE_SIZE) as u64;

        let start_addr = NEXT_STACK_ADDR.fetch_add(total_size, Ordering::SeqCst);
        let guard_page = VirtAddr::new(start_addr);
        let bottom = VirtAddr::new(start_addr + PAGE_SIZE as u64);
        let top = VirtAddr::new(start_addr + total_size);

        let mapper = get_mapper();

        // 1. Guard page is deliberately left unmapped (PAGE_PRESENT clear)
        // If translation is performed, it returns None.

        // 2. Map all usable stack pages with WRITABLE + NO_EXECUTE
        for i in 0..page_count {
            let virt = VirtAddr::new(bottom.as_u64() + (i * PAGE_SIZE) as u64);
            let phys = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            mapper.map_page(
                virt,
                phys,
                PAGE_PRESENT | PAGE_WRITABLE | PAGE_GLOBAL | PAGE_NO_EXECUTE,
            )?;
        }

        Ok(KernelStack {
            guard_page,
            bottom,
            top,
            page_count,
        })
    }

    pub fn is_overflow(&self, rsp: u64) -> bool {
        rsp < self.bottom.as_u64() && rsp >= self.guard_page.as_u64()
    }
}
