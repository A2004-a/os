//! NovaOS Zero — Freestanding 4-Level x86_64 Virtual Memory Paging & Mapper
//! PML4 (Level 4) -> PDPT (Level 3) -> PD (Level 2) -> PT (Level 1)

use crate::arch::x86_64::registers::ControlRegisters;
use super::frame::BootInfoFrameAllocator;

pub const PAGE_SIZE: u64 = 4096;

pub const PAGE_PRESENT: u64  = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64     = 1 << 2;
pub const PAGE_WRITETHROUGH: u64 = 1 << 3;
pub const PAGE_NO_CACHE: u64 = 1 << 4;
pub const PAGE_ACCESSED: u64 = 1 << 5;
pub const PAGE_DIRTY: u64    = 1 << 6;
pub const PAGE_HUGE_2MB: u64 = 1 << 7;
pub const PAGE_GLOBAL: u64   = 1 << 8;
pub const PAGE_NO_EXEC: u64  = 1 << 63;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    pub const fn empty() -> Self {
        Self { entries: [0; 512] }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = 0;
        }
    }
}

pub struct PageMapper {
    pub pml4_phys: u64,
}

impl PageMapper {
    pub fn current() -> Self {
        Self {
            pml4_phys: ControlRegisters::read_cr3() & !0xFFF,
        }
    }

    /// Decompose a 48-bit canonical virtual address into page table indices
    pub fn split_address(virt: u64) -> (usize, usize, usize, usize, usize) {
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx   = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx   = ((virt >> 12) & 0x1FF) as usize;
        let offset   = (virt & 0xFFF) as usize;
        (pml4_idx, pdpt_idx, pd_idx, pt_idx, offset)
    }

    /// Translate a virtual address to physical address by walking page tables
    pub fn translate(&self, virt: u64) -> Option<u64> {
        let (pml4_i, pdpt_i, pd_i, pt_i, offset) = Self::split_address(virt);

        unsafe {
            let pml4 = &*(self.pml4_phys as *const PageTable);
            let pml4_entry = pml4.entries[pml4_i];
            if (pml4_entry & PAGE_PRESENT) == 0 {
                return None;
            }

            let pdpt_phys = pml4_entry & 0x000F_FFFF_FFFF_F000;
            let pdpt = &*(pdpt_phys as *const PageTable);
            let pdpt_entry = pdpt.entries[pdpt_i];
            if (pdpt_entry & PAGE_PRESENT) == 0 {
                return None;
            }

            // Check for 1 GiB huge page
            if (pdpt_entry & PAGE_HUGE_2MB) != 0 {
                let phys_base = pdpt_entry & 0x000F_FFFF_C000_0000;
                return Some(phys_base + (virt & 0x3FFF_FFFF));
            }

            let pd_phys = pdpt_entry & 0x000F_FFFF_FFFF_F000;
            let pd = &*(pd_phys as *const PageTable);
            let pd_entry = pd.entries[pd_i];
            if (pd_entry & PAGE_PRESENT) == 0 {
                return None;
            }

            // Check for 2 MiB huge page
            if (pd_entry & PAGE_HUGE_2MB) != 0 {
                let phys_base = pd_entry & 0x000F_FFFF_FFE0_0000;
                return Some(phys_base + (virt & 0x1F_FFFF));
            }

            let pt_phys = pd_entry & 0x000F_FFFF_FFFF_F000;
            let pt = &*(pt_phys as *const PageTable);
            let pt_entry = pt.entries[pt_i];
            if (pt_entry & PAGE_PRESENT) == 0 {
                return None;
            }

            let frame_phys = pt_entry & 0x000F_FFFF_FFFF_F000;
            Some(frame_phys + offset as u64)
        }
    }

    /// Map a 4 KiB virtual page to a physical frame
    pub unsafe fn map_page(
        &mut self,
        virt: u64,
        phys: u64,
        flags: u64,
        allocator: &mut BootInfoFrameAllocator,
    ) -> Result<(), &'static str> {
        let (pml4_i, pdpt_i, pd_i, pt_i, _) = Self::split_address(virt);

