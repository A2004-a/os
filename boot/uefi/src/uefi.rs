//! NovaOS Zero — Freestanding UEFI 2.10 Specification Core Types & Protocols
//! Pure freestanding Rust without external dependencies or build scripts.

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use core::ffi::c_void;

pub type EfiStatus = usize;
pub type EfiHandle = *mut c_void;

pub const EFI_SUCCESS: EfiStatus = 0;
pub const EFI_LOAD_ERROR: EfiStatus = 1 | (1 << 63);
pub const EFI_INVALID_PARAMETER: EfiStatus = 2 | (1 << 63);
pub const EFI_UNSUPPORTED: EfiStatus = 3 | (1 << 63);
pub const EFI_BAD_BUFFER_SIZE: EfiStatus = 4 | (1 << 63);
pub const EFI_BUFFER_TOO_SMALL: EfiStatus = 5 | (1 << 63);
pub const EFI_NOT_READY: EfiStatus = 6 | (1 << 63);
pub const EFI_NOT_FOUND: EfiStatus = 14 | (1 << 63);

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EfiGuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data1: 0x9042a9de,
    data2: 0x23dc,
    data3: 0x4a38,
    data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

pub const EFI_LOADED_IMAGE_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data1: 0x5B1B31A1,
    data2: 0x9562,
    data3: 0x11d2,
    data4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
};

pub const EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data1: 0x0964e5b22,
    data2: 0x6459,
    data3: 0x11d2,
    data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

