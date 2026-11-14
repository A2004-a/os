//! NovaOS Apex — Stable Boot Information Structure (UEFI -> Kernel Handshake)
//! Shared ABI definition between bootloader and kernel.

pub const BOOT_INFO_MAGIC: u64 = 0x4E4F5641_41504558; // "NOVAAPEX"
pub const BOOT_INFO_VERSION: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootInfo {
    pub magic: u64,
    pub version: u32,
    pub flags: u32,

    // Framebuffer information (UEFI GOP)
    pub framebuffer_base: u64,
    pub framebuffer_size: u64,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixels_per_scan_line: u32,
    pub pixel_format: u32, // 0 = PixelRedGreenBlueReserved8BitPerColor, 1 = PixelBlueGreenRedReserved8BitPerColor

    // ACPI RSDP physical address
    pub rsdp_address: u64,

    // UEFI Memory Map descriptor array
    pub memory_map_addr: u64,
    pub memory_map_size: u64,
    pub descriptor_size: u64,
    pub descriptor_version: u32,
    pub memory_map_entries: u32,

    // Kernel Image details
    pub kernel_virt_base: u64,
    pub kernel_phys_base: u64,
    pub kernel_size: u64,
    pub kernel_entry: u64,

    // Boot command line string
    pub cmdline_addr: u64,
    pub cmdline_len: u64,
}

impl BootInfo {
    pub fn is_valid(&self) -> bool {
        self.magic == BOOT_INFO_MAGIC && self.version == BOOT_INFO_VERSION
    }
}
