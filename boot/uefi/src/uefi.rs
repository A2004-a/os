//! NovaOS Apex — Freestanding UEFI 2.10 Protocol Declarations

#[repr(C)]
pub struct SystemTable {
    pub hdr: TableHeader,
    pub firmware_vendor: *const u16,
    pub firmware_revision: u32,
    pub console_in_handle: usize,
    pub con_in: usize,
    pub console_out_handle: usize,
    pub con_out: *mut SimpleTextOutput,
    pub standard_error_handle: usize,
    pub std_err: *mut SimpleTextOutput,
    pub runtime_services: usize,
    pub boot_services: *mut BootServices,
    pub number_of_table_entries: usize,
    pub configuration_table: *const ConfigurationTable,
}

#[repr(C)]
pub struct TableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct ConfigurationTable {
    pub vendor_guid: Guid,
    pub vendor_table: *const core::ffi::c_void,
}

#[repr(C)]
pub struct SimpleTextOutput {
    pub reset: usize,
    pub output_string: unsafe extern "efiapi" fn(*mut SimpleTextOutput, *const u16) -> usize,
    pub test_string: usize,
    pub query_mode: usize,
    pub set_mode: usize,
    pub set_attribute: usize,
    pub clear_screen: unsafe extern "efiapi" fn(*mut SimpleTextOutput) -> usize,
    pub set_cursor_position: usize,
    pub enable_cursor: usize,
    pub mode: usize,
}

