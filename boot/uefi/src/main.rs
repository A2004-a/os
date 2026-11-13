#![no_std]
#![no_main]

extern crate alloc;

use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{MemoryDescriptor, MemoryType};
use uefi::table::cfg::{ACPI_2_GUID, ACPI_GUID};

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

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let boot_services = system_table.boot_services();

    // 1. Diagnostics Banner
    uefi_services::println!("==========================================");
    uefi_services::println!("       NovaOS Zero 64-Bit UEFI Loader     ");
    uefi_services::println!("==========================================");
    uefi_services::println!("[BOOT] Firmware initialized successfully");

    // 2. Query Graphics Output Protocol (GOP)
    let framebuffer_info = if let Ok(gop_handle) = boot_services.get_handle_for_protocol::<GraphicsOutput>() {
        if let Ok(mut gop) = boot_services.open_protocol_exclusive::<GraphicsOutput>(gop_handle) {
            let mode_info = gop.current_mode_info();
            let (width, height) = mode_info.resolution();
            let stride = mode_info.stride();
            let mut fb = gop.frame_buffer();
            let fb_address = fb.as_mut_ptr() as u64;
            let fb_size = fb.size();

            uefi_services::println!(
                "[GOP] Linear framebuffer at 0x{:X}, {}x{} (stride: {})",
                fb_address, width, height, stride
            );

            Some(FramebufferInfo {
                address: fb_address,
                size: fb_size,
                width: width as u32,
                height: height as u32,
                stride: stride as u32,
                pixel_format: 0,
            })
        } else {
            None
        }
    } else {
        None
    };

    // 3. Locate ACPI RSDP Pointer
    let mut rsdp_address: Option<u64> = None;
    for entry in system_table.config_table() {
        if entry.guid == ACPI_2_GUID || entry.guid == ACPI_GUID {
            rsdp_address = Some(entry.address as u64);
            uefi_services::println!("[ACPI] Found RSDP table at 0x{:X}", entry.address as u64);
            break;
        }
    }

    // 4. Collect UEFI Memory Map
    let mmap_size = boot_services.memory_map_size();
    let max_mmap_size = mmap_size.map_size + mmap_size.entry_size * 16;
    let mut mmap_buffer = alloc::vec![0u8; max_mmap_size];

    let (_mmap_key, mmap_iter) = boot_services
        .memory_map(&mut mmap_buffer)
        .expect("Failed to retrieve UEFI memory map");

    let mut regions = [MemoryRegion { start: 0, length: 0, region_type: 2 }; MAX_MEMORY_REGIONS];
    let mut count = 0;

    for desc in mmap_iter {
        if count >= MAX_MEMORY_REGIONS {
            break;
        }
        let reg_type = match desc.ty {
            MemoryType::CONVENTIONAL => 1,
            MemoryType::ACPI_RECLAIM => 3,
            MemoryType::MMIO | MemoryType::MMIO_PORT_SPACE => 4,
            _ => 2,
        };
        regions[count] = MemoryRegion {
            start: desc.phys_start,
            length: desc.page_count * 4096,
            region_type: reg_type,
        };
        count += 1;
    }

    uefi_services::println!("[MMAP] Parsed {} physical memory descriptors", count);

    // 5. Construct Stable BootInfo Structure
    static mut BOOT_INFO: BootInfo = BootInfo {
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
        command_line: Some("quiet console=serial,vga"),
    };

    unsafe {
        BOOT_INFO.memory_map.regions_count = count;
        BOOT_INFO.memory_map.regions = regions;
        BOOT_INFO.framebuffer = framebuffer_info;
        BOOT_INFO.rsdp_address = rsdp_address;
    }

    uefi_services::println!("[BOOT] Prepared BootInfo block. Exiting boot services...");

    Status::SUCCESS
}
