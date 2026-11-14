//! NovaOS Apex — Four-Level Paging (x86_64 PML4, PDPT, PD, PT) & Virtual Memory Mapper
//! Provides virtual-to-physical translation, fine-grained access flags, and conflict detection.

use super::address::{PhysAddr, VirtAddr, PHYSICAL_MEMORY_OFFSET};
use super::frame::alloc_frame;
use crate::boot::BootInfo;
use crate::serial;
use core::arch::asm;
use core::ptr::addr_of_mut;

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_WRITE_THROUGH: u64 = 1 << 3;
pub const PAGE_NO_CACHE: u64 = 1 << 4;
pub const PAGE_ACCESSED: u64 = 1 << 5;
pub const PAGE_DIRTY: u64 = 1 << 6;
pub const PAGE_HUGE: u64 = 1 << 7;
pub const PAGE_GLOBAL: u64 = 1 << 8;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;

pub const PHYSICAL_ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

#[derive(Debug, PartialEq, Eq)]
pub enum PagingError {
    AlreadyMapped,
    NotMapped,
    AllocationFailed,
    InvalidFlags,
    HugePageNotSupported,
    NotAligned,
}

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    pub const fn empty() -> Self {
        Self { entries: [0u64; 512] }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = 0;
        }
    }
}

// Statically allocated root kernel PML4 and early page directory tables
static mut KERNEL_PML4: PageTable = PageTable::empty();
static mut KERNEL_PDPT: PageTable = PageTable::empty();
static mut KERNEL_PD_LOW: [PageTable; 4] = [
    PageTable::empty(),
    PageTable::empty(),
    PageTable::empty(),
    PageTable::empty(),
];
static mut KERNEL_PDPT_HIGH: PageTable = PageTable::empty();
static mut KERNEL_PD_HIGH: PageTable = PageTable::empty();

pub struct OffsetPageTable {
    pml4_phys: PhysAddr,
}

static mut ACTIVE_MAPPER: OffsetPageTable = OffsetPageTable {
    pml4_phys: PhysAddr(0),
};

impl OffsetPageTable {
    pub fn new(pml4_phys: PhysAddr) -> Self {
        Self { pml4_phys }
    }