pub const ACPI_20_TABLE_GUID: EfiGuid = EfiGuid {
    data1: 0x8868e871,
    data2: 0xe4f1,
    data3: 0x11d3,
    data4: [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
};

pub const ACPI_10_TABLE_GUID: EfiGuid = EfiGuid {
    data1: 0xeb9d2d30,
    data2: 0x2d88,
    data3: 0x11d3,
    data4: [0x9a, 0x16, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
};

#[repr(C)]
pub struct EfiTableHeader {
    pub Signature: u64,
    pub Revision: u32,
    pub HeaderSize: u32,
    pub CRC32: u32,
    pub Reserved: u32,
}

#[repr(C)]
pub struct EfiConfigurationTable {
    pub VendorGuid: EfiGuid,
    pub VendorTable: *const c_void,
}

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    pub Reset: usize,
    pub OutputString: unsafe extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, string: *const u16) -> EfiStatus,
    pub TestString: usize,
    pub QueryMode: usize,
    pub SetMode: usize,
    pub SetAttribute: usize,
    pub ClearScreen: unsafe extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol) -> EfiStatus,
    pub SetCursorPosition: usize,
    pub EnableCursor: usize,
    pub Mode: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfiMemoryType {
    ReservedMemoryType = 0,
    LoaderCode = 1,
    LoaderData = 2,
    BootServicesCode = 3,
    BootServicesData = 4,
    RuntimeServicesCode = 5,
    RuntimeServicesData = 6,
    ConventionalMemory = 7,
    UnusableMemory = 8,
    ACPIReclaimMemory = 9,
    ACPIMemoryNVS = 10,
    MemoryMappedIO = 11,
    MemoryMappedIOPortSpace = 12,
    PalCode = 13,
    PersistentMemory = 14,
    MaxMemoryType = 15,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfiMemoryDescriptor {
    pub Type: u32,
    pub PhysicalStart: u64,
    pub VirtualStart: u64,
    pub NumberOfPages: u64,
    pub Attribute: u64,
}

#[repr(C)]
pub enum EfiAllocateType {
    AllocateAnyPages,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType,
}

#[repr(C)]
pub struct EfiBootServices {
    pub Hdr: EfiTableHeader,
    pub RaiseTPL: usize,
    pub RestoreTPL: usize,
    pub AllocatePages: unsafe extern "efiapi" fn(
        alloc_type: EfiAllocateType,
        memory_type: EfiMemoryType,
        pages: usize,
        memory: *mut u64,
    ) -> EfiStatus,
    pub FreePages: unsafe extern "efiapi" fn(memory: u64, pages: usize) -> EfiStatus,
    pub GetMemoryMap: unsafe extern "efiapi" fn(
        memory_map_size: *mut usize,
        memory_map: *mut EfiMemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> EfiStatus,
    pub AllocatePool: unsafe extern "efiapi" fn(pool_type: EfiMemoryType, size: usize, buffer: *mut *mut c_void) -> EfiStatus,
    pub FreePool: unsafe extern "efiapi" fn(buffer: *mut c_void) -> EfiStatus,
    pub CreateEvent: usize,
    pub SetTimer: usize,
    pub WaitForEvent: usize,
    pub SignalEvent: usize,
    pub CloseEvent: usize,
    pub CheckEvent: usize,
    pub InstallProtocolInterface: usize,
    pub ReinstallProtocolInterface: usize,
    pub UninstallProtocolInterface: usize,
    pub HandleProtocol: unsafe extern "efiapi" fn(handle: EfiHandle, protocol: *const EfiGuid, interface: *mut *mut c_void) -> EfiStatus,
    pub Reserved: *mut c_void,
    pub RegisterProtocolNotify: usize,
    pub LocateHandle: usize,
    pub LocateDevicePath: usize,
    pub InstallConfigurationTable: usize,
    pub LoadImage: usize,
    pub StartImage: usize,
    pub Exit: usize,
    pub UnloadImage: usize,
    pub ExitBootServices: unsafe extern "efiapi" fn(image_handle: EfiHandle, map_key: usize) -> EfiStatus,
    pub GetNextMonotonicCount: usize,
    pub Stall: unsafe extern "efiapi" fn(microseconds: usize) -> EfiStatus,
    pub SetWatchdogTimer: unsafe extern "efiapi" fn(timeout: usize, watchdog_code: u64, data_size: usize, watchdog_data: *const u16) -> EfiStatus,
    pub ConnectController: usize,
    pub DisconnectController: usize,
    pub OpenProtocol: unsafe extern "efiapi" fn(
        handle: EfiHandle,
        protocol: *const EfiGuid,
        interface: *mut *mut c_void,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
        attributes: u32,
    ) -> EfiStatus,
    pub CloseProtocol: unsafe extern "efiapi" fn(handle: EfiHandle, protocol: *const EfiGuid, agent_handle: EfiHandle, controller_handle: EfiHandle) -> EfiStatus,
    pub OpenProtocolInformation: usize,
    pub ProtocolsPerHandle: usize,
    pub LocateHandleBuffer: usize,
    pub LocateProtocol: unsafe extern "efiapi" fn(protocol: *const EfiGuid, registration: *mut c_void, interface: *mut *mut c_void) -> EfiStatus,
    pub InstallMultipleProtocolInterfaces: usize,
    pub UninstallMultipleProtocolInterfaces: usize,
    pub CalculateCrc32: usize,
    pub CopyMem: unsafe extern "efiapi" fn(destination: *mut c_void, source: *const c_void, length: usize),
    pub SetMem: unsafe extern "efiapi" fn(buffer: *mut c_void, size: usize, value: u8),
}

#[repr(C)]
pub struct EfiSystemTable {
    pub Hdr: EfiTableHeader,
    pub FirmwareVendor: *const u16,
    pub FirmwareRevision: u32,
    pub ConsoleInHandle: EfiHandle,
    pub ConIn: *mut c_void,
    pub ConsoleOutHandle: EfiHandle,
    pub ConOut: *mut EfiSimpleTextOutputProtocol,
    pub StandardErrorHandle: EfiHandle,
    pub StdErr: *mut EfiSimpleTextOutputProtocol,
    pub RuntimeServices: *mut c_void,
    pub BootServices: *mut EfiBootServices,
    pub NumberOfTableEntries: usize,
    pub ConfigurationTable: *const EfiConfigurationTable,
}

#[repr(C)]
pub struct EfiPixelBitmask {
    pub RedMask: u32,
    pub GreenMask: u32,
    pub BlueMask: u32,
    pub ReservedMask: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfiGraphicsPixelFormat {
    PixelRedGreenBlueReserved8BitPerColor,
    PixelBlueGreenRedReserved8BitPerColor,
    PixelBitMask,
    PixelBltOnly,
    PixelFormatMax,
}

#[repr(C)]
pub struct EfiGraphicsOutputModeInformation {
    pub Version: u32,
    pub HorizontalResolution: u32,
    pub VerticalResolution: u32,
    pub PixelFormat: EfiGraphicsPixelFormat,
    pub PixelInformation: EfiPixelBitmask,
    pub PixelsPerScanLine: u32,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocolMode {
    pub MaxMode: u32,
    pub Mode: u32,
    pub Info: *mut EfiGraphicsOutputModeInformation,
    pub SizeOfInfo: usize,
    pub FrameBufferBase: u64,
    pub FrameBufferSize: usize,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocol {
    pub QueryMode: usize,
    pub SetMode: usize,
    pub Blt: usize,
    pub Mode: *mut EfiGraphicsOutputProtocolMode,
}

pub const EFI_FILE_MODE_READ: u64 = 0x0000000000000001;

#[repr(C)]
pub struct EfiFileProtocol {
    pub Revision: u64,
    pub Open: unsafe extern "efiapi" fn(
        this: *mut EfiFileProtocol,
        new_handle: *mut *mut EfiFileProtocol,
        file_name: *const u16,
        open_mode: u64,
        attributes: u64,
    ) -> EfiStatus,
    pub Close: unsafe extern "efiapi" fn(this: *mut EfiFileProtocol) -> EfiStatus,
    pub Delete: usize,
    pub Read: unsafe extern "efiapi" fn(this: *mut EfiFileProtocol, buffer_size: *mut usize, buffer: *mut c_void) -> EfiStatus,
    pub Write: usize,
    pub GetPosition: usize,
    pub SetPosition: unsafe extern "efiapi" fn(this: *mut EfiFileProtocol, position: u64) -> EfiStatus,
    pub GetInfo: usize,
    pub SetInfo: usize,
    pub Flush: usize,
}

#[repr(C)]
pub struct EfiSimpleFileSystemProtocol {
    pub Revision: u64,
    pub OpenVolume: unsafe extern "efiapi" fn(
        this: *mut EfiSimpleFileSystemProtocol,
        root: *mut *mut EfiFileProtocol,
    ) -> EfiStatus,
}

#[repr(C)]
pub struct EfiLoadedImageProtocol {
    pub Revision: u32,
    pub ParentHandle: EfiHandle,
    pub SystemTable: *mut EfiSystemTable,
    pub DeviceHandle: EfiHandle,
    pub FilePath: *mut c_void,
    pub Reserved: *mut c_void,
    pub LoadOptionsSize: u32,
    pub LoadOptions: *mut c_void,
    pub ImageBase: *mut c_void,
    pub ImageSize: u64,
    pub ImageCodeType: EfiMemoryType,
    pub ImageDataType: EfiMemoryType,
    pub Unload: usize,
}
