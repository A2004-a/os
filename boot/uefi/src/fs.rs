//! NovaOS Apex — UEFI Simple File System Helper

use crate::uefi::*;

pub struct UefiVolume {
    root: *mut FileProtocol,
}

impl UefiVolume {
    pub fn open_boot_volume(image_handle: usize, bs: &BootServices) -> Option<Self> {
        let mut loaded_image_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let status = unsafe {
            (bs.handle_protocol)(
                image_handle,
                &LOADED_IMAGE_PROTOCOL_GUID,
                &mut loaded_image_ptr,
            )
        };

        if status != 0 || loaded_image_ptr.is_null() {
            return None;
        }

        let loaded_image = unsafe { &*(loaded_image_ptr as *mut LoadedImageProtocol) };
        let device_handle = loaded_image.device_handle;

        let mut sfs_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let status2 = unsafe {
            (bs.handle_protocol)(
                device_handle,
                &SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
                &mut sfs_ptr,
            )
        };

        if status2 != 0 || sfs_ptr.is_null() {
            return None;
        }

        let sfs = unsafe { &*(sfs_ptr as *mut SimpleFileSystemProtocol) };
        let mut root: *mut FileProtocol = core::ptr::null_mut();
        let status3 = unsafe {
            (sfs.open_volume)(sfs_ptr as *mut SimpleFileSystemProtocol, &mut root)
        };

        if status3 != 0 || root.is_null() {
            return None;
        }

        Some(Self { root })
    }

    pub fn read_file(&self, path_utf8: &str, bs: &BootServices) -> Option<(&'static [u8], usize)> {
        // Convert UTF-8 path to UTF-16 null-terminated array
        let mut utf16_path = [0u16; 128];
        let mut idx = 0;
        for c in path_utf8.chars() {
            if idx >= 127 {
                break;
            }
            // Normalize '/' to '\' for UEFI paths
            if c == '/' {
                utf16_path[idx] = '\\' as u16;
            } else {
                utf16_path[idx] = c as u16;
            }
            idx += 1;
        }
        utf16_path[idx] = 0;

        let root = unsafe { &*self.root };
        let mut file_ptr: *mut FileProtocol = core::ptr::null_mut();

        let status = unsafe {
            (root.open)(
                self.root,
                &mut file_ptr,
                utf16_path.as_ptr(),
                EFI_FILE_MODE_READ,
                0,
            )
        };

        if status != 0 || file_ptr.is_null() {
            return None;
        }

        let file = unsafe { &*file_ptr };

        // Allocate a buffer up to 32MB for reading
        const MAX_FILE_SIZE: usize = 32 * 1024 * 1024;
        let pages = (MAX_FILE_SIZE + 4095) / 4096;
        let mut alloc_addr: u64 = 0;
        let alloc_status = unsafe {
            (bs.allocate_pages)(0, 2, pages, &mut alloc_addr) // 0 = AllocateAnyPages, 2 = EfiLoaderData
        };

        if alloc_status != 0 || alloc_addr == 0 {
            unsafe { (file.close)(file_ptr); }
            return None;
        }

        let mut read_size = MAX_FILE_SIZE;
        let read_status = unsafe {
            (file.read)(file_ptr, &mut read_size, alloc_addr as *mut u8)
        };

        unsafe {
            (file.close)(file_ptr);
        }

        if read_status != 0 {
            unsafe { (bs.free_pages)(alloc_addr, pages); }
            return None;
        }

        let slice = unsafe {
            core::slice::from_raw_parts(alloc_addr as *const u8, read_size)
        };

        Some((slice, pages))
    }
}
