pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod io;
pub mod registers;

pub fn init() {
    gdt::init();
    idt::init();
}
