//! NovaOS Zero — 64-Bit UEFI Boot Manager (BOOTX64.EFI)
//! Freestanding UEFI 2.10 PE32+ Application

#![no_std]
#![no_main]

mod config;
mod elf;
mod uefi;

use config::BootConfig;
use core::panic::PanicInfo;
use elf::Elf64Header;
use uefi::*;

pub const NOVAOS_BOOT_MAGIC: u64 = 0x4E4F56414F532131; // "NOVAOS!1"
pub const MAX_MEMORY_REGIONS: usize = 128;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub length: u64,
    pub region_type: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub regions_count: usize,
    pub regions: [MemoryRegion; MAX_MEMORY_REGIONS],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootVolumeInfo {
    pub partition_type: u32,
    pub total_sectors: u64,
    pub bytes_per_sector: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub magic: u64,
    pub memory_map: MemoryMap,
    pub framebuffer: Option<FramebufferInfo>,
    pub rsdp_address: Option<u64>,
    pub boot_volume: BootVolumeInfo,
    pub kernel_physical_base: u64,
    pub kernel_virtual_base: u64,
    pub command_line: Option<&'static str>,
}

static mut GLOBAL_BOOT_INFO: BootInfo = BootInfo {
    magic: NOVAOS_BOOT_MAGIC,
    memory_map: MemoryMap {
        regions_count: 0,
        regions: [MemoryRegion { start: 0, length: 0, region_type: 2 }; MAX_MEMORY_REGIONS],
    },
    framebuffer: None,
    rsdp_address: None,
    boot_volume: BootVolumeInfo {
        partition_type: 1,
        total_sectors: 0,
        bytes_per_sector: 512,
    },
    kernel_physical_base: 0x200000,
    kernel_virtual_base: 0xFFFF_8000_0020_0000,
    command_line: Some("console=serial,gop loglevel=debug"),
};

// Console output helpers
unsafe fn puts(st: *mut EfiSystemTable, s: &str) {
    if st.is_null() || (*st).ConOut.is_null() {
        return;
    }
    let mut buf = [0u16; 128];
    let mut i = 0;
    for b in s.bytes() {
        if b == b'\n' {
            buf[i] = b'\r' as u16;
            i += 1;
            if i >= buf.len() - 1 {
                buf[i] = 0;
                ((*(*st).ConOut).OutputString)((*st).ConOut, buf.as_ptr());
                i = 0;
            }
        }
        buf[i] = b as u16;
        i += 1;
        if i >= buf.len() - 1 {
            buf[i] = 0;
            ((*(*st).ConOut).OutputString)((*st).ConOut, buf.as_ptr());
            i = 0;
        }
    }
    if i > 0 {
        buf[i] = 0;
        ((*(*st).ConOut).OutputString)((*st).ConOut, buf.as_ptr());
    }
}

