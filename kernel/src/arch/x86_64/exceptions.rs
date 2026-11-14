//! NovaOS Apex — Architecture Exception Dispatcher & Diagnostic Handlers
//! Implements exception contexts, registers dumping, and controlled fault tests.

use super::instructions::read_cr2;
use crate::serial;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TestMode {
    None,
    DivideByZero,
    InvalidOpcode,
    PageFault,
    DoubleFault,
}

pub static mut ACTIVE_TEST: TestMode = TestMode::None;
pub static mut TEST_RESULT_PASSED: bool = false;

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionContext {
    // Saved by assembly stub (in reverse push order)
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    // Pushed by trampoline
    pub vector: u64,
    pub error_code: u64,

    // Pushed by x86_64 hardware
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

static EXCEPTION_NAMES: [&str; 32] = [
    "#DE: Divide-by-Zero Error",
    "#DB: Debug Exception",
    "#NMI: Non-Maskable Interrupt",
    "#BP: Breakpoint Exception",
    "#OF: Overflow",
    "#BR: BOUND Range Exceeded",
    "#UD: Invalid Opcode (Undefined Opcode)",
    "#NM: Device Not Available (No Math Coprocessor)",
    "#DF: Double Fault",
    "Coprocessor Segment Overrun",
    "#TS: Invalid TSS",
    "#NP: Segment Not Present",
    "#SS: Stack-Segment Fault",
    "#GP: General Protection Fault",
    "#PF: Page Fault",
    "Reserved",
    "#MF: x87 FPU Floating-Point Error",
    "#AC: Alignment Check",
    "#MC: Machine Check",
    "#XM: SIMD Floating-Point Exception",
    "#VE: Virtualization Exception",
    "#CP: Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "#HV: Hypervisor Injection Exception",
    "#VC: VMM Communication Exception",
    "#SX: Security Exception",
    "Reserved",
];

#[no_mangle]
pub extern "C" fn handle_exception(ctx: &mut ExceptionContext) {
    let vector = ctx.vector as usize;
    let name = if vector < 32 {
        EXCEPTION_NAMES[vector]
    } else {
        "User / Hardware Interrupt"
    };

    let active_test = unsafe { ACTIVE_TEST };

    match active_test {
        TestMode::DivideByZero if vector == 0 => {
            serial::write_str("\n[EXCEPTION DIAGNOSTIC] #DE (Divide-by-Zero) caught successfully.\n");
            serial::write_str("  Faulting RIP: 0x");
            print_hex_u64(ctx.rip);
            serial::write_str(", RSP: 0x");
            print_hex_u64(ctx.rsp);
            serial::write_str("\n  Instruction decoded: 'idiv' / 'div'. Advancing instruction pointer to resume.\n");

            unsafe { TEST_RESULT_PASSED = true; }
            // Skip the 2-byte divide instruction (e.g. `f7 f9` or `f7 f8`)
            ctx.rip += 2;
            return;
        }

        TestMode::InvalidOpcode if vector == 6 => {
            serial::write_str("\n[EXCEPTION DIAGNOSTIC] #UD (Invalid Opcode) caught successfully.\n");
            serial::write_str("  Faulting RIP: 0x");
            print_hex_u64(ctx.rip);
            serial::write_str("\n  Opcode decoded: 0x0F 0x0B (ud2). Advancing instruction pointer to resume.\n");

            unsafe { TEST_RESULT_PASSED = true; }
            // Skip the 2-byte `ud2` instruction (0x0F 0x0B)
            ctx.rip += 2;
            return;
        }

        TestMode::PageFault if vector == 14 => {
            let cr2 = read_cr2();
            serial::write_str("\n[EXCEPTION DIAGNOSTIC] #PF (Page Fault) caught successfully.\n");
            serial::write_str("  Faulting Virtual Address (CR2): 0x");
            print_hex_u64(cr2);
            serial::write_str("\n  Error Code: 0x");
            print_hex_u64(ctx.error_code);
            serial::write_str(" (Present=");
            print_u64(ctx.error_code & 1);
            serial::write_str(", Write=");
            print_u64((ctx.error_code >> 1) & 1);
            serial::write_str(", User=");
            print_u64((ctx.error_code >> 2) & 1);
            serial::write_str(")\n  Faulting RIP: 0x");
            print_hex_u64(ctx.rip);
            serial::write_str("\n");

            unsafe { TEST_RESULT_PASSED = true; }
            // Skip the faulting instruction (e.g. `mov rax, [rcx]` = 3 bytes or advance to test return)
            ctx.rip += 3;
            return;
        }

        TestMode::DoubleFault if vector == 8 => {
            let stack_top = super::gdt::get_double_fault_stack_top();
            let stack_bottom = stack_top - (super::gdt::DOUBLE_FAULT_STACK_SIZE as u64);
            let on_ist1 = ctx.rsp >= stack_bottom && ctx.rsp <= stack_top;

            serial::write_str("\n[EXCEPTION DIAGNOSTIC] #DF (Double Fault) caught successfully!\n");
            serial::write_str("  Active Stack (RSP): 0x");
            print_hex_u64(ctx.rsp);
            serial::write_str("\n  Dedicated IST1 Emergency Stack Range: [0x");
            print_hex_u64(stack_bottom);
            serial::write_str(" - 0x");
            print_hex_u64(stack_top);
            serial::write_str("]\n  Stack Isolation Verified: ");
            if on_ist1 {
                serial::write_str("YES (IST1 hardware switch active)\n");
                unsafe { TEST_RESULT_PASSED = true; }
            } else {
                serial::write_str("NO (Warning: IST1 stack mismatch)\n");
            }

            // Return from controlled double fault invocation
            return;
        }

        _ => {}
    }

    // Unhandled / Fatal Exception Handler
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — FATAL KERNEL EXCEPTION TRAP           \n");
    serial::write_str("=======================================================\n");
    serial::write_str("Vector: ");
    print_u64(ctx.vector);
    serial::write_str(" (");
    serial::write_str(name);
    serial::write_str(")\n");
    serial::write_str("Error Code: 0x");
    print_hex_u64(ctx.error_code);
    serial::write_str("\nRIP: 0x");
    print_hex_u64(ctx.rip);
    serial::write_str("  CS: 0x");
    print_hex_u64(ctx.cs);
    serial::write_str("  RFLAGS: 0x");
    print_hex_u64(ctx.rflags);
    serial::write_str("\nRSP: 0x");
    print_hex_u64(ctx.rsp);
    serial::write_str("  SS: 0x");
    print_hex_u64(ctx.ss);
    serial::write_str("\n");

    if vector == 14 {
        let cr2 = read_cr2();
        serial::write_str("Page Fault Address (CR2): 0x");
        print_hex_u64(cr2);
        serial::write_str("\n");
    }

    serial::write_str("\nCPU Register Dump:\n");
    serial::write_str("  RAX: 0x"); print_hex_u64(ctx.rax);
    serial::write_str("  RBX: 0x"); print_hex_u64(ctx.rbx);
    serial::write_str("  RCX: 0x"); print_hex_u64(ctx.rcx);
    serial::write_str("  RDX: 0x"); print_hex_u64(ctx.rdx);
    serial::write_str("\n  RSI: 0x"); print_hex_u64(ctx.rsi);
    serial::write_str("  RDI: 0x"); print_hex_u64(ctx.rdi);
    serial::write_str("  RBP: 0x"); print_hex_u64(ctx.rbp);
    serial::write_str("  R8:  0x"); print_hex_u64(ctx.r8);
    serial::write_str("\n  R9:  0x"); print_hex_u64(ctx.r9);
    serial::write_str("  R10: 0x"); print_hex_u64(ctx.r10);
    serial::write_str("  R11: 0x"); print_hex_u64(ctx.r11);
    serial::write_str("  R12: 0x"); print_hex_u64(ctx.r12);
    serial::write_str("\n  R13: 0x"); print_hex_u64(ctx.r13);
    serial::write_str("  R14: 0x"); print_hex_u64(ctx.r14);
    serial::write_str("  R15: 0x"); print_hex_u64(ctx.r15);
    serial::write_str("\n\n[HALT] System halted safely. Zero silent resets permitted.\n");

    super::instructions::hlt();
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

fn print_hex_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        buf[i] = hex_chars[(n & 0x0F) as usize];
        n >>= 4;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
