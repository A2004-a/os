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
