//! NovaOS Boot Information ABI
//! Structured data passed from UEFI Bootloader to Kernel Entry

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
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub magic: u64,
    pub version: u32,
    pub framebuffer: FramebufferInfo,
    pub memory_regions_count: usize,
    pub memory_regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    pub rsdp_address: u64,
    pub physical_memory_offset: u64,
}

impl BootInfo {
    pub fn is_valid(&self) -> bool {
        self.magic == NOVAOS_BOOT_MAGIC
    }
}
