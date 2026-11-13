//! NovaOS x86_64 Interrupt Descriptor Table (IDT)
//! Handles CPU internal exceptions and hardware IRQ routing

use crate::arch::x86_64::gdt;
use crate::serial_println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // Hardware IRQ vectors: IRQ0 (Timer) = 32, IRQ1 (Keyboard) = 33
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt[33].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    serial_println!("[CPU EXCEPTION: DIVISION BY ZERO]");
    serial_println!("{:#?}", stack_frame);
    panic!("Division by zero exception occurred.");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("[CPU TRAP: BREAKPOINT]");
    serial_println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    serial_println!("[CPU EXCEPTION: INVALID OPCODE]");
    serial_println!("{:#?}", stack_frame);
    panic!("Invalid CPU instruction executed.");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println!("[CPU EXCEPTION: GENERAL PROTECTION FAULT]");
    serial_println!("Error Code: 0x{:X}", error_code);
    serial_println!("{:#?}", stack_frame);
    panic!("General protection fault encountered.");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[FATAL EXCEPTION: DOUBLE FAULT]");
    serial_println!("{:#?}", stack_frame);
    panic!("Fatal double fault exception triggered.");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use crate::arch::x86_64::registers::ControlRegisters;

    let fault_addr = ControlRegisters::read_cr2();
    serial_println!("[CPU EXCEPTION: PAGE FAULT]");
    serial_println!("Faulting Address (CR2): 0x{:016X}", fault_addr);
    serial_println!("Flags: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);
    panic!("Unrecoverable page fault at address 0x{:016X}", fault_addr);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::interrupts::timer::on_timer_tick();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::ps2_keyboard::on_keyboard_interrupt();
}