#[repr(C)]
pub struct BootServices {
    pub hdr: TableHeader,
    pub raise_tpl: usize,
    pub restore_tpl: usize,
    pub allocate_pages: unsafe extern "efiapi" fn(
        alloc_type: u32,
        memory_type: u32,
        pages: usize,
        memory: *mut u64,
    ) -> usize,
    pub free_pages: unsafe extern "efiapi" fn(memory: u64, pages: usize) -> usize,
    pub get_memory_map: unsafe extern "efiapi" fn(
        memory_map_size: *mut usize,
        memory_map: *mut u8,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> usize,
    pub allocate_pool: unsafe extern "efiapi" fn(
        pool_type: u32,
        size: usize,
        buffer: *mut *mut u8,
    ) -> usize,
    pub free_pool: unsafe extern "efiapi" fn(buffer: *mut u8) -> usize,
    pub create_event: usize,
    pub set_timer: usize,
    pub wait_for_event: usize,
    pub signal_event: usize,
    pub close_event: usize,
    pub check_event: usize,
    pub install_protocol_interface: usize,
    pub reinstall_protocol_interface: usize,
    pub uninstall_protocol_interface: usize,
    pub handle_protocol: unsafe extern "efiapi" fn(
        handle: usize,
        protocol: *const Guid,
        interface: *mut *mut core::ffi::c_void,
    ) -> usize,
    pub reserved: usize,
    pub register_protocol_notify: usize,
    pub locate_handle: usize,
    pub locate_device_path: usize,
    pub install_configuration_table: usize,
    pub load_image: usize,
    pub start_image: usize,
    pub exit: usize,
    pub unload_image: usize,
    pub exit_boot_services: unsafe extern "efiapi" fn(image_handle: usize, map_key: usize) -> usize,
    pub get_next_monotonic_count: usize,
    pub stall: unsafe extern "efiapi" fn(microseconds: usize) -> usize,
    pub set_watchdog_timer: unsafe extern "efiapi" fn(timeout: usize, code: u64, len: usize, data: *const u16) -> usize,
    pub connect_controller: usize,
    pub disconnect_controller: usize,
    pub open_protocol: unsafe extern "efiapi" fn(
        handle: usize,
        protocol: *const Guid,
        interface: *mut *mut core::ffi::c_void,
        agent_handle: usize,
        controller_handle: usize,
        attributes: u32,
    ) -> usize,
    pub close_protocol: unsafe extern "efiapi" fn(
        handle: usize,
        protocol: *const Guid,
        agent_handle: usize,
        controller_handle: usize,
    ) -> usize,
    pub open_protocol_information: usize,
    pub protocols_per_handle: usize,
    pub locate_handle_buffer: usize,
    pub locate_protocol: unsafe extern "efiapi" fn(
        protocol: *const Guid,
        registration: *const core::ffi::c_void,
        interface: *mut *mut core::ffi::c_void,
    ) -> usize,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub const GOP_GUID: Guid = Guid {
    data1: 0x9042a9de,
    data2: 0x23dc,
    data3: 0x4a38,
    data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

pub const LOADED_IMAGE_PROTOCOL_GUID: Guid = Guid {
    data1: 0x5b1b31a1,
    data2: 0x9562,
    data3: 0x11d2,
    data4: [0x8e, 0x3f, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

pub const SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: Guid = Guid {
    data1: 0x964e5b22,
    data2: 0x6459,
    data3: 0x11d2,
    data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

pub const FILE_INFO_GUID: Guid = Guid {
    data1: 0x09576e92,
    data2: 0x6d3f,
    data3: 0x11d2,
    data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

pub const ACPI_20_TABLE_GUID: Guid = Guid {
    data1: 0x8868e871,
    data2: 0xe4f1,
    data3: 0x11d3,
    data4: [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
};

pub const ACPI_10_TABLE_GUID: Guid = Guid {
    data1: 0xeb9d2d30,
    data2: 0x2d88,
    data3: 0x11d3,
    data4: [0x9a, 0x16, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
};

#[repr(C)]
pub struct GraphicsOutputProtocol {
    pub query_mode: usize,
    pub set_mode: usize,
    pub blt: usize,
    pub mode: *mut GopMode,
}

#[repr(C)]
pub struct GopMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *mut GopModeInfo,
    pub size_of_info: usize,
    pub framebuffer_base: u64,
    pub framebuffer_size: usize,
}

#[repr(C)]
pub struct GopModeInfo {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub reserved_mask: u32,
    pub pixels_per_scan_line: u32,
}

#[repr(C)]
pub struct LoadedImageProtocol {
    pub revision: u32,
    pub parent_handle: usize,
    pub system_table: *mut SystemTable,
    pub device_handle: usize,
    pub file_path: usize,
    pub reserved: usize,
    pub load_options_size: u32,
    pub load_options: *mut core::ffi::c_void,
    pub image_base: *mut core::ffi::c_void,
    pub image_size: u64,
    pub image_code_type: u32,
    pub image_data_type: u32,
    pub unload: usize,
}

#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume: unsafe extern "efiapi" fn(
        this: *mut SimpleFileSystemProtocol,
        root: *mut *mut FileProtocol,
    ) -> usize,
}

pub const EFI_FILE_MODE_READ: u64 = 0x0000000000000001;

#[repr(C)]
pub struct FileProtocol {
    pub revision: u64,
    pub open: unsafe extern "efiapi" fn(
        this: *mut FileProtocol,
        new_handle: *mut *mut FileProtocol,
        file_name: *const u16,
        open_mode: u64,
        attributes: u64,
    ) -> usize,
    pub close: unsafe extern "efiapi" fn(this: *mut FileProtocol) -> usize,
    pub delete: usize,
    pub read: unsafe extern "efiapi" fn(
        this: *mut FileProtocol,
        buffer_size: *mut usize,
        buffer: *mut u8,
    ) -> usize,
    pub write: usize,
    pub get_position: usize,
    pub set_position: usize,
    pub get_info: unsafe extern "efiapi" fn(
        this: *mut FileProtocol,
        information_type: *const Guid,
        buffer_size: *mut usize,
        buffer: *mut core::ffi::c_void,
    ) -> usize,
    pub set_info: usize,
    pub flush: usize,
}

#[repr(C)]
pub struct FileInfo {
    pub size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: [u8; 16],
    pub last_access_time: [u8; 16],
    pub modification_time: [u8; 16],
    pub attribute: u64,
    pub file_name: [u16; 32],
}
