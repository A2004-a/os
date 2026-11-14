//! NovaOS Apex — Bare-Metal Kernel Entrypoint (Milestone 3)
//! Initializes GDT/TSS/IDT, 4-level paging, heap, objects, and all 10 first-party device drivers.

#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use novaos_apex_kernel::arch;
use novaos_apex_kernel::arch::x86_64::instructions::{hlt, sti};
use novaos_apex_kernel::boot::BootInfo;
use novaos_apex_kernel::drivers;
use novaos_apex_kernel::fs;
use novaos_apex_kernel::graphics;
use novaos_apex_kernel::installer;
use novaos_apex_kernel::ipc;
use novaos_apex_kernel::memory;
use novaos_apex_kernel::network;
use novaos_apex_kernel::objects;
use novaos_apex_kernel::package;
use novaos_apex_kernel::posix;
use novaos_apex_kernel::process;
use novaos_apex_kernel::recovery;
use novaos_apex_kernel::runtime;
use novaos_apex_kernel::scheduler;
use novaos_apex_kernel::security;
use novaos_apex_kernel::serial;
use novaos_apex_kernel::syscall;

const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64 KiB kernel stack

#[allow(dead_code)]
#[repr(align(16))]
struct KernelStack([u8; KERNEL_STACK_SIZE]);

static mut KERNEL_STACK: KernelStack = KernelStack([0u8; KERNEL_STACK_SIZE]);

