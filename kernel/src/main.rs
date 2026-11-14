#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(novaos_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use novaos_kernel::boot::info::BootInfo;
use novaos_kernel::{print, println, serial_println};

#[no_mangle]
pub extern "C" fn _start(boot_info_ptr: *const BootInfo) -> ! {
    kernel_entry(boot_info_ptr);
}

#[no_mangle]
pub extern "C" fn kernel_entry(boot_info_ptr: *const BootInfo) -> ! {
    let boot_info = unsafe {
        if boot_info_ptr.is_null() {
            panic!("Kernel received null BootInfo pointer");
        }
        &*boot_info_ptr
    };

    if !boot_info.is_valid() {
        panic!("Invalid BootInfo magic header");
    }

    // Initialize all kernel subsystems
    novaos_kernel::init(boot_info);

    // Run Milestone M1 CPU architecture exception diagnostic suite
    novaos_kernel::arch::x86_64::exceptions_test::run_exception_diagnostics();

    // Run Milestone M2 Memory, Paging, and Object diagnostic suite
    novaos_kernel::memory::memory_test::run_memory_diagnostics(boot_info);

    // Run Milestone M3 Hardware Driver and Storage diagnostic suite
    novaos_kernel::drivers::drivers_test::run_driver_diagnostics();

    // Run Milestone M4 Multitasking Scheduler, Syscall, and ELF Loader verification suite
    novaos_kernel::scheduler::scheduler_test::run_m4_tests();

    // Run Milestone M5 Virtual Filesystem, NovaFS, and Storage verification suite
    novaos_kernel::fs::fs_test::run_fs_diagnostics();

    // Run Milestone M6 VirtIO-Net, Network Protocols, and Multi-User verification suite
    novaos_kernel::net::net_test::run_network_diagnostics();

    // Run Milestone M7 2D Compositor, Window Server, and NovaUI verification suite
    novaos_kernel::graphics::graphics_test::run_graphics_diagnostics();

    // Run Milestone M8 Desktop Shell, Taskbar, Launcher, and Workspaces verification suite
    novaos_kernel::graphics::shell_test::run_shell_diagnostics();

    // Run Milestone M9 Package Manager, App Model & Native Applications verification suite
    novaos_kernel::process::pkg_test::run_pkg_diagnostics();

    // Run Milestone M10 GPT Partitioning, NovaFS Installer & Recovery verification suite
    novaos_kernel::storage::installer_test::run_installer_diagnostics();

    // Run Milestone M11 Hardware Expansion (AHCI, NVMe, xHCI, Audio) verification suite
    novaos_kernel::drivers::hardware_test::run_hardware_diagnostics();

    println!("NovaOS Bare-Metal Kernel v0.2.0 [x86_64-uefi]");
    println!("Copyright (c) 2026 NovaOS Systems Engineering\n");

    println!("[CPU] Features: {:?}", novaos_kernel::cpu::detect_features());
    println!("[PCI] Probing bus for hardware controllers...");
    let pci_devices = novaos_kernel::drivers::pci::scan_pci_bus();
    for dev in pci_devices.iter().flatten() {
        println!("  - Bus {:02X}:{:02X}: Vendor 0x{:04X}, Device 0x{:04X}", dev.bus, dev.device, dev.vendor_id, dev.device_id);
    }

    // Graphical display test: Draw branding rectangle and title on linear framebuffer
    novaos_kernel::graphics::draw_rect(10, 10, 300, 60, 0x1E, 0x3A, 0x8A);
    novaos_kernel::graphics::draw_string(24, 30, "NOVAOS GRAPHICS READY", 0xFF, 0xFF, 0xFF);

    #[cfg(test)]
    test_main();

    println!("\nNovaOS Kernel initialized. Type on PS/2 Keyboard:");
    print!("nova> ");

    novaos_kernel::arch::x86_64::interrupts::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n[KERNEL PANIC]\n{}", info);
    serial_println!("\n[KERNEL PANIC]\n{}", info);
    novaos_kernel::arch::x86_64::interrupts::hlt_loop();
}
