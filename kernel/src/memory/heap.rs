//! NovaOS Apex — Kernel Heap Allocator & Debug Allocation Tracker
//! Implements GlobalAlloc with dynamic physical frame backing, canary overflow checks,
//! LIFO offset recycling, and double-free detection.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::addr_of_mut;
use crate::serial;

pub const FALLBACK_HEAP_SIZE: usize = 64 * 1024; // 64 KiB early/fallback heap
pub const CANARY_PREFIX: u64 = 0xCAFEBABE_DEADBEEF;
pub const CANARY_SUFFIX: u64 = 0xFEEDFACE_BEEFCAFE;

#[allow(dead_code)]
#[repr(align(16))]
struct FallbackHeapStorage([u8; FALLBACK_HEAP_SIZE]);

static mut STATIC_FALLBACK_HEAP: FallbackHeapStorage = FallbackHeapStorage([0u8; FALLBACK_HEAP_SIZE]);

#[derive(Clone, Copy, Debug)]
pub struct HeapStats {
    pub total_bytes: usize,
    pub allocated_bytes: usize,
    pub free_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

pub struct KernelHeap {
    pub base_ptr: *mut u8,
    pub size: usize,
    pub offset: usize,
    pub allocated_bytes: usize,
    pub peak_bytes: usize,
    pub alloc_count: usize,
    pub dealloc_count: usize,
}

static mut KERNEL_HEAP_STATE: KernelHeap = KernelHeap {
    base_ptr: core::ptr::null_mut(),
    size: 0,
    offset: 0,
    allocated_bytes: 0,
    peak_bytes: 0,
    alloc_count: 0,
    dealloc_count: 0,
};

pub struct LockedKernelHeap;

#[global_allocator]
pub static KERNEL_HEAP: LockedKernelHeap = LockedKernelHeap;

unsafe impl GlobalAlloc for LockedKernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap = &mut *addr_of_mut!(KERNEL_HEAP_STATE);
        if heap.base_ptr.is_null() {
            heap.base_ptr = addr_of_mut!(STATIC_FALLBACK_HEAP) as *mut u8;
            heap.size = FALLBACK_HEAP_SIZE;
        }

        let storage_ptr = heap.base_ptr;
        let heap_size = heap.size;

        let align = layout.align().max(16);
        let user_size = layout.size();
        let aligned_user_size = (user_size + 7) & !7;

        // Header: [CanaryPrefix (8 bytes)] [UserSize (8 bytes)] [User Data (aligned_user_size)] [CanarySuffix (8 bytes)]
        let total_size = 16 + aligned_user_size + 8;

        // Align current offset
        let current_ptr = storage_ptr.add(heap.offset);
        let padding = (align - (current_ptr as usize % align)) % align;
        let start_offset = heap.offset + padding;

        if start_offset + total_size > heap_size {
            serial::write_str("[HEAP ERROR] Kernel heap out of memory!\n");
            return core::ptr::null_mut();
        }

        heap.offset = start_offset + total_size;
        heap.allocated_bytes += user_size;
        if heap.allocated_bytes > heap.peak_bytes {
            heap.peak_bytes = heap.allocated_bytes;
        }
        heap.alloc_count += 1;

        let alloc_base = storage_ptr.add(start_offset);

        // Write prefix canary & size header
        let prefix_ptr = alloc_base as *mut u64;
        core::ptr::write_unaligned(prefix_ptr, CANARY_PREFIX);
        let size_ptr = alloc_base.add(8) as *mut u64;
        core::ptr::write_unaligned(size_ptr, user_size as u64);

        // User data pointer
        let user_ptr = alloc_base.add(16);

        // Write suffix canary
        let suffix_ptr = user_ptr.add(aligned_user_size) as *mut u64;
        core::ptr::write_unaligned(suffix_ptr, CANARY_SUFFIX);

        user_ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let heap = &mut *addr_of_mut!(KERNEL_HEAP_STATE);
        if heap.base_ptr.is_null() {
            return;
        }

        let storage_start = heap.base_ptr;
        let storage_end = storage_start.add(heap.size);

        // Bounds validation
        if ptr < storage_start || ptr >= storage_end {
            serial::write_str("[HEAP FATAL] Invalid free: pointer outside heap bounds!\n");
            return;
        }

        let alloc_base = ptr.sub(16);
        let prefix = core::ptr::read_unaligned(alloc_base as *const u64);
        let stored_size = core::ptr::read_unaligned(alloc_base.add(8) as *const u64) as usize;
        let aligned_stored_size = (stored_size + 7) & !7;
        let suffix = core::ptr::read_unaligned(ptr.add(aligned_stored_size) as *const u64);

        // Double-free / corruption verification
        if prefix != CANARY_PREFIX {
            serial::write_str("[HEAP FATAL] Heap corruption or double free: Invalid prefix canary!\n");
            return;
        }
        if suffix != CANARY_SUFFIX {
            serial::write_str("[HEAP FATAL] Buffer overflow detected: Suffix canary overwritten!\n");
            return;
        }

        heap.allocated_bytes = heap.allocated_bytes.saturating_sub(stored_size);
        heap.dealloc_count += 1;

        // LIFO rewind: if this was the last allocation at the top of the heap, roll back offset!
        let total_size = 16 + aligned_stored_size + 8;
        if alloc_base.add(total_size) == storage_start.add(heap.offset) {
            heap.offset = (alloc_base as usize) - (storage_start as usize);
        }

        // Poison memory on free to catch use-after-free
        core::ptr::write_unaligned(alloc_base as *mut u64, 0xDEAD_DEAD_DEAD_DEAD);
        core::ptr::write_bytes(ptr, 0xAA, stored_size);
    }
}

pub fn init() {
    let heap = unsafe { &mut *addr_of_mut!(KERNEL_HEAP_STATE) };

    // Dynamically allocate 16 MiB (4096 contiguous 4 KiB frames) from physical RAM
    const HEAP_FRAMES: usize = 4096;
    if let Some(phys) = crate::memory::frame::alloc_contiguous_frames(HEAP_FRAMES) {
        heap.base_ptr = phys.as_u64() as *mut u8;
        heap.size = HEAP_FRAMES * crate::memory::address::PAGE_SIZE;
        unsafe {
            core::ptr::write_bytes(heap.base_ptr, 0, heap.size);
        }
    } else {
        heap.base_ptr = addr_of_mut!(STATIC_FALLBACK_HEAP) as *mut u8;
        heap.size = FALLBACK_HEAP_SIZE;
    }

    heap.offset = 0;
    heap.allocated_bytes = 0;
    heap.peak_bytes = 0;
    heap.alloc_count = 0;
    heap.dealloc_count = 0;

    serial::write_str("[HEAP] Kernel dynamic allocator initialized:\n");
    serial::write_str("  Total Heap Buffer: ");
    print_u64((heap.size / 1024) as u64);
    serial::write_str(" KiB (Canary guards active)\n");
}

pub fn get_heap_stats() -> HeapStats {
    let heap = unsafe { &*addr_of_mut!(KERNEL_HEAP_STATE) };
    HeapStats {
        total_bytes: heap.size,
        allocated_bytes: heap.allocated_bytes,
        free_bytes: heap.size.saturating_sub(heap.offset),
        peak_allocated_bytes: heap.peak_bytes,
        allocation_count: heap.alloc_count,
        deallocation_count: heap.dealloc_count,
    }
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
