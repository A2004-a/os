//! NovaOS Apex — 64-bit UEFI Boot Manager (BOOTX64.EFI)
//! Milestone 0: Toolchain, Boot Proof, UART Serial Logging & Early Hardware Discovery

#![no_std]
#![no_main]

pub mod serial;
pub mod uefi;

use core::panic::PanicInfo;
use uefi::{GraphicsOutputProtocol, SystemTable, GOP_GUID};

#[no_mangle]
pub extern "efiapi" fn efi_main(_image_handle: usize, system_table: *mut SystemTable) -> usize {
    // 1. Initialize UART 16550 serial port for immediate debug logging
    serial::init();
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — 64-bit UEFI Boot Manager (M0)         \n");
    serial::write_str("=======================================================\n");
    serial::write_str("[BOOT] Firmware handoff initiated from UEFI 2.10\n");

    let st = unsafe {
        if system_table.is_null() {
            serial::write_str("[FATAL] SystemTable pointer is null\n");
            loop { core::hint::spin_loop(); }
        }
        &*system_table
    };

    // 2. Output status string to UEFI Graphical Console
    if !st.con_out.is_null() {
        let msg = [
            'N' as u16, 'o' as u16, 'v' as u16, 'a' as u16, 'O' as u16, 'S' as u16, ' ' as u16,
            'A' as u16, 'p' as u16, 'e' as u16, 'x' as u16, ' ' as u16,
            'U' as u16, 'E' as u16, 'F' as u16, 'I' as u16, ' ' as u16,
            'B' as u16, 'o' as u16, 'o' as u16, 't' as u16, ' ' as u16,
            'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16,
            '\r' as u16, '\n' as u16, 0
        ];
        unsafe {
            let con_out = &*st.con_out;
            (con_out.output_string)(st.con_out, msg.as_ptr());
        }
    }

    // 3. Discover GOP (Graphics Output Protocol)
    let bs = unsafe {
        if st.boot_services.is_null() {
            serial::write_str("[FATAL] BootServices pointer is null\n");
            loop { core::hint::spin_loop(); }
        }
        &*st.boot_services
    };

    let mut gop_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
    let status = unsafe {
        (bs.locate_protocol)(&GOP_GUID, core::ptr::null(), &mut gop_ptr)
    };

    if status == 0 && !gop_ptr.is_null() {
        let gop = unsafe { &*(gop_ptr as *mut GraphicsOutputProtocol) };
        if !gop.mode.is_null() {
            let mode = unsafe { &*gop.mode };
            serial::write_str("[GOP] Framebuffer discovered at physical address: 0x");
            print_hex_u64(mode.framebuffer_base);
            serial::write_str(", Size: ");
            print_u64(mode.framebuffer_size as u64);
            serial::write_str(" bytes\n");

            if !mode.info.is_null() {
                let info = unsafe { &*mode.info };
                serial::write_str("[GOP] Resolution: ");
                print_u64(info.horizontal_resolution as u64);
                serial::write_str("x");
                print_u64(info.vertical_resolution as u64);
                serial::write_str(", PixelsPerScanLine: ");
                print_u64(info.pixels_per_scan_line as u64);
                serial::write_str("\n");
            }
        }
    } else {
        serial::write_str("[WARN] GOP locate_protocol returned non-zero status\n");
    }

    serial::write_str("[M0 SUCCESS] Boot proof confirmed! Reached NovaOS Apex boot manager.\n");
    serial::write_str("[M0] Ready for Milestone 1 Kernel Payload & Page Table Entry.\n");

    0 // EFI_SUCCESS
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
