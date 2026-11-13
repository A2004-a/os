#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use task::executor::Executor;
use task::keyboard;
use task::Task;
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 1. Initialize Early Debug Console via UART 16550 Serial
    serial::init();
    serial_println!("========================================");
    serial_println!("   NovaOS x86_64 Bare-Metal Kernel     ");
    serial_println!("========================================");
    serial_println!("[BOOT] CPU long-mode active");

    // 2. Clear and format VGA text buffer
    println!("NovaOS Kernel v0.1.0 [x86_64-bare-metal]");
    println!("Copyright (c) 2026 NovaOS Systems Engineering\n");

    // 3. Initialize Segmentation & Global Descriptor Table (with TSS)
    print!("[INIT] Loading GDT & TSS... ");
    gdt::init();
    println!("[OK]");
    serial_println!("[INIT] GDT loaded with emergency Double Fault stack");

    // 4. Initialize Interrupt Descriptor Table (IDT)
    print!("[INIT] Loading IDT... ");
    interrupts::init_idt();
    println!("[OK]");
    serial_println!("[INIT] IDT registered for CPU exceptions and IRQs");

    // 5. Initialize 8259 PIC and enable CPU hardware interrupts
    print!("[INIT] Remapping PIC & enabling interrupts... ");
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    println!("[OK]");
    serial_println!("[INIT] 8259 PIC unmasked; IRQ0 (Timer) and IRQ1 (Keyboard) active");

    // 6. Initialize 4-Level Paging and Physical Memory Frame Allocator
    print!("[INIT] Initializing 4-level paging... ");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    println!("[OK]");
    serial_println!("[INIT] Virtual memory paging operational");

    // 7. Initialize Dynamic Kernel Heap Allocator
    print!("[INIT] Allocating kernel heap (256 KiB)... ");
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Kernel heap initialization failed");
    println!("[OK]");
    serial_println!("[INIT] Kernel heap active at virtual address 0x4444_4444_0000");

    // 8. Test Dynamic Memory Allocations
    let test_box = Box::new(42);
    let mut test_vec = Vec::new();
    for i in 0..10 {
        test_vec.push(i);
    }
    let test_str = String::from("NovaOS Native Allocator Ready");
    println!("[HEAP] Self-test verified: Box={}, Vec_len={}, Str='{}'", test_box, test_vec.len(), test_str);

    #[cfg(test)]
    test_main();

    println!("\nNovaOS prompt ready. Type on the keyboard:");
    print!("nova> ");

    // 9. Launch Async/Await Preemptive Task Executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

/// Panic handler called on kernel assertion failure or unrecoverable error.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n[KERNEL PANIC]\n{}", info);
    serial_println!("\n[KERNEL PANIC]\n{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
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
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
