//! NovaOS Apex — Bare-Metal Kernel Entrypoint (Milestone 1)
//! Boots from UEFI handoff, sets up kernel stack, validates BootInfo,
//! initializes GDT/TSS/IDT, and executes deliberate fault tests.

#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use novaos_apex_kernel::arch;
use novaos_apex_kernel::arch::x86_64::instructions::hlt;
use novaos_apex_kernel::boot::BootInfo;
use novaos_apex_kernel::serial;

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
    // 1. Initial Serial Milestone 1 Greeting
    serial::write_str("\nNovaOS Apex started\n");
    serial::write_str("=======================================================\n");
    serial::write_str("   NovaOS Apex — Bare-Metal Kernel Core (Milestone 1)  \n");
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

    // 4. Run Deliberate Exception & Fault Tests
    let tests_passed = arch::tests::run_all_tests();

    if tests_passed {
        serial::write_str("\n[MILESTONE 1 VERIFIED] UEFI bootloader, kernel entry, GDT, TSS, IDT,\n");
        serial::write_str("                       and deliberate exception recovery passed with zero errors!\n");
    } else {
        serial::write_str("\n[MILESTONE 1 WARN] Exception verification encountered anomalies.\n");
    }

    // 5. Stable CPU Halt Loop
    serial::write_str("[KERNEL] Entering stable CPU halt state. Awaiting hardware events...\n");
    hlt();
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
