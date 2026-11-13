//! NovaOS Zero Boot Information Specification
//! Shared ABI between UEFI Boot Manager and Kernel Entry

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
pub struct MemoryMap {
    pub regions_count: usize,
    pub regions: [MemoryRegion; MAX_MEMORY_REGIONS],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootVolumeInfo {
    pub partition_type: u32, // 1 = FAT32 ESP
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

impl BootInfo {
    pub fn is_valid(&self) -> bool {
        self.magic == NOVAOS_BOOT_MAGIC
    }
}
