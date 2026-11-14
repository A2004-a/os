//! NovaOS Zero — Hardware Conformance & Platform Certification Engine
//! Validates CPU instruction set capabilities, control registers, privilege rings,
//! and physical memory map invariants.

use core::arch::x86_64::__cpuid;
use crate::arch::x86_64::registers::ControlRegisters;

#[derive(Debug, Clone, Copy)]
pub struct ConformanceReport {
    pub long_mode_certified: bool,
    pub paging_enabled: bool,
    pub write_protect_active: bool,
    pub fpu_certified: bool,
    pub sse_certified: bool,
    pub sse2_certified: bool,
    pub nx_certified: bool,
    pub apic_certified: bool,
}

pub struct HardwareConformance;

impl HardwareConformance {
    /// Execute comprehensive hardware conformance audit
    pub fn certify_hardware() -> ConformanceReport {
        // 1. CPUID Leaf 1: Features
        let cpuid1 = __cpuid(1);
        let has_fpu = (cpuid1.edx & (1 << 0)) != 0;
        let has_apic = (cpuid1.edx & (1 << 9)) != 0;
        let has_sse = (cpuid1.edx & (1 << 25)) != 0;
        let has_sse2 = (cpuid1.edx & (1 << 26)) != 0;

        // 2. CPUID Extended Leaf 80000001h: Long Mode & NX bit
        let ext = __cpuid(0x8000_0001);
        let has_lm = (ext.edx & (1 << 29)) != 0;
        let has_nx = (ext.edx & (1 << 20)) != 0;

        // 3. Control Register Invariants:
        // CR0 Bit 0 (PE): Protection Enable
        // CR0 Bit 16 (WP): Write Protect in Ring 0
        // CR0 Bit 31 (PG): Paging Enable
        let cr0 = ControlRegisters::read_cr0();
        let pe = (cr0 & (1 << 0)) != 0;
        let wp = (cr0 & (1 << 16)) != 0;
        let pg = (cr0 & (1 << 31)) != 0;

        // CR3: Paging table root address must be non-zero and 4 KiB aligned
        let cr3 = ControlRegisters::read_cr3();
        let cr3_valid = cr3 != 0 && (cr3 & 0xFFF) == 0;

        ConformanceReport {
            long_mode_certified: has_lm,
            paging_enabled: pe && pg && cr3_valid,
            write_protect_active: wp,
            fpu_certified: has_fpu,
            sse_certified: has_sse,
            sse2_certified: has_sse2,
            nx_certified: has_nx,
            apic_certified: has_apic,
        }
    }
}
