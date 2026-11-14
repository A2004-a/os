//! NovaOS Apex — Architecture-Specific x86_64 Assembly Primitives

use core::arch::asm;

#[repr(C, packed)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}

#[inline(always)]
pub unsafe fn lgdt(ptr: *const DescriptorTablePointer) {
    asm!("lgdt [{}]", in(reg) ptr, options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn lidt(ptr: *const DescriptorTablePointer) {
    asm!("lidt [{}]", in(reg) ptr, options(nostack, preserves_flags));
}

#[inline(always)]
pub unsafe fn ltr(selector: u16) {
    asm!("ltr {0:x}", in(reg) selector, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
pub fn read_cr2() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline(always)]
pub fn read_cr3() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline(always)]
pub unsafe fn write_cr3(val: u64) {
    asm!("mov cr3, {}", in(reg) val, options(nostack, preserves_flags));
}

#[inline(always)]
pub fn read_rsp() -> u64 {
    let val: u64;
    unsafe {
        asm!("mov {}, rsp", out(reg) val, options(nomem, nostack, preserves_flags));
    }
    val
}

#[inline(always)]
pub fn cli() {
    unsafe {
        asm!("cli", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn sti() {
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn hlt() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}

#[inline(always)]
pub fn pause() {
    unsafe {
        asm!("pause", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub unsafe fn reload_segments(cs: u16, ds: u16) {
    asm!(
        "mov ds, {0:x}",
        "mov es, {0:x}",
        "mov ss, {0:x}",
        "mov fs, {0:x}",
        "mov gs, {0:x}",
        "lea {2}, [rip + 2f]",
        "push {1:r}",
        "push {2}",
        "retfq",
        "2:",
        in(reg) ds,
        in(reg) cs as u64,
        out(reg) _,
        options(preserves_flags)
    );
}

pub const IA32_FS_BASE: u32 = 0xC000_0100;
pub const IA32_GS_BASE: u32 = 0xC000_0101;
pub const IA32_KERNEL_GS_BASE: u32 = 0xC000_0102;

#[inline(always)]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
    ((high as u64) << 32) | (low as u64)
}

#[inline(always)]
pub unsafe fn wrmsr(msr: u32, val: u64) {
    let low = val as u32;
    let high = (val >> 32) as u32;
    asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
}
