#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod arch;
pub mod boot;
pub mod cpu;
pub mod drivers;
pub mod fs;
pub mod graphics;
pub mod interrupts;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod security;
pub mod storage;
pub mod syscall;

use boot::info::BootInfo;
use core::panic::PanicInfo;
use x86_64::VirtAddr;

pub fn init(boot_info: &BootInfo) {
    // 1. Initialize serial port first for early diagnostics
    drivers::serial::init();
    serial_println!("========================================");
    serial_println!("       NovaOS Zero x86_64 Microkernel   ");
    serial_println!("========================================");

    // 2. Load GDT and TSS with emergency IST stack
    arch::x86_64::gdt::init();
    serial_println!("[INIT] GDT and Task State Segment active");

    // 3. Load IDT with 32 CPU exceptions and hardware IRQs
    arch::x86_64::idt::init();
    serial_println!("[INIT] IDT registered for CPU exceptions");

    // 4. Remap 8259 PIC and enable hardware timer/keyboard interrupts
    interrupts::init();
    arch::x86_64::interrupts::enable();
    serial_println!("[INIT] 8259 PIC unmasked; IRQ interrupts enabled");

    // 5. Initialize 4-level paging and physical frame allocator
    let phys_offset = VirtAddr::new(0xFFFF_8000_0000_0000);
    let mut mapper = unsafe { memory::paging::init_paging(phys_offset) };
    let mut frame_allocator = memory::frame::BootInfoFrameAllocator::new(
        boot_info.memory_map.regions,
        boot_info.memory_map.regions_count,
    );
    serial_println!("[INIT] Virtual memory 4-level paging initialized");

    // 6. Initialize dynamic kernel heap (256 KiB)
    memory::heap::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Kernel heap initialization failed");
    serial_println!("[INIT] Kernel heap active at 0x4444_4444_0000");

    // 7. Initialize hardware drivers (VGA, Framebuffer, Keyboard)
    drivers::init(boot_info.framebuffer);
    serial_println!("[INIT] Drivers initialized (UART, VGA, Framebuffer, PS/2 Keyboard)");
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    arch::x86_64::interrupts::hlt_loop();
}
