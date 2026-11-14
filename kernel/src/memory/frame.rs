//! NovaOS Apex — UEFI Memory Map Parser & Physical Frame Allocator
//! Bitmap-backed 4 KiB physical page frame allocator with double-free detection.

use super::address::{PhysAddr, PAGE_SIZE};
use crate::boot::BootInfo;
use crate::serial;
use core::ptr::addr_of_mut;

pub const MAX_PHYSICAL_MEMORY: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB addressable space
pub const TOTAL_FRAMES: usize = (MAX_PHYSICAL_MEMORY / PAGE_SIZE as u64) as usize; // 1,048,576 frames
pub const BITMAP_WORDS: usize = TOTAL_FRAMES / 64; // 16,384 u64 words = 128 KiB

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UefiMemoryDescriptor {
    pub mem_type: u32,
    pub pad: u32,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_count: u64,
    pub attribute: u64,
}

pub const EFI_CONVENTIONAL_MEMORY: u32 = 7;
pub const EFI_LOADER_CODE: u32 = 1;
pub const EFI_LOADER_DATA: u32 = 2;
pub const EFI_BOOT_SERVICES_CODE: u32 = 3;
pub const EFI_BOOT_SERVICES_DATA: u32 = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
    OutOfMemory,
    NotAligned,
    OutOfBounds,
    DoubleFree,
    InvalidFrame,
}

#[allow(dead_code)]
pub struct PhysicalFrameAllocator {
    bitmap: [u64; BITMAP_WORDS],
    total_frames: usize,
    free_frames: usize,
    allocated_frames: usize,
    reserved_frames: usize,
}

static mut FRAME_ALLOCATOR: PhysicalFrameAllocator = PhysicalFrameAllocator {
    bitmap: [!0u64; BITMAP_WORDS], // Initially mark all frames as allocated/unavailable (1 = allocated)
    total_frames: TOTAL_FRAMES,
    free_frames: 0,
    allocated_frames: TOTAL_FRAMES,
    reserved_frames: TOTAL_FRAMES,
};

impl PhysicalFrameAllocator {
    #[inline(always)]
    fn set_bit(&mut self, frame_idx: usize) {
        let word = frame_idx / 64;
        let bit = frame_idx % 64;
        self.bitmap[word] |= 1u64 << bit;
    }

    #[inline(always)]
    fn clear_bit(&mut self, frame_idx: usize) {
        let word = frame_idx / 64;
        let bit = frame_idx % 64;
        self.bitmap[word] &= !(1u64 << bit);
    }

    #[inline(always)]
    fn test_bit(&self, frame_idx: usize) -> bool {
        let word = frame_idx / 64;
        let bit = frame_idx % 64;
        (self.bitmap[word] & (1u64 << bit)) != 0
    }
}

pub fn init(boot_info: &BootInfo) {
    let alloc = unsafe { &mut *addr_of_mut!(FRAME_ALLOCATOR) };

    // Initially all frames are allocated (!0u64)
    for word in alloc.bitmap.iter_mut() {
        *word = !0u64;
    }

    let mmap_addr = boot_info.memory_map_addr;
    let mmap_size = boot_info.memory_map_size as usize;
    let desc_size = boot_info.descriptor_size as usize;
    let desc_count = boot_info.memory_map_entries as usize;

    if mmap_addr == 0 || desc_size == 0 || desc_count == 0 {
        serial::write_str("[FRAME WARN] UEFI memory map invalid or unavailable.\n");
        return;
    }

    // Protected memory bounds
    let kernel_start = boot_info.kernel_phys_base;
    let kernel_end = kernel_start + boot_info.kernel_size.max(4 * 1024 * 1024); // Protect at least 4MB kernel space
    let fb_start = boot_info.framebuffer_base;
    let fb_end = fb_start + boot_info.framebuffer_size;
    let mmap_end = mmap_addr + mmap_size as u64;

    let mut usable_frames_found = 0;

    for i in 0..desc_count {
        let desc_ptr = (mmap_addr + (i * desc_size) as u64) as *const UefiMemoryDescriptor;
        let desc = unsafe { &*desc_ptr };

        // We accept conventional memory and reclaimable boot services memory
        if desc.mem_type == EFI_CONVENTIONAL_MEMORY
            || desc.mem_type == EFI_BOOT_SERVICES_CODE
            || desc.mem_type == EFI_BOOT_SERVICES_DATA
        {
            let start_frame = (desc.phys_start / PAGE_SIZE as u64) as usize;
            let count = desc.page_count as usize;

            for f in 0..count {
                let frame_idx = start_frame + f;
                if frame_idx >= TOTAL_FRAMES {
                    break;
                }

                let frame_phys = (frame_idx * PAGE_SIZE) as u64;

                // Protect low 1 MiB (0x0 .. 0x100000)
                if frame_phys < 0x100000 {
                    continue;
                }

                // Protect kernel image range
                if frame_phys >= kernel_start && frame_phys < kernel_end {
                    continue;
                }

                // Protect framebuffer
                if fb_size_gt_zero(boot_info) && frame_phys >= fb_start && frame_phys < fb_end {
                    continue;
                }

                // Protect UEFI memory map buffer
                if frame_phys >= mmap_addr && frame_phys < mmap_end {
                    continue;
                }

                // Protect BootInfo struct itself
                let boot_info_phys = boot_info as *const BootInfo as u64;
                if frame_phys == (boot_info_phys & !0xFFF) {
                    continue;
                }

                // Safe usable physical frame!
                alloc.clear_bit(frame_idx);
                usable_frames_found += 1;
            }
        }
    }

    alloc.free_frames = usable_frames_found;
    alloc.allocated_frames = 0;
    alloc.reserved_frames = TOTAL_FRAMES.saturating_sub(usable_frames_found);

    serial::write_str("[FRAME] Initialized physical frame allocator:\n");
    serial::write_str("  Total Tracked Space: ");
    print_u64((TOTAL_FRAMES * PAGE_SIZE / (1024 * 1024)) as u64);
    serial::write_str(" MiB (");
    print_u64(TOTAL_FRAMES as u64);
    serial::write_str(" frames)\n  Usable Physical RAM: ");
    print_u64((usable_frames_found * PAGE_SIZE / (1024 * 1024)) as u64);
    serial::write_str(" MiB (");
    print_u64(usable_frames_found as u64);
    serial::write_str(" free frames)\n");
}

