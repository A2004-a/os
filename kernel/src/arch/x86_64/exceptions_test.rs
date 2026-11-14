//! NovaOS Zero — Automated CPU Architecture Exception Diagnostic Tests

use crate::serial_println;

pub fn run_exception_diagnostics() {
    serial_println!("[TEST] Commencing CPU architecture exception diagnostic suite...");

    // 1. Breakpoint Test (#BP - Trap)
    serial_println!("[TEST] 1/3: Triggering Breakpoint Trap (#BP, int3)...");
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
    serial_println!("[TEST] -> Breakpoint handled successfully. Execution continued.");

    serial_println!("[TEST] Diagnostic suite ready for hardware fault injection.");
}

/// Deliberate division by zero test (invoked on demand)
pub fn trigger_divide_by_zero() -> ! {
    serial_println!("[TEST] Triggering Divide-by-Zero (#DE)...");
    unsafe {
        core::arch::asm!(
            "mov edx, 0",
            "mov eax, 42",
            "mov ecx, 0",
            "idiv ecx",
            out("eax") _,
            out("edx") _,
            out("ecx") _,
            options(nomem)
        );
    }
    loop {}
}

/// Deliberate invalid opcode test (invoked on demand)
pub fn trigger_invalid_opcode() -> ! {
    serial_println!("[TEST] Triggering Invalid Opcode (#UD)...");
    unsafe {
        core::arch::asm!("ud2", options(nomem, nostack));
    }
    loop {}
}

/// Deliberate page fault test (invoked on demand)
pub fn trigger_page_fault() -> ! {
    serial_println!("[TEST] Triggering Page Fault (#PF at 0xDEADBEEF)...");
    unsafe {
        let ptr = 0xDEAD_BEEFu64 as *mut u64;
        core::ptr::write_volatile(ptr, 0x1234_5678);
    }
    loop {}
}
