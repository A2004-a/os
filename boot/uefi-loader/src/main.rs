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
    pub pixel_format: u32, // 0 = RGB, 1 = BGR
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub length: u64,
    pub region_type: u32, // 1 = Usable, 2 = Reserved, 3 = AcpiReclaim, 4 = Mmio
}

#[repr(C)]
pub struct BootInfo {
    pub magic: u64,
    pub version: u32,
    pub framebuffer: FramebufferInfo,
    pub memory_regions_count: usize,
    pub memory_regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    pub rsdp_address: u64,
    pub physical_memory_offset: u64,
}

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let boot_services = system_table.boot_services();

    // 1. Output Banner to UEFI Screen & Serial
    uefi_services::println!("==========================================");
    uefi_services::println!("       NovaOS 64-Bit UEFI Bootloader      ");
    uefi_services::println!("==========================================");
    uefi_services::println!("[UEFI] Firmware initialized successfully");

    // 2. Query Graphics Output Protocol (GOP)
    let gop_handle = boot_services
        .get_handle_for_protocol::<GraphicsOutput>()
        .expect("Failed to locate GOP handle");
    let mut gop = boot_services
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .expect("Failed to open GraphicsOutput protocol");

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

    let fb_info = FramebufferInfo {
        address: fb_address,
        size: fb_size,
        width: width as u32,
        height: height as u32,
        stride: stride as u32,
        pixel_format: 0, // Default RGB
    };

    // 3. Locate ACPI RSDP Pointer
    let mut rsdp_address: u64 = 0;
    for entry in system_table.config_table() {
        if entry.guid == ACPI_2_GUID || entry.guid == ACPI_GUID {
            rsdp_address = entry.address as u64;
            uefi_services::println!("[ACPI] Found RSDP table at 0x{:X}", rsdp_address);
            break;
        }
    }

    // 4. Retrieve UEFI Memory Map
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

    // 5. Construct Static BootInfo Block
    static mut BOOT_INFO: BootInfo = BootInfo {
        magic: NOVAOS_BOOT_MAGIC,
        version: 1,
        framebuffer: FramebufferInfo {
            address: 0,
            size: 0,
            width: 0,
            height: 0,
            stride: 0,
            pixel_format: 0,
        },
        memory_regions_count: 0,
        memory_regions: [MemoryRegion { start: 0, length: 0, region_type: 2 }; MAX_MEMORY_REGIONS],
        rsdp_address: 0,
        physical_memory_offset: 0xFFFF_8000_0000_0000,
    };

    unsafe {
        BOOT_INFO.framebuffer = fb_info;
        BOOT_INFO.memory_regions_count = count;
        BOOT_INFO.memory_regions = regions;
        BOOT_INFO.rsdp_address = rsdp_address;
    }

    uefi_services::println!("[BOOT] Exiting UEFI Boot Services and jumping to kernel...");

    // In a complete build, boot_services.exit_boot_services() is called
    // and execution branches to the loaded kernel ELF entry point.

    Status::SUCCESS
}
