//! NovaOS Apex — 64-bit UEFI Boot Manager (BOOTX64.EFI)
//! Milestone 1: ELF64 Kernel Loading, GOP Discovery, ACPI RSDP, Memory Map & Kernel Handoff

#![no_std]
#![no_main]

pub mod boot_info;
pub mod config;
pub mod elf;
pub mod fs;
pub mod serial;
pub mod uefi;

use boot_info::{BootInfo, BOOT_INFO_MAGIC, BOOT_INFO_VERSION};
use config::BootConfig;
use core::panic::PanicInfo;
use fs::UefiVolume;
use uefi::*;

static mut STATIC_BOOT_INFO: BootInfo = BootInfo {
    magic: BOOT_INFO_MAGIC,
    version: BOOT_INFO_VERSION,
    flags: 0,
    framebuffer_base: 0,
    framebuffer_size: 0,
    horizontal_resolution: 0,
    vertical_resolution: 0,
    pixels_per_scan_line: 0,
    pixel_format: 0,
    rsdp_address: 0,
    memory_map_addr: 0,
    memory_map_size: 0,
    descriptor_size: 0,
    descriptor_version: 0,
    memory_map_entries: 0,
    kernel_virt_base: 0,
    kernel_phys_base: 0,
    kernel_size: 0,
    kernel_entry: 0,
    cmdline_addr: 0,
    cmdline_len: 0,
};