unsafe fn put_hex(st: *mut EfiSystemTable, mut val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [b'0'; 18];
    buf[0] = b'0';
    buf[1] = b'x';
    for i in (0..16).rev() {
        buf[2 + i] = hex_chars[(val & 0xF) as usize];
        val >>= 4;
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        puts(st, s);
    }
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: EfiHandle, system_table: *mut EfiSystemTable) -> EfiStatus {
    unsafe {
        let bs = (*system_table).BootServices;

        // 1. Diagnostics Banner
        puts(system_table, "==========================================\n");
        puts(system_table, "       NovaOS Zero 64-Bit UEFI Loader     \n");
        puts(system_table, "==========================================\n");
        puts(system_table, "[BOOT] Firmware initialized successfully\n");

        // 2. Locate Root Filesystem
        let mut loaded_image: *mut EfiLoadedImageProtocol = core::ptr::null_mut();
        let status = ((*bs).HandleProtocol)(
            image_handle,
            &EFI_LOADED_IMAGE_PROTOCOL_GUID,
            &mut loaded_image as *mut _ as *mut *mut core::ffi::c_void,
        );

        let mut config = BootConfig::default();
        if status == EFI_SUCCESS && !loaded_image.is_null() {
            let mut fs_proto: *mut EfiSimpleFileSystemProtocol = core::ptr::null_mut();
            let fs_status = ((*bs).HandleProtocol)(
                (*loaded_image).DeviceHandle,
                &EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
                &mut fs_proto as *mut _ as *mut *mut core::ffi::c_void,
            );

            if fs_status == EFI_SUCCESS && !fs_proto.is_null() {
                let mut root_dir: *mut EfiFileProtocol = core::ptr::null_mut();
                if ((*fs_proto).OpenVolume)(fs_proto, &mut root_dir) == EFI_SUCCESS {
                    // Try to read novaos.cfg
                    let mut cfg_file: *mut EfiFileProtocol = core::ptr::null_mut();
                    let cfg_name: [u16; 11] = [
                        b'n' as u16, b'o' as u16, b'v' as u16, b'a' as u16, b'o' as u16,
                        b's' as u16, b'.' as u16, b'c' as u16, b'f' as u16, b'g' as u16, 0,
                    ];
                    if ((*root_dir).Open)(root_dir, &mut cfg_file, cfg_name.as_ptr(), EFI_FILE_MODE_READ, 0) == EFI_SUCCESS {
                        let mut buf = [0u8; 1024];
                        let mut size = 1023;
                        if ((*cfg_file).Read)(cfg_file, &mut size, buf.as_mut_ptr() as *mut _) == EFI_SUCCESS {
                            if let Ok(s) = core::str::from_utf8(&buf[..size]) {
                                config = BootConfig::parse(s);
                                puts(system_table, "[CONFIG] Loaded boot configuration from /novaos.cfg\n");
                            }
                        }
                        ((*cfg_file).Close)(cfg_file);
                    }

                    // Try to read and validate /novaos.elf
                    let mut kernel_file: *mut EfiFileProtocol = core::ptr::null_mut();
                    let kernel_name: [u16; 11] = [
                        b'n' as u16, b'o' as u16, b'v' as u16, b'a' as u16, b'o' as u16,
                        b's' as u16, b'.' as u16, b'e' as u16, b'l' as u16, b'f' as u16, 0,
                    ];
                    if ((*root_dir).Open)(root_dir, &mut kernel_file, kernel_name.as_ptr(), EFI_FILE_MODE_READ, 0) == EFI_SUCCESS {
                        // Allocate buffer for kernel image (up to 4 MiB)
                        let mut kernel_buf_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
                        let max_kernel_size = 4 * 1024 * 1024;
                        if ((*bs).AllocatePool)(EfiMemoryType::LoaderData, max_kernel_size, &mut kernel_buf_ptr) == EFI_SUCCESS {
                            let mut read_size = max_kernel_size;
                            if ((*kernel_file).Read)(kernel_file, &mut read_size, kernel_buf_ptr) == EFI_SUCCESS && read_size > 0 {
                                let raw_slice = core::slice::from_raw_parts(kernel_buf_ptr as *const u8, read_size);
                                match Elf64Header::parse(raw_slice) {
                                    Ok(hdr) => {
                                        puts(system_table, "[ELF] Validated kernel ELF64 image (x86_64, little-endian)\n");
                                        let _ = hdr.load_segments(raw_slice);
                                    }
                                    Err(_) => {
                                        puts(system_table, "[ELF] Warning: Failed to validate kernel ELF64 header\n");
                                    }
                                }
                            }
                            let _ = ((*bs).FreePool)(kernel_buf_ptr);
                        }
                        ((*kernel_file).Close)(kernel_file);
                    }

                    ((*root_dir).Close)(root_dir);
                }
            }
        }

        puts(system_table, "[CONFIG] Command line: ");
        puts(system_table, config.command_line);
        puts(system_table, "\n");
        GLOBAL_BOOT_INFO.command_line = Some(config.command_line);

        // 3. Query Graphics Output Protocol (GOP)
        let mut gop: *mut EfiGraphicsOutputProtocol = core::ptr::null_mut();
        let gop_status = ((*bs).LocateProtocol)(
            &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
            core::ptr::null_mut(),
            &mut gop as *mut _ as *mut *mut core::ffi::c_void,
        );

        let mut fb_info: Option<FramebufferInfo> = None;
        if gop_status == EFI_SUCCESS && !gop.is_null() && !(*gop).Mode.is_null() {
            let mode = (*gop).Mode;
            let info = (*mode).Info;
            let width = (*info).HorizontalResolution;
            let height = (*info).VerticalResolution;
            let stride = (*info).PixelsPerScanLine;
            let base = (*mode).FrameBufferBase;
            let size = (*mode).FrameBufferSize;

            puts(system_table, "[GOP] Linear framebuffer at ");
            put_hex(system_table, base);
            puts(system_table, "\n");

            fb_info = Some(FramebufferInfo {
                address: base,
                size,
                width,
                height,
                stride,
                pixel_format: (*info).PixelFormat as u32,
            });
        }
        GLOBAL_BOOT_INFO.framebuffer = fb_info;

        // 4. Locate ACPI RSDP Table
        let mut rsdp_addr: Option<u64> = None;
        for i in 0..(*system_table).NumberOfTableEntries {
            let entry = &*(*system_table).ConfigurationTable.add(i);
            if entry.VendorGuid == ACPI_20_TABLE_GUID || entry.VendorGuid == ACPI_10_TABLE_GUID {
                rsdp_addr = Some(entry.VendorTable as u64);
                puts(system_table, "[ACPI] Found RSDP table at ");
                put_hex(system_table, entry.VendorTable as u64);
                puts(system_table, "\n");
                break;
            }
        }
        GLOBAL_BOOT_INFO.rsdp_address = rsdp_addr;

        // 5. Gather Physical Memory Map
        let mut mmap_buf = [0u8; 16384]; // 16 KiB memory map buffer
        let mut mmap_size = mmap_buf.len();
        let mut map_key: usize = 0;
        let mut desc_size: usize = 0;
        let mut desc_version: u32 = 0;

        let mmap_status = ((*bs).GetMemoryMap)(
            &mut mmap_size,
            mmap_buf.as_mut_ptr() as *mut EfiMemoryDescriptor,
            &mut map_key,
            &mut desc_size,
            &mut desc_version,
        );

        if mmap_status == EFI_SUCCESS && desc_size > 0 {
            let num_entries = mmap_size / desc_size;
            let mut count = 0;
            for i in 0..num_entries {
                if count >= MAX_MEMORY_REGIONS {
                    break;
                }
                let desc = &*(mmap_buf.as_ptr().add(i * desc_size) as *const EfiMemoryDescriptor);
                let reg_type = match desc.Type {
                    7 => 1, // Conventional RAM
                    9 => 3, // ACPI Reclaim
                    11 | 12 => 4, // MMIO
                    _ => 2, // Reserved
                };

                GLOBAL_BOOT_INFO.memory_map.regions[count] = MemoryRegion {
                    start: desc.PhysicalStart,
                    length: desc.NumberOfPages * 4096,
                    region_type: reg_type,
                };
                count += 1;
            }
            GLOBAL_BOOT_INFO.memory_map.regions_count = count;
            puts(system_table, "[MMAP] Parsed physical memory descriptors\n");
        }

        // 6. Disable Interrupts and Exit Boot Services
        puts(system_table, "[BOOT] Calling ExitBootServices...\n");

        // Clear interrupt flag before exiting boot services
        core::arch::asm!("cli", options(nomem, nostack));

        let exit_status = ((*bs).ExitBootServices)(image_handle, map_key);
        if exit_status != EFI_SUCCESS {
            // Memory map changed, retry once with fresh key
            mmap_size = mmap_buf.len();
            if ((*bs).GetMemoryMap)(
                &mut mmap_size,
                mmap_buf.as_mut_ptr() as *mut EfiMemoryDescriptor,
                &mut map_key,
                &mut desc_size,
                &mut desc_version,
            ) == EFI_SUCCESS {
                let _ = ((*bs).ExitBootServices)(image_handle, map_key);
            }
        }

        // 7. Execution Handover to Kernel
        // The kernel is mapped at physical 2 MiB (0x200000).
        // Transition stack and jump to kernel_entry.
        let kernel_entry_fn: extern "C" fn(*const BootInfo) -> ! =
            core::mem::transmute(0x200000 as usize);

        kernel_entry_fn(&raw const GLOBAL_BOOT_INFO);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