fn fb_size_gt_zero(bi: &BootInfo) -> bool {
    bi.framebuffer_base != 0 && bi.framebuffer_size > 0
}

pub fn alloc_frame() -> Option<PhysAddr> {
    let alloc = unsafe { &mut *addr_of_mut!(FRAME_ALLOCATOR) };

    for word_idx in 0..BITMAP_WORDS {
        if alloc.bitmap[word_idx] != !0u64 {
            // There is at least one free bit (0)
            let word = alloc.bitmap[word_idx];
            let bit_idx = (!word).trailing_zeros() as usize;
            let frame_idx = word_idx * 64 + bit_idx;

            if frame_idx < TOTAL_FRAMES {
                alloc.set_bit(frame_idx);
                alloc.free_frames = alloc.free_frames.saturating_sub(1);
                alloc.allocated_frames += 1;
                return Some(PhysAddr::new((frame_idx * PAGE_SIZE) as u64));
            }
        }
    }
    None
}

pub fn alloc_contiguous_frames(count: usize) -> Option<PhysAddr> {
    if count == 0 {
        return None;
    }
    let alloc = unsafe { &mut *addr_of_mut!(FRAME_ALLOCATOR) };

    let mut consecutive = 0;
    let mut start_idx = 0;

    for frame_idx in 0..TOTAL_FRAMES {
        if !alloc.test_bit(frame_idx) {
            if consecutive == 0 {
                start_idx = frame_idx;
            }
            consecutive += 1;
            if consecutive == count {
                for i in 0..count {
                    alloc.set_bit(start_idx + i);
                }
                alloc.free_frames = alloc.free_frames.saturating_sub(count);
                alloc.allocated_frames += count;
                return Some(PhysAddr::new((start_idx * PAGE_SIZE) as u64));
            }
        } else {
            consecutive = 0;
        }
    }
    None
}

pub fn free_frame(addr: PhysAddr) -> Result<(), FrameError> {
    if !addr.is_page_aligned() {
        return Err(FrameError::NotAligned);
    }

    let frame_idx = (addr.as_u64() / PAGE_SIZE as u64) as usize;
    if frame_idx >= TOTAL_FRAMES {
        return Err(FrameError::OutOfBounds);
    }

    let alloc = unsafe { &mut *addr_of_mut!(FRAME_ALLOCATOR) };

    // Double-free check: if bit is already 0, it was not allocated!
    if !alloc.test_bit(frame_idx) {
        return Err(FrameError::DoubleFree);
    }

    alloc.clear_bit(frame_idx);
    alloc.allocated_frames = alloc.allocated_frames.saturating_sub(1);
    alloc.free_frames += 1;

    Ok(())
}

pub fn free_contiguous_frames(addr: PhysAddr, count: usize) -> Result<(), FrameError> {
    for i in 0..count {
        let frame_addr = PhysAddr::new(addr.as_u64() + (i * PAGE_SIZE) as u64);
        free_frame(frame_addr)?;
    }
    Ok(())
}

pub fn is_frame_allocated(addr: PhysAddr) -> bool {
    let frame_idx = (addr.as_u64() / PAGE_SIZE as u64) as usize;
    if frame_idx >= TOTAL_FRAMES {
        return true;
    }
    let alloc = unsafe { &*addr_of_mut!(FRAME_ALLOCATOR) };
    alloc.test_bit(frame_idx)
}

#[derive(Clone, Copy, Debug)]
pub struct FrameStats {
    pub total_bytes: u64,
    pub usable_bytes: u64,
    pub allocated_bytes: u64,
    pub free_bytes: u64,
}

pub fn get_frame_stats() -> FrameStats {
    let alloc = unsafe { &*addr_of_mut!(FRAME_ALLOCATOR) };
    FrameStats {
        total_bytes: (TOTAL_FRAMES * PAGE_SIZE) as u64,
        usable_bytes: ((alloc.free_frames + alloc.allocated_frames) * PAGE_SIZE) as u64,
        allocated_bytes: (alloc.allocated_frames * PAGE_SIZE) as u64,
        free_bytes: (alloc.free_frames * PAGE_SIZE) as u64,
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