static mut CMDLINE_BUFFER: [u8; 128] = [0u8; 128];

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: usize, system_table: *mut SystemTable) -> usize {
    // 1. Initialize COM1 UART serial port
    serial::init();
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — 64-bit UEFI Boot Manager (M1)         \n");
    serial::write_str("=======================================================\n");
    serial::write_str("[BOOT] Initializing UEFI 2.10 platform services...\n");

    let st = unsafe {
        if system_table.is_null() {
            serial::write_str("[FATAL] SystemTable pointer is null\n");
            loop { core::hint::spin_loop(); }
        }
        &*system_table
    };

    let bs = unsafe {
        if st.boot_services.is_null() {
            serial::write_str("[FATAL] BootServices pointer is null\n");
            loop { core::hint::spin_loop(); }
        }
        &*st.boot_services
    };

    // 2. Output status banner to UEFI graphical console
    if !st.con_out.is_null() {
        let msg = [
            'N' as u16, 'o' as u16, 'v' as u16, 'a' as u16, 'O' as u16, 'S' as u16, ' ' as u16,
            'A' as u16, 'p' as u16, 'e' as u16, 'x' as u16, ' ' as u16,
            'U' as u16, 'E' as u16, 'F' as u16, 'I' as u16, ' ' as u16,
            'L' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 'e' as u16, 'r' as u16,
            '\r' as u16, '\n' as u16, 0
        ];
        unsafe {
            let con_out = &*st.con_out;
            (con_out.output_string)(st.con_out, msg.as_ptr());
        }
    }

    // 3. Read Boot Configuration (/novaos.cfg)
    let mut boot_cfg = BootConfig::default();
    let volume = UefiVolume::open_boot_volume(image_handle, bs);

    if let Some(ref vol) = volume {
        serial::write_str("[BOOT] Opened UEFI boot volume successfully.\n");
        if let Some((cfg_bytes, _)) = vol.read_file("\\novaos.cfg", bs) {
            serial::write_str("[BOOT] Loaded /novaos.cfg from boot volume.\n");
            boot_cfg = BootConfig::parse(cfg_bytes);
        } else if let Some((cfg_bytes, _)) = vol.read_file("\\EFI\\BOOT\\novaos.cfg", bs) {
            serial::write_str("[BOOT] Loaded /EFI/BOOT/novaos.cfg.\n");
            boot_cfg = BootConfig::parse(cfg_bytes);
        } else {
            serial::write_str("[BOOT] Using internal default boot configuration.\n");
        }
    } else {
        serial::write_str("[WARN] Could not open boot filesystem volume directly.\n");
    }

    // 4. Locate and load kernel ELF64 image
    let mut loaded_kernel_info = None;
    if let Some(ref vol) = volume {
        let kernel_path = core::str::from_utf8(&boot_cfg.kernel_path[..boot_cfg.kernel_path_len])
            .unwrap_or("\\novaos-apex.elf");

        serial::write_str("[BOOT] Searching for kernel image: ");
        serial::write_str(kernel_path);
        serial::write_str("\n");

        if let Some((elf_bytes, _)) = vol.read_file(kernel_path, bs) {
            serial::write_str("[BOOT] Kernel image read (");
            print_u64(elf_bytes.len() as u64);
            serial::write_str(" bytes). Parsing ELF64 structure...\n");

            match elf::validate_and_load(elf_bytes, bs) {
                Ok(info) => {
                    serial::write_str("[ELF64] Validation passed. Entry point: 0x");
                    print_hex_u64(info.entry_point);
                    serial::write_str(", Physical Base: 0x");
                    print_hex_u64(info.phys_base);
                    serial::write_str(", Virtual Base: 0x");
                    print_hex_u64(info.virt_base);
                    serial::write_str(", Size: ");
                    print_u64(info.total_size);
                    serial::write_str(" bytes\n");
                    loaded_kernel_info = Some(info);
                }
                Err(e) => {
                    serial::write_str("[ERROR] ELF validation failed: ");
                    match e {
                        elf::ElfError::InvalidMagic => serial::write_str("InvalidMagic\n"),
                        elf::ElfError::Not64Bit => serial::write_str("Not64Bit\n"),
                        elf::ElfError::NotLittleEndian => serial::write_str("NotLittleEndian\n"),
                        elf::ElfError::UnsupportedArchitecture => serial::write_str("UnsupportedArchitecture\n"),
                        elf::ElfError::NoLoadableSegments => serial::write_str("NoLoadableSegments\n"),
                        elf::ElfError::AllocationFailed => serial::write_str("AllocationFailed\n"),
                        elf::ElfError::OutOfBounds => serial::write_str("OutOfBounds\n"),
                    }
                }
            }
        } else {
            serial::write_str("[WARN] Kernel not found at primary path, checking fallback /EFI/BOOT/novaos-apex.elf...\n");
            if let Some((elf_bytes, _)) = vol.read_file("\\EFI\\BOOT\\novaos-apex.elf", bs) {
                if let Ok(info) = elf::validate_and_load(elf_bytes, bs) {
                    serial::write_str("[ELF64] Loaded kernel from fallback path.\n");
                    loaded_kernel_info = Some(info);
                }
            }
        }
    }

    // 5. Discover GOP Framebuffer
    let mut gop_base = 0u64;
    let mut gop_size = 0u64;
    let mut gop_w = 0u32;
    let mut gop_h = 0u32;
    let mut gop_stride = 0u32;
    let mut gop_format = 0u32;

    let mut gop_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    let gop_status = unsafe {
        (bs.locate_protocol)(&GOP_GUID, core::ptr::null(), &mut gop_ptr)
    };

    if gop_status == 0 && !gop_ptr.is_null() {
        let gop = unsafe { &*(gop_ptr as *mut GraphicsOutputProtocol) };
        if !gop.mode.is_null() {
            let mode = unsafe { &*gop.mode };
            gop_base = mode.framebuffer_base;
            gop_size = mode.framebuffer_size as u64;

            if !mode.info.is_null() {
                let info = unsafe { &*mode.info };
                gop_w = info.horizontal_resolution;
                gop_h = info.vertical_resolution;
                gop_stride = info.pixels_per_scan_line;
                gop_format = info.pixel_format;

                serial::write_str("[GOP] Framebuffer active: ");
                print_u64(gop_w as u64);
                serial::write_str("x");
                print_u64(gop_h as u64);
                serial::write_str(" @ 0x");
                print_hex_u64(gop_base);
                serial::write_str(" (Stride: ");
                print_u64(gop_stride as u64);
                serial::write_str(")\n");
            }
        }
    }

    // 6. Discover ACPI RSDP Pointer
    let mut rsdp_addr: u64 = 0;
    if !st.configuration_table.is_null() && st.number_of_table_entries > 0 {
        let tables = unsafe {
            core::slice::from_raw_parts(st.configuration_table, st.number_of_table_entries)
        };
        for entry in tables {
            if entry.vendor_guid == ACPI_20_TABLE_GUID {
                rsdp_addr = entry.vendor_table as u64;
                serial::write_str("[ACPI] Discovered ACPI 2.0 RSDP at: 0x");
                print_hex_u64(rsdp_addr);
                serial::write_str("\n");
                break;
            } else if entry.vendor_guid == ACPI_10_TABLE_GUID && rsdp_addr == 0 {
                rsdp_addr = entry.vendor_table as u64;
                serial::write_str("[ACPI] Discovered ACPI 1.0 RSDP at: 0x");
                print_hex_u64(rsdp_addr);
                serial::write_str("\n");
            }
        }
    }

    // 7. Collect UEFI Memory Map
    let mut mem_map_size: usize = 0;
    let mut map_key: usize = 0;
    let mut desc_size: usize = 0;
    let mut desc_version: u32 = 0;

    // First call to determine buffer size
    unsafe {
        (bs.get_memory_map)(
            &mut mem_map_size,
            core::ptr::null_mut(),
            &mut map_key,
            &mut desc_size,
            &mut desc_version,
        );
    }

    // Allocate additional space for memory map descriptors added during allocation
    mem_map_size += 8 * 4096;
    let mem_map_pages = (mem_map_size + 4095) / 4096;
    let mut mem_map_buf_addr: u64 = 0;
    let alloc_mmap_status = unsafe {
        (bs.allocate_pages)(0, 2, mem_map_pages, &mut mem_map_buf_addr)
    };

    if alloc_mmap_status != 0 {
        serial::write_str("[FATAL] Failed to allocate buffer for UEFI memory map\n");
        loop { core::hint::spin_loop(); }
    }

    let mmap_status = unsafe {
        (bs.get_memory_map)(
            &mut mem_map_size,
            mem_map_buf_addr as *mut u8,
            &mut map_key,
            &mut desc_size,
            &mut desc_version,
        )
    };

    if mmap_status != 0 {
        serial::write_str("[FATAL] Failed to retrieve UEFI memory map\n");
        loop { core::hint::spin_loop(); }
    }

    let entries_count = if desc_size > 0 { (mem_map_size / desc_size) as u32 } else { 0 };
    serial::write_str("[MEM] UEFI Memory Map collected: ");
    print_u64(entries_count as u64);
    serial::write_str(" descriptors (size=");
    print_u64(mem_map_size as u64);
    serial::write_str(" bytes)\n");

    // 8. Populate Stable BootInfo Structure
    let boot_info_ptr = core::ptr::addr_of_mut!(STATIC_BOOT_INFO);
    let boot_info = unsafe { &mut *boot_info_ptr };
    boot_info.magic = BOOT_INFO_MAGIC;
    boot_info.version = BOOT_INFO_VERSION;
    boot_info.flags = 0;

    boot_info.framebuffer_base = gop_base;
    boot_info.framebuffer_size = gop_size;
    boot_info.horizontal_resolution = gop_w;
    boot_info.vertical_resolution = gop_h;
    boot_info.pixels_per_scan_line = gop_stride;
    boot_info.pixel_format = gop_format;

    boot_info.rsdp_address = rsdp_addr;

    boot_info.memory_map_addr = mem_map_buf_addr;
    boot_info.memory_map_size = mem_map_size as u64;
    boot_info.descriptor_size = desc_size as u64;
    boot_info.descriptor_version = desc_version;
    boot_info.memory_map_entries = entries_count;

    if let Some(ref kernel) = loaded_kernel_info {
        boot_info.kernel_virt_base = kernel.virt_base;
        boot_info.kernel_phys_base = kernel.phys_base;
        boot_info.kernel_size = kernel.total_size;
        boot_info.kernel_entry = kernel.entry_point;
    } else {
        // Fallback default values
        boot_info.kernel_virt_base = 0xFFFF_8000_0020_0000;
        boot_info.kernel_phys_base = 0x0020_0000;
        boot_info.kernel_size = 64 * 1024;
        boot_info.kernel_entry = 0x0020_1270;
    }

    // Copy command line to static buffer
    let cmdline_len = boot_cfg.cmdline_len.min(127);
    unsafe {
        let cmdline_buf_ptr = core::ptr::addr_of_mut!(CMDLINE_BUFFER) as *mut u8;
        core::ptr::copy_nonoverlapping(boot_cfg.cmdline.as_ptr(), cmdline_buf_ptr, cmdline_len);
        boot_info.cmdline_addr = cmdline_buf_ptr as u64;
        boot_info.cmdline_len = cmdline_len as u64;
    }

    serial::write_str("[BOOT] BootInfo structure initialized successfully.\n");
    serial::write_str("[BOOT] Exiting UEFI Boot Services (firmware handoff)...\n");

    // 9. Exit UEFI Boot Services
    let exit_status = unsafe {
        (bs.exit_boot_services)(image_handle, map_key)
    };

    if exit_status != 0 {
        // Memory map changed between get_memory_map and exit_boot_services; re-acquire and retry once
        serial::write_str("[WARN] Retrying ExitBootServices with updated memory map key...\n");
        let _ = unsafe {
            (bs.get_memory_map)(
                &mut mem_map_size,
                mem_map_buf_addr as *mut u8,
                &mut map_key,
                &mut desc_size,
                &mut desc_version,
            )
        };
        let exit_retry = unsafe {
            (bs.exit_boot_services)(image_handle, map_key)
        };
        if exit_retry != 0 {
            serial::write_str("[FATAL] ExitBootServices failed\n");
            loop { core::hint::spin_loop(); }
        }
    }

    // 10. Hand off execution to NovaOS Apex Kernel
    let kernel_entry_addr = boot_info.kernel_entry;
    let boot_info_ptr = boot_info as *const BootInfo as u64;

    serial::write_str("[BOOT] Jumping to NovaOS Apex Kernel at physical entry 0x");
    print_hex_u64(kernel_entry_addr);
    serial::write_str("\n");

    unsafe {
        core::arch::asm!(
            "mov rdi, {0}",
            "jmp {1}",
            in(reg) boot_info_ptr,
            in(reg) kernel_entry_addr,
            options(noreturn)
        );
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::write_str("\n[BOOT PANIC] ");
    if let Some(location) = info.location() {
        serial::write_str("at ");
        serial::write_str(location.file());
        serial::write_str(":");
        print_u64(location.line() as u64);
        serial::write_str("\n");
    }
    loop {
        core::hint::spin_loop();
    }
}