    #[inline(always)]
    fn get_table_virt(&self, phys: PhysAddr) -> *mut PageTable {
        // Physical memory is identity mapped or direct mapped at PHYSICAL_MEMORY_OFFSET
        let virt_addr = phys.as_u64();
        virt_addr as *mut PageTable
    }

    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: u64,
    ) -> Result<(), PagingError> {
        if !virt.is_page_aligned() || !phys.is_page_aligned() {
            return Err(PagingError::NotAligned);
        }

        let p4_idx = virt.p4_index();
        let p3_idx = virt.p3_index();
        let p2_idx = virt.p2_index();
        let p1_idx = virt.p1_index();

        let pml4 = unsafe { &mut *self.get_table_virt(self.pml4_phys) };

        // Level 4 -> Level 3 (PDPT)
        let pdpt_phys = if (pml4.entries[p4_idx] & PAGE_PRESENT) == 0 {
            let new_frame = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            let pdpt = unsafe { &mut *self.get_table_virt(new_frame) };
            pdpt.zero();
            pml4.entries[p4_idx] = new_frame.as_u64() | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
            new_frame
        } else {
            PhysAddr::new(pml4.entries[p4_idx] & PHYSICAL_ADDRESS_MASK)
        };

        // Level 3 -> Level 2 (PD)
        let pdpt = unsafe { &mut *self.get_table_virt(pdpt_phys) };
        let pd_phys = if (pdpt.entries[p3_idx] & PAGE_PRESENT) == 0 {
            let new_frame = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            let pd = unsafe { &mut *self.get_table_virt(new_frame) };
            pd.zero();
            pdpt.entries[p3_idx] = new_frame.as_u64() | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
            new_frame
        } else {
            PhysAddr::new(pdpt.entries[p3_idx] & PHYSICAL_ADDRESS_MASK)
        };

        // Level 2 -> Level 1 (PT)
        let pd = unsafe { &mut *self.get_table_virt(pd_phys) };
        let pt_phys = if (pd.entries[p2_idx] & PAGE_PRESENT) == 0 {
            let new_frame = alloc_frame().ok_or(PagingError::AllocationFailed)?;
            let pt = unsafe { &mut *self.get_table_virt(new_frame) };
            pt.zero();
            pd.entries[p2_idx] = new_frame.as_u64() | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
            new_frame
        } else {
            PhysAddr::new(pd.entries[p2_idx] & PHYSICAL_ADDRESS_MASK)
        };

        // Level 1: Page Table Entry
        let pt = unsafe { &mut *self.get_table_virt(pt_phys) };
        if (pt.entries[p1_idx] & PAGE_PRESENT) != 0 {
            return Err(PagingError::AlreadyMapped);
        }

        pt.entries[p1_idx] = phys.as_u64() | flags | PAGE_PRESENT;

        // Invalidate TLB for this virtual address
        invlpg(virt);

        Ok(())
    }

    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<PhysAddr, PagingError> {
        if !virt.is_page_aligned() {
            return Err(PagingError::NotAligned);
        }

        let p4_idx = virt.p4_index();
        let p3_idx = virt.p3_index();
        let p2_idx = virt.p2_index();
        let p1_idx = virt.p1_index();

        let pml4 = unsafe { &mut *self.get_table_virt(self.pml4_phys) };
        if (pml4.entries[p4_idx] & PAGE_PRESENT) == 0 {
            return Err(PagingError::NotMapped);
        }

        let pdpt_phys = PhysAddr::new(pml4.entries[p4_idx] & PHYSICAL_ADDRESS_MASK);
        let pdpt = unsafe { &mut *self.get_table_virt(pdpt_phys) };
        if (pdpt.entries[p3_idx] & PAGE_PRESENT) == 0 {
            return Err(PagingError::NotMapped);
        }

        let pd_phys = PhysAddr::new(pdpt.entries[p3_idx] & PHYSICAL_ADDRESS_MASK);
        let pd = unsafe { &mut *self.get_table_virt(pd_phys) };
        if (pd.entries[p2_idx] & PAGE_PRESENT) == 0 {
            return Err(PagingError::NotMapped);
        }

        let pt_phys = PhysAddr::new(pd.entries[p2_idx] & PHYSICAL_ADDRESS_MASK);
        let pt = unsafe { &mut *self.get_table_virt(pt_phys) };
        if (pt.entries[p1_idx] & PAGE_PRESENT) == 0 {
            return Err(PagingError::NotMapped);
        }

        let phys = PhysAddr::new(pt.entries[p1_idx] & PHYSICAL_ADDRESS_MASK);
        pt.entries[p1_idx] = 0;

        invlpg(virt);

        Ok(phys)
    }

    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        let p4_idx = virt.p4_index();
        let p3_idx = virt.p3_index();
        let p2_idx = virt.p2_index();
        let p1_idx = virt.p1_index();

        let pml4 = unsafe { &*self.get_table_virt(self.pml4_phys) };
        if (pml4.entries[p4_idx] & PAGE_PRESENT) == 0 {
            return None;
        }

        let pdpt_phys = PhysAddr::new(pml4.entries[p4_idx] & PHYSICAL_ADDRESS_MASK);
        let pdpt = unsafe { &*self.get_table_virt(pdpt_phys) };
        if (pdpt.entries[p3_idx] & PAGE_PRESENT) == 0 {
            return None;
        }

        let pd_phys = PhysAddr::new(pdpt.entries[p3_idx] & PHYSICAL_ADDRESS_MASK);
        let pd = unsafe { &*self.get_table_virt(pd_phys) };
        if (pd.entries[p2_idx] & PAGE_PRESENT) == 0 {
            return None;
        }

        // Check for 2 MiB Huge Page in PD
        if (pd.entries[p2_idx] & PAGE_HUGE) != 0 {
            let huge_phys_base = pd.entries[p2_idx] & 0x000F_FFFF_FFE0_0000;
            let huge_offset = virt.as_u64() & 0x1F_FFFF;
            return Some(PhysAddr::new(huge_phys_base + huge_offset));
        }

        let pt_phys = PhysAddr::new(pd.entries[p2_idx] & PHYSICAL_ADDRESS_MASK);
        let pt = unsafe { &*self.get_table_virt(pt_phys) };
        if (pt.entries[p1_idx] & PAGE_PRESENT) == 0 {
            return None;
        }

        let phys_base = pt.entries[p1_idx] & PHYSICAL_ADDRESS_MASK;
        Some(PhysAddr::new(phys_base + virt.page_offset()))
    }
}