        let pml4 = &mut *(self.pml4_phys as *mut PageTable);

        // Level 4 -> Level 3
        if (pml4.entries[pml4_i] & PAGE_PRESENT) == 0 {
            let frame = allocator.allocate_frame().ok_or("Out of memory for PDPT")?;
            let pt_ptr = frame as *mut PageTable;
            (*pt_ptr).zero();
            pml4.entries[pml4_i] = frame | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
        }

        let pdpt_phys = pml4.entries[pml4_i] & 0x000F_FFFF_FFFF_F000;
        let pdpt = &mut *(pdpt_phys as *mut PageTable);

        // Level 3 -> Level 2
        if (pdpt.entries[pdpt_i] & PAGE_PRESENT) == 0 {
            let frame = allocator.allocate_frame().ok_or("Out of memory for PD")?;
            let pt_ptr = frame as *mut PageTable;
            (*pt_ptr).zero();
            pdpt.entries[pdpt_i] = frame | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
        }

        let pd_phys = pdpt.entries[pdpt_i] & 0x000F_FFFF_FFFF_F000;
        let pd = &mut *(pd_phys as *mut PageTable);

        // Level 2 -> Level 1
        if (pd.entries[pd_i] & PAGE_PRESENT) == 0 {
            let frame = allocator.allocate_frame().ok_or("Out of memory for PT")?;
            let pt_ptr = frame as *mut PageTable;
            (*pt_ptr).zero();
            pd.entries[pd_i] = frame | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
        }

        let pt_phys = pd.entries[pd_i] & 0x000F_FFFF_FFFF_F000;
        let pt = &mut *(pt_phys as *mut PageTable);

        // Level 1: Page Table Entry
        pt.entries[pt_i] = (phys & !0xFFF) | flags | PAGE_PRESENT;

        // Invalidate TLB for this virtual address
        ControlRegisters::invlpg(virt);

        Ok(())
    }

    /// Unmap a virtual page and flush TLB
    pub unsafe fn unmap_page(&mut self, virt: u64) -> Result<(), &'static str> {
        let (pml4_i, pdpt_i, pd_i, pt_i, _) = Self::split_address(virt);

        let pml4 = &*(self.pml4_phys as *const PageTable);
        if (pml4.entries[pml4_i] & PAGE_PRESENT) == 0 {
            return Err("Page not mapped (PML4 missing)");
        }

        let pdpt_phys = pml4.entries[pml4_i] & 0x000F_FFFF_FFFF_F000;
        let pdpt = &*(pdpt_phys as *const PageTable);
        if (pdpt.entries[pdpt_i] & PAGE_PRESENT) == 0 {
            return Err("Page not mapped (PDPT missing)");
        }

        let pd_phys = pdpt.entries[pdpt_i] & 0x000F_FFFF_FFFF_F000;
        let pd = &*(pd_phys as *const PageTable);
        if (pd.entries[pd_i] & PAGE_PRESENT) == 0 {
            return Err("Page not mapped (PD missing)");
        }

        let pt_phys = pd.entries[pd_i] & 0x000F_FFFF_FFFF_F000;
        let pt = &mut *(pt_phys as *mut PageTable);
        if (pt.entries[pt_i] & PAGE_PRESENT) == 0 {
            return Err("Page not mapped (PT entry empty)");
        }

        pt.entries[pt_i] = 0;
        ControlRegisters::invlpg(virt);

        Ok(())
    }

    /// Create a guard page by clearing the PRESENT bit
    pub unsafe fn create_guard_page(&mut self, virt: u64) -> Result<(), &'static str> {
        let _ = self.unmap_page(virt);
        ControlRegisters::invlpg(virt);
        Ok(())
    }
}

pub fn init_paging() -> PageMapper {
    PageMapper::current()
}
