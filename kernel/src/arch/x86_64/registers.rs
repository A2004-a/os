//! x86_64 Control Register Manipulation

use core::arch::asm;

pub struct ControlRegisters;

impl ControlRegisters {
    /// Read CR0 register (holds control bits PE, MP, EM, TS, ET, NE, WP, AM, NW, CD, PG)
    #[inline]
    pub fn read_cr0() -> u64 {
        let value: u64;
        unsafe {
            asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
        }
        value
    }

    /// Read CR2 register (holds linear faulting address on Page Fault)
    #[inline]
    pub fn read_cr2() -> u64 {
        let value: u64;
        unsafe {
            asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
        }
        value
    }

    /// Read CR3 register (holds physical base address of PML4 page table)
    #[inline]
    pub fn read_cr3() -> u64 {
        let value: u64;
        unsafe {
            asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        }
        value
    }

    /// Write CR3 register (loads new PML4 page table and flushes non-global TLB)
    #[inline]
    pub unsafe fn write_cr3(val: u64) {
        asm!("mov cr3, {}", in(reg) val, options(nostack, preserves_flags));
    }

    /// Invalidate single TLB entry for specified virtual address
    #[inline]
    pub unsafe fn invlpg(addr: u64) {
        asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
    }
}
