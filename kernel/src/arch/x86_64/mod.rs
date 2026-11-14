//! NovaOS Apex — x86_64 Architecture Support Module

pub mod exceptions;
pub mod gdt;
pub mod idt;
pub mod instructions;
pub mod tests;

pub fn init() {
    // 1. Initialize Global Descriptor Table (GDT) and Task State Segment (TSS with IST1)
    gdt::init();

    // 2. Initialize Interrupt Descriptor Table (IDT) with exception handlers & IST1 for #DF
    idt::init();
}