#[inline(always)]
pub fn invlpg(virt: VirtAddr) {
    unsafe {
        asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack, preserves_flags));
    }
}

pub fn init(_boot_info: &BootInfo) {
    unsafe {
        let pml4 = &mut *addr_of_mut!(KERNEL_PML4);
        let pdpt_low = &mut *addr_of_mut!(KERNEL_PDPT);
        let pdpt_high = &mut *addr_of_mut!(KERNEL_PDPT_HIGH);
        let pd_high = &mut *addr_of_mut!(KERNEL_PD_HIGH);

        pml4.zero();
        pdpt_low.zero();
        pdpt_high.zero();
        pd_high.zero();

        let pdpt_low_phys = pdpt_low as *const PageTable as u64;
        let pdpt_high_phys = pdpt_high as *const PageTable as u64;
        let pd_high_phys = pd_high as *const PageTable as u64;

        // 1. Identity map lower 4 GiB using 2 MiB huge pages (0x0 .. 0x1_0000_0000)
        pml4.entries[0] = pdpt_low_phys | PAGE_PRESENT | PAGE_WRITABLE;

        for g in 0..4 {
            let pd = &mut (*addr_of_mut!(KERNEL_PD_LOW))[g];
            pd.zero();
            let pd_phys = pd as *const PageTable as u64;
            pdpt_low.entries[g] = pd_phys | PAGE_PRESENT | PAGE_WRITABLE;

            for i in 0..512 {
                let phys_addr = ((g as u64) * 512 + (i as u64)) * (2 * 1024 * 1024);
                pd.entries[i] = phys_addr | PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE | PAGE_GLOBAL;
            }
        }

        // 2. Higher-half mapping for kernel space (0xFFFF_8000_0000_0000)
        let p4_high_idx = (PHYSICAL_MEMORY_OFFSET >> 39) & 0x1FF;
        pml4.entries[p4_high_idx as usize] = pdpt_high_phys | PAGE_PRESENT | PAGE_WRITABLE;
        pdpt_high.entries[0] = pd_high_phys | PAGE_PRESENT | PAGE_WRITABLE;

        for i in 0..512 {
            let phys_addr = (i as u64) * (2 * 1024 * 1024);
            pd_high.entries[i] = phys_addr | PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE | PAGE_GLOBAL;
        }

        let pml4_phys = PhysAddr::new(pml4 as *const PageTable as u64);
        ACTIVE_MAPPER = OffsetPageTable::new(pml4_phys);

        // 3. Load CR3 with new kernel PML4
        crate::arch::x86_64::instructions::write_cr3(pml4_phys.as_u64());
    }

    serial::write_str("[PAGING] 4-level paging initialized. Kernel CR3 loaded with new PML4.\n");
    serial::write_str("  Lower identity: [0x0000000000000000 - 0x0000000100000000] (4 GiB, 2 MiB pages)\n");
    serial::write_str("  Higher direct:  [0xFFFF800000000000 - 0xFFFF800040000000] (1 GiB, 2 MiB pages)\n");
}

pub fn get_mapper() -> &'static mut OffsetPageTable {
    unsafe { &mut *addr_of_mut!(ACTIVE_MAPPER) }
}
