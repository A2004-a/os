//! NovaOS Apex — Milestone 2 Memory Subsystem Verification Suite
//! Tests physical frame allocator, 4-level paging mapper, kernel heap, and stack guard pages.

use super::address::{VirtAddr, PAGE_SIZE};
use super::frame::{alloc_contiguous_frames, alloc_frame, free_contiguous_frames, free_frame, FrameError};
use super::heap::{CANARY_PREFIX, CANARY_SUFFIX, KERNEL_HEAP};
use super::paging::{get_mapper, PagingError, PAGE_GLOBAL, PAGE_NO_EXECUTE, PAGE_PRESENT, PAGE_WRITABLE};
use super::stack::KernelStack;
use crate::serial;
use core::alloc::{GlobalAlloc, Layout};

pub fn run_all_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex (M2) — Memory Subsystem Test Suite      \n");
    serial::write_str("=======================================================\n");

    let t1 = test_physical_frame_allocator();
    let t2 = test_virtual_memory_mapper();
    let t3 = test_kernel_heap();
    let t4 = test_stack_guard_pages();

    serial::write_str("\n=======================================================\n");
    serial::write_str("   Milestone 2 Memory Acceptance Summary               \n");
    serial::write_str("=======================================================\n");
    serial::write_str("  1. Physical Frame Allocator:   ");
    if t1 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    serial::write_str("  2. 4-Level Paging Mapper:      ");
    if t2 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    serial::write_str("  3. Kernel Heap & Canaries:     ");
    if t3 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    serial::write_str("  4. Hardware Stack Guard Pages: ");
    if t4 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    let all_passed = t1 && t2 && t3 && t4;
    if all_passed {
        serial::write_str("\n[M2 SUCCESS] All memory systems, allocators, and mappers verified!\n");
    } else {
        serial::write_str("\n[M2 FAILURE] One or more memory verification tests failed.\n");
    }
    serial::write_str("=======================================================\n");

    all_passed
}

fn test_physical_frame_allocator() -> bool {
    serial::write_str("[MEM TEST 1/4] Testing physical frame allocator...\n");

    // 1. Single frame allocation
    let f1 = match alloc_frame() {
        Some(f) => f,
        None => {
            serial::write_str("  [FAIL] alloc_frame failed\n");
            return false;
        }
    };
    let f2 = match alloc_frame() {
        Some(f) => f,
        None => {
            serial::write_str("  [FAIL] alloc_frame 2 failed\n");
            return false;
        }
    };

    if f1 == f2 || !f1.is_page_aligned() || !f2.is_page_aligned() {
        serial::write_str("  [FAIL] Frame addresses invalid or colliding\n");
        return false;
    }

    // 2. Contiguous allocation
    let contig = match alloc_contiguous_frames(4) {
        Some(f) => f,
        None => {
            serial::write_str("  [FAIL] alloc_contiguous_frames failed\n");
            return false;
        }
    };

    // 3. Free single frame
    if free_frame(f1).is_err() {
        serial::write_str("  [FAIL] free_frame failed\n");
        return false;
    }

    // 4. Double-free rejection test
    if free_frame(f1) != Err(FrameError::DoubleFree) {
        serial::write_str("  [FAIL] Double-free detection failed\n");
        return false;
    }

    // Clean up remaining test frames
    assert!(free_frame(f2).is_ok());
    assert!(free_contiguous_frames(contig, 4).is_ok());

    serial::write_str("  [PASS] Physical frames, contiguous runs, and double-free detection verified\n");
    true
}

