//! NovaOS Zero — Milestone M2 Automated Memory & Object Diagnostic Suite

use crate::boot::info::BootInfo;
use crate::memory::frame::BootInfoFrameAllocator;
use crate::memory::paging::{PageMapper, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::object::{active_object_count, KernelObject, KernelObjectType, HandleTable};
use crate::security::CapabilityFlags;
use crate::serial_println;

pub fn run_memory_diagnostics(boot_info: &BootInfo) {
    serial_println!("----------------------------------------");
    serial_println!("[TEST-M2] Commencing Memory & Object Suite");
    serial_println!("----------------------------------------");

    // 1. Physical Frame Allocator Test
    let mut frame_allocator = BootInfoFrameAllocator::new(
        boot_info.memory_map.regions,
        boot_info.memory_map.regions_count,
    );

    let total = frame_allocator.total_frames();
    serial_println!("[TEST-M2] 1/4: Frame Allocator: Total {} frames ({} MiB)", total, (total * 4) / 1024);

    let frame1 = frame_allocator.allocate_frame().expect("Frame 1 alloc failed");
    let frame2 = frame_allocator.allocate_frame().expect("Frame 2 alloc failed");
    assert!(frame1 % 4096 == 0, "Frame 1 must be 4 KiB aligned");
    assert!(frame2 % 4096 == 0, "Frame 2 must be 4 KiB aligned");
    assert!(frame1 != frame2, "Frames must not collide");
    frame_allocator.deallocate_frame(frame1);
    frame_allocator.deallocate_frame(frame2);
    serial_println!("[TEST-M2] -> Frame Allocator verified. Alignment & non-collision OK.");

    // 2. 4-Level Virtual Memory Paging & Translation Test
    let mut mapper = PageMapper::current();
    let test_virt = 0x0000_1000_0000u64;
    let test_phys = frame_allocator.allocate_frame().expect("Phys frame allocation failed");

    unsafe {
        let map_res = mapper.map_page(
            test_virt,
            test_phys,
            PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER,
            &mut frame_allocator,
        );
        assert!(map_res.is_ok(), "Page mapping must succeed");
    }

    let translated = mapper.translate(test_virt);
    assert_eq!(translated, Some(test_phys), "Virtual translation must match physical address");
    serial_println!("[TEST-M2] 2/4: Paging: Virtual 0x{:016X} -> Phys 0x{:016X} OK.", test_virt, test_phys);

    unsafe {
        let unmap_res = mapper.unmap_page(test_virt);
        assert!(unmap_res.is_ok(), "Page unmap must succeed");
    }
    assert_eq!(mapper.translate(test_virt), None, "Unmapped page must not translate");
    serial_println!("[TEST-M2] -> Unmapping & TLB invalidation verified.");

    // 3. Kernel Object Model & Reference Counting Test
    let initial_objects = active_object_count();
    let obj = KernelObject::new(KernelObjectType::SharedMemory, test_phys);
    assert_eq!(obj.ref_count(), 1, "Initial ref count must be 1");
    assert_eq!(active_object_count(), initial_objects + 1);

    obj.retain();
    assert_eq!(obj.ref_count(), 2, "Retained ref count must be 2");

    let released_first = obj.release();
    assert!(!released_first, "Object must not be destroyed while ref count > 0");

    let released_final = obj.release();
    assert!(released_final, "Object must be destroyed when ref count reaches 0");
    assert_eq!(active_object_count(), initial_objects, "Active objects must return to baseline");
    serial_println!("[TEST-M2] 3/4: Kernel Object: Retain/Release lifecycle verified. Zero leaks.");

    // 4. Per-Process Handle Table & Capability Check Test
    let mut handle_table = HandleTable::new();
    let h1 = handle_table.allocate(101, CapabilityFlags::READ).expect("Handle alloc failed");
    let h2 = handle_table.allocate(102, CapabilityFlags::READ | CapabilityFlags::WRITE).expect("Handle alloc failed");

    assert!(handle_table.get(h1, CapabilityFlags::READ).is_ok(), "Read permitted on h1");
    assert!(handle_table.get(h1, CapabilityFlags::WRITE).is_err(), "Write denied on h1");
    assert!(handle_table.get(h2, CapabilityFlags::WRITE).is_ok(), "Write permitted on h2");

    let closed = handle_table.close(h1);
    assert_eq!(closed, Ok(101), "Closed handle must match object ID");
    assert!(handle_table.get(h1, CapabilityFlags::READ).is_err(), "Closed handle must fail");

    let _closed_all = handle_table.close_all();
    assert_eq!(handle_table.active_handles(), 0, "Handle table must be completely empty on exit");
    serial_println!("[TEST-M2] 4/4: Handle Table: Permissions & process teardown verified.");

    serial_println!("[TEST-M2] ALL MILESTONE M2 TESTS PASSED SUCCESSFULLY.\n");
}
