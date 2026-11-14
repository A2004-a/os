//! NovaOS CPU Feature Discovery & Model Detection

pub mod conformance;

use core::arch::x86_64::__cpuid;

#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    pub has_fpu: bool,
    pub has_apic: bool,
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_nx: bool,
}

pub fn detect_features() -> CpuFeatures {
    let res = __cpuid(1);
    let has_fpu = (res.edx & (1 << 0)) != 0;
    let has_apic = (res.edx & (1 << 9)) != 0;
    let has_sse = (res.edx & (1 << 25)) != 0;
    let has_sse2 = (res.edx & (1 << 26)) != 0;

    let ext = __cpuid(0x80000001);
    let has_nx = (ext.edx & (1 << 20)) != 0;

    CpuFeatures {
        has_fpu,
        has_apic,
        has_sse,
        has_sse2,
        has_nx,
    }
}