fn test_virtual_memory_mapper() -> bool {
    serial::write_str("[MEM TEST 2/4] Testing 4-level paging mapper/unmapper...\n");

    let mapper = get_mapper();
    let test_virt = VirtAddr::new(0xFFFF_8000_3000_0000);
    let test_frame = match alloc_frame() {
        Some(f) => f,
        None => return false,
    };

    // 1. Map page
    let map_res = mapper.map_page(
        test_virt,
        test_frame,
        PAGE_PRESENT | PAGE_WRITABLE | PAGE_GLOBAL | PAGE_NO_EXECUTE,
    );
    if map_res.is_err() {
        serial::write_str("  [FAIL] Map page failed\n");
        return false;
    }

    // 2. Conflict detection: Map already mapped page
    let conflict_res = mapper.map_page(
        test_virt,
        test_frame,
        PAGE_PRESENT | PAGE_WRITABLE,
    );
    if conflict_res != Err(PagingError::AlreadyMapped) {
        serial::write_str("  [FAIL] Paging conflict detection failed (expected AlreadyMapped)\n");
        return false;
    }

    // 3. Translation verification
    let translated = mapper.translate(test_virt);
    if translated != Some(test_frame) {
        serial::write_str("  [FAIL] Virtual-to-physical translation mismatch\n");
        return false;
    }

    // 4. Memory read/write test through mapped virtual address
    unsafe {
        let ptr = test_virt.as_mut_ptr::<u64>();
        *ptr = 0x5A5A_A5A5_1234_5678;
        if *ptr != 0x5A5A_A5A5_1234_5678 {
            serial::write_str("  [FAIL] Virtual memory read/write verification failed\n");
            return false;
        }
    }

    // 5. Unmap page
    let unmapped_phys = match mapper.unmap_page(test_virt) {
        Ok(p) => p,
        Err(_) => {
            serial::write_str("  [FAIL] Unmap page failed\n");
            return false;
        }
    };
    if unmapped_phys != test_frame {
        serial::write_str("  [FAIL] Unmapped frame mismatch\n");
        return false;
    }

    // 6. Verify translation now returns None
    if mapper.translate(test_virt).is_some() {
        serial::write_str("  [FAIL] Page still translated after unmapping\n");
        return false;
    }

    free_frame(test_frame).unwrap();

    serial::write_str("  [PASS] Mapping, translation, conflict detection, and unmapping verified\n");
    true
}

fn test_kernel_heap() -> bool {
    serial::write_str("[MEM TEST 3/4] Testing kernel dynamic heap allocator...\n");

    let layout_small = Layout::from_size_align(64, 8).unwrap();
    let ptr1 = unsafe { KERNEL_HEAP.alloc(layout_small) };
    if ptr1.is_null() {
        serial::write_str("  [FAIL] Heap alloc small failed\n");
        return false;
    }

    // Write test data
    unsafe {
        core::ptr::write_bytes(ptr1, 0x42, 64);
        for i in 0..64 {
            if *ptr1.add(i) != 0x42 {
                serial::write_str("  [FAIL] Heap memory integrity check failed\n");
                return false;
            }
        }
    }

    // Validate canaries
    let prefix = unsafe { *(ptr1.sub(16) as *const u64) };
    let suffix = unsafe { *(ptr1.add(64) as *const u64) };
    if prefix != CANARY_PREFIX || suffix != CANARY_SUFFIX {
        serial::write_str("  [FAIL] Heap canary validation failed\n");
        return false;
    }

    unsafe {
        KERNEL_HEAP.dealloc(ptr1, layout_small);
    }

    serial::write_str("  [PASS] Heap allocation, canaries, and safe deallocation verified\n");
    true
}

fn test_stack_guard_pages() -> bool {
    serial::write_str("[MEM TEST 4/4] Testing kernel stack allocator & hardware guard pages...\n");

    let stack = match KernelStack::allocate(2) {
        Ok(s) => s,
        Err(_) => {
            serial::write_str("  [FAIL] Failed to allocate stack with guard page\n");
            return false;
        }
    };

    let mapper = get_mapper();

    // Guard page must be unmapped
    if mapper.translate(stack.guard_page).is_some() {
        serial::write_str("  [FAIL] Guard page is mapped (expected unmapped)\n");
        return false;
    }

    // Usable stack pages must be mapped
    if mapper.translate(stack.bottom).is_none() {
        serial::write_str("  [FAIL] Stack bottom page is not mapped\n");
        return false;
    }

    let top_page = VirtAddr::new(stack.top.as_u64() - PAGE_SIZE as u64);
    if mapper.translate(top_page).is_none() {
        serial::write_str("  [FAIL] Stack top page is not mapped\n");
        return false;
    }

    serial::write_str("  [PASS] Stack guard page is unmapped and data pages are mapped correctly\n");
    true
}