#[no_mangle]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    unsafe {
        let stack_top = (addr_of_mut!(KERNEL_STACK) as u64) + (KERNEL_STACK_SIZE as u64);
        core::arch::asm!(
            "mov rsp, {0}",
            "mov rdi, {1}",
            "call kernel_entry",
            in(reg) stack_top,
            in(reg) boot_info,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub extern "C" fn kernel_entry(boot_info_ptr: *const BootInfo) -> ! {
    // 1. Initial Serial Milestone 4 Greeting
    serial::write_str("\nNovaOS Apex started\n");
    serial::write_str("=======================================================\n");
    serial::write_str("   NovaOS Apex — Bare-Metal Kernel Core (Milestone 4)  \n");
    serial::write_str("=======================================================\n");

    // 2. Validate BootInfo from UEFI Handoff
    if boot_info_ptr.is_null() {
        serial::write_str("[KERNEL FATAL] Received null BootInfo pointer from bootloader.\n");
        hlt();
    }

    let boot_info = unsafe { &*boot_info_ptr };
    if !boot_info.is_valid() {
        serial::write_str("[KERNEL WARN] BootInfo magic/version mismatch! Proceeding with caution.\n");
    } else {
        serial::write_str("[KERNEL] BootInfo validated successfully (magic=NOVAAPEX, version=1)\n");
        serial::write_str("[KERNEL] Framebuffer: ");
        print_u64(boot_info.horizontal_resolution as u64);
        serial::write_str("x");
        print_u64(boot_info.vertical_resolution as u64);
        serial::write_str(" @ 0x");
        print_hex_u64(boot_info.framebuffer_base);
        serial::write_str(" (Stride: ");
        print_u64(boot_info.pixels_per_scan_line as u64);
        serial::write_str(")\n");

        if boot_info.rsdp_address != 0 {
            serial::write_str("[KERNEL] ACPI RSDP Pointer: 0x");
            print_hex_u64(boot_info.rsdp_address);
            serial::write_str("\n");
        }

        serial::write_str("[KERNEL] UEFI Memory Map: ");
        print_u64(boot_info.memory_map_entries as u64);
        serial::write_str(" descriptors at 0x");
        print_hex_u64(boot_info.memory_map_addr);
        serial::write_str("\n");

        if boot_info.cmdline_len > 0 && boot_info.cmdline_addr != 0 {
            let cmdline_slice = unsafe {
                core::slice::from_raw_parts(boot_info.cmdline_addr as *const u8, boot_info.cmdline_len as usize)
            };
            if let Ok(cmdline_str) = core::str::from_utf8(cmdline_slice) {
                serial::write_str("[KERNEL] Boot Command Line: '");
                serial::write_str(cmdline_str);
                serial::write_str("'\n");
            }
        }
    }

    // 3. Initialize Architecture Subsystem (GDT, TSS, IDT)
    serial::write_str("[KERNEL] Initializing 64-bit GDT, TSS (IST1 32 KiB Emergency Stack), and IDT...\n");
    arch::init();
    serial::write_str("[KERNEL] CPU segment tables and 256 interrupt gates armed.\n");

    // 4. Run Milestone 1 Deliberate Exception & Fault Tests
    let m1_passed = arch::tests::run_all_tests();

    // 5. Initialize Milestone 2 Memory Subsystem
    serial::write_str("\n[KERNEL] Initializing Milestone 2 Memory Subsystem...\n");
    memory::init(boot_info);
    let m2_mem_passed = memory::tests::run_all_tests();
    let m2_obj_passed = objects::tests::run_object_tests();
    memory::stats::print_summary();
    process::init();

    // 6. Initialize Milestone 3 Device Drivers (in exact Section 10 order)
    serial::write_str("\n[KERNEL] Initializing Milestone 3 Device Drivers...\n");
    drivers::init(boot_info);

    // 8. Run Milestone 3 Device Driver Verification Suite
    let m3_drivers_passed = drivers::tests::run_all_tests();

    if m1_passed && m2_mem_passed && m2_obj_passed && m3_drivers_passed {
        serial::write_str("\n[MILESTONE 3 VERIFIED] All 10 first-party device drivers (UART, GOP,\n");
        serial::write_str("                       Console, Timer/APIC, Keyboard, Mouse, PCI, VirtIO-Block,\n");
        serial::write_str("                       VirtIO-Net, Audio) passed verification gates with zero errors!\n");
    } else {
        serial::write_str("\n[MILESTONE 3 WARN] Driver verification encountered anomalies.\n");
    }

    // 9. Initialize Milestone 5 Filesystem Subsystem (VFS, RAMFS, DevFS, NovaFS)
    serial::write_str("\n[KERNEL] Initializing Milestone 5 Filesystem Subsystem...\n");
    fs::init();
    let m5_fs_passed = fs::tests::run_m5_verification_suite();
    if m5_fs_passed {
        serial::write_str("\n[MILESTONE 5 VERIFIED] Filesystem and User Security subsystems confirmed!\n");
    } else {
        serial::write_str("\n[MILESTONE 5 WARN] Filesystem verification encountered anomalies.\n");
    }

    // 10. Initialize Milestone 6 Network Subsystem (Ethernet, ARP, IPv4, ICMP, UDP, TCP)
    serial::write_str("\n[KERNEL] Initializing Milestone 6 Network Subsystem...\n");
    network::init();
    let m6_net_passed = network::tests::run_m6_verification_suite();
    if m6_net_passed {
        serial::write_str("\n[MILESTONE 6 VERIFIED] VirtIO-Net, ARP, IPv4, ICMP, UDP, TCP 3-way handshake,\n");
        serial::write_str("                       and Multi-User socket security confirmed operational!\n");
    } else {
        serial::write_str("\n[MILESTONE 6 WARN] Network verification encountered anomalies.\n");
    }

    // 11. Initialize Milestone 7 Graphics & Window Server Subsystem
    serial::write_str("\n[KERNEL] Initializing Milestone 7 Graphics & Window Server Subsystem...\n");
    graphics::init();
    let m7_graphics_passed = graphics::tests::run_all();
    if m7_graphics_passed {
        serial::write_str("\n[MILESTONE 7 VERIFIED] 2D Software Compositor, Window Server, and NovaUI confirmed!\n");
    } else {
        serial::write_str("\n[MILESTONE 7 WARN] Graphics verification encountered anomalies.\n");
    }

    // 12. Run Milestone 8 Desktop Shell Verification Suite
    serial::write_str("\n[KERNEL] Initializing Milestone 8 Nova Desktop Shell...\n");
    let m8_shell_passed = graphics::shell_tests::run_m8_verification_suite();
    if m8_shell_passed {
        serial::write_str("\n[MILESTONE 8 VERIFIED] Nova Bar, Nova Launcher, Snap Assist, and Virtual Workspaces confirmed!\n");
    } else {
        serial::write_str("\n[MILESTONE 8 WARN] Desktop shell verification encountered anomalies.\n");
    }

    // 13. Run Milestone 9 Core Native Applications Verification Suite
    serial::write_str("\n[KERNEL] Initializing Milestone 9 Core Native Applications...\n");
    let m9_apps_passed = graphics::app_tests::run_m9_verification_suite();
    if m9_apps_passed {
        serial::write_str("\n[MILESTONE 9 VERIFIED] Nova Files, Nova System Monitor, Nova Settings,\n");
        serial::write_str("                       Nova Package Manager, and Nova Terminal confirmed operational!\n");
    } else {
        serial::write_str("\n[MILESTONE 9 WARN] Core native applications encountered anomalies.\n");
    }

    // 14. Initialize Milestone 10 App Platform, Installer & Recovery
    serial::write_str("\n[KERNEL] Initializing Milestone 10 App Platform, Installer & Recovery Subsystem...\n");
    package::init();
    installer::init();
    recovery::init();
    recovery::get_recovery().record_boot_start();

    let m10_passed = recovery::tests::run_m10_verification_suite();
    if m10_passed {
        serial::write_str("\n[MILESTONE 10 VERIFIED] Signed .nvp packages, GPT partitioner,\n");
        serial::write_str("                        automated OS installer, and recovery rollback confirmed operational!\n");
    } else {
        serial::write_str("\n[MILESTONE 10 WARN] Milestone 10 encountered anomalies.\n");
    }
    recovery::get_recovery().record_boot_success();

    // 15. Run Milestone 11 Hardware Expansion Verification Suite
    serial::write_str("\n[KERNEL] Initializing Milestone 11 Hardware Expansion Subsystem...\n");
    let m11_passed = drivers::hardware_tests::run_m11_verification_suite();
    if m11_passed {
        serial::write_str("\n[MILESTONE 11 VERIFIED] AHCI SATA 1.3, NVMe 1.4, USB 3.0 xHCI & HID,\n");
        serial::write_str("                        and Intel High Definition Audio (HDA) confirmed operational!\n");
    } else {
        serial::write_str("\n[MILESTONE 11 WARN] Hardware Expansion encountered anomalies.\n");
    }

    // 16. Run Milestone 12 Hardening & Release Engineering Verification Suite
    serial::write_str("\n[KERNEL] Initializing Milestone 12 Hardening & Release Subsystem...\n");
    let m12_passed = security::tests::run_m12_verification_suite();
    if m12_passed {
        serial::write_str("\n[MILESTONE 12 VERIFIED] Adversarial fuzzing, memory leak audits, stress stability,\n");
        serial::write_str("                        and production release conformance certified!\n");
    } else {
        serial::write_str("\n[MILESTONE 12 WARN] Hardening & Release verification encountered anomalies.\n");
    }

    // 16b. Run Comprehensive Keyboard & Mouse Subsystem Verification Suite (24 Gates)
    let input_passed = drivers::input_tests::run_all_input_tests();
    if input_passed {
        serial::write_str("[KERNEL] All 24 keyboard & mouse hardware gates certified!\n");
    } else {
        serial::write_str("[KERNEL WARN] Keyboard & mouse verification encountered failures.\n");
    }

    // 16c. Run Comprehensive Terminal & NovaShell Verification Suite (25 Gates)
    let term_passed = drivers::terminal_tests::run_all_terminal_tests();
    if term_passed {
        serial::write_str("[KERNEL] All 25 Terminal & NovaShell functional gates certified!\n");
    } else {
        serial::write_str("[KERNEL WARN] Terminal & NovaShell verification encountered failures.\n");
    }

    // 16d. Run Nova Store & Package Management Verification Suite (20 Gates)
    let store_passed = package::store_tests::run_all_store_tests();
    if store_passed {
        serial::write_str("[KERNEL] All 20 Nova Store & Package Management functional gates certified!\n");
    } else {
        serial::write_str("[KERNEL WARN] Nova Store verification encountered failures.\n");
    }

    // 16e. Run Windows 11 Desktop Features Verification Suite (105 Gates)
    let win105_passed = graphics::windows_features_test::run_all_105_windows_feature_tests();
    if win105_passed {
        serial::write_str("[KERNEL] Windows 11-class desktop subsystem certified with 105 passing features!\n");
    } else {
        serial::write_str("[KERNEL WARN] Some Windows 11 features did not pass verification.\n");
    }

    // 16f. Run Post-1.0 Milestone M11 (VirtIO-GPU, Clipboard Service, Drag-and-Drop) Verification Suite (16 Gates)
    let m11_post10_passed = drivers::m11_tests::run_all_m11_tests();
    if m11_post10_passed {
        serial::write_str("[KERNEL] Milestone M11 (VirtIO-GPU, Clipboard, DND) certified with 16 passing gates!\n");
    } else {
        serial::write_str("[KERNEL WARN] Milestone M11 encountered anomalies.\n");
    }

    // 17. Initialize Scheduler & IPC Subsystems
    scheduler::init();
    ipc::init();

    // 17b. Run Post-1.0 Milestone M12 (Userspace Threads, Futex, Dynamic Linking) Verification Suite (16 Gates)
    let m12_post10_passed = process::m12_tests::run_all_m12_tests();
    if m12_post10_passed {
        serial::write_str("[KERNEL] Milestone M12 (Userspace Threads, Futex, Dynamic Linking) certified with 16 passing gates!\n");
    } else {
        serial::write_str("[KERNEL WARN] Milestone M12 encountered anomalies.\n");
    }

    // 17c. Run Post-1.0 Milestone M13 (musl libc Port & POSIX Layer) Verification Suite (16 Gates)
    let m13_post10_passed = posix::m13_tests::run_all_m13_tests();
    if m13_post10_passed {
        serial::write_str("[KERNEL] Milestone M13 (musl libc Port & POSIX Layer) certified with 16 passing gates!\n");
    } else {
        serial::write_str("[KERNEL WARN] Milestone M13 encountered anomalies.\n");
    }

    // 17d. Run Post-1.0 Milestone M14 (Coreutils, MicroPython, WASM Runtime) Verification Suite (16 Gates)
    let m14_post10_passed = runtime::m14_tests::run_all_m14_tests();
    if m14_post10_passed {
        serial::write_str("[KERNEL] Milestone M14 (Coreutils, MicroPython, WASM) certified with 16 passing gates!\n");
    } else {
        serial::write_str("[KERNEL WARN] Milestone M14 encountered anomalies.\n");
    }

    // 14. Run Milestone 4 Verification Test Suite & Spawn PID 1
    static INIT_ELF: &[u8] = include_bytes!("../../target/x86_64-unknown-none/debug/novaos-apex-init");
    syscall::tests::run_m4_verification_suite(INIT_ELF);

    // Refresh and cleanly composite the primary Windows 11 desktop after all boot tests & init finish
    graphics::redraw_all();

    // 13. Enable Hardware Interrupts & Handoff to Preemptive Multitasking
    serial::write_str("[KERNEL] Enabling hardware interrupts via STI and entering multitasking...\n");
    sti();

    // 12. Stable CPU Halt Loop (polls keyboard/mouse inputs and halts when idle)
    loop {
        if let Some(mut shell_guard) = crate::graphics::desktop::DESKTOP.try_lock() {
            if let Some(shell) = shell_guard.as_mut() {
                shell.poll_input();
            }
        }
        hlt();
    }
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
