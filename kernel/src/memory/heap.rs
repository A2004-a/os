//! NovaOS Zero — Freestanding Dynamic Kernel Heap Allocator

use core::alloc::{GlobalAlloc, Layout};
use crate::sync::Spinlock;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 256 * 1024; // 256 KiB

pub struct SimpleHeap {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl SimpleHeap {
    pub const fn empty() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.next = start;
        self.allocations = 0;
    }
}

pub struct LockedHeap(Spinlock<SimpleHeap>);

impl LockedHeap {
    pub const fn empty() -> Self {
        Self(Spinlock::new(SimpleHeap::empty()))
    }

    pub fn lock(&self) -> crate::sync::SpinlockGuard<'_, SimpleHeap> {
        self.0.lock()
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.0.lock();
        if heap.heap_start == 0 {
            return core::ptr::null_mut();
        }
        let align = layout.align();
        let size = layout.size();
        let alloc_start = (heap.next + align - 1) & !(align - 1);
        let alloc_end = alloc_start + size;
        if alloc_end > heap.heap_end {
            core::ptr::null_mut()
        } else {
            heap.next = alloc_end;
            heap.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut heap = self.0.lock();
        if heap.allocations > 0 {
            heap.allocations -= 1;
            if heap.allocations == 0 {
                heap.next = heap.heap_start;
            }
        }
    }
}

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    unsafe {
        // In M1 bootstrap, configure static heap buffer
        static mut STATIC_HEAP_STORAGE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
        let start = core::ptr::addr_of_mut!(STATIC_HEAP_STORAGE) as usize;
        ALLOCATOR.lock().init(start, HEAP_SIZE);
    }
}
