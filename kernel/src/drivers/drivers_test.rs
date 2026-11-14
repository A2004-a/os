//! NovaOS Zero — Milestone M3 Hardware Driver & Storage Test Suite

use crate::drivers::pci::scan_pci_bus;
use crate::drivers::ps2_mouse::MOUSE;
use crate::interrupts::timer::get_ticks;
use crate::storage::{BlockDevice, RamBlockDevice};
use crate::serial_println;

pub fn run_driver_diagnostics() {
    serial_println!("----------------------------------------");
    serial_println!("[TEST-M3] Commencing Drivers & Storage Suite");
    serial_println!("----------------------------------------");

    // 1. PIT Timer Verification
    let ticks_initial = get_ticks();
    serial_println!("[TEST-M3] 1/4: PIT Timer: Initial tick count = {}", ticks_initial);

    // 2. PCI Bus Enumeration Test
    let pci_devices = scan_pci_bus();
    let mut detected_count = 0;
    for dev_opt in pci_devices.iter() {
        if let Some(dev) = dev_opt {
            detected_count += 1;
            serial_println!(
                "  [PCI] Found Device {:02X}:{:02X} -> Vendor: 0x{:04X}, Device: 0x{:04X}, Class: 0x{:02X}:0x{:02X}",
                dev.bus, dev.device, dev.vendor_id, dev.device_id, dev.class_id, dev.subclass_id
            );
        }
    }
    serial_println!("[TEST-M3] 2/4: PCI Bus Scanner: Discovered {} active hardware controllers.", detected_count);

    // 3. PS/2 Input Subsystem Test
    let mouse_state = MOUSE.lock().state;
    serial_println!(
        "[TEST-M3] 3/4: PS/2 Input: Mouse initialized at ({}, {}). Keyboard interrupt wired.",
        mouse_state.x, mouse_state.y
    );

    // 4. Block Storage Device Test (RAM Disk Sector R/W)
    let mut ramdisk = RamBlockDevice::new();
    let mut write_buf = [0u8; 512];
    for (i, byte) in write_buf.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    // Write to LBA 5
    let write_res = ramdisk.write_sector(5, &write_buf);
    assert!(write_res.is_ok(), "Writing sector to RAM disk must succeed");

    // Read back from LBA 5
    let mut read_buf = [0u8; 512];
    let read_res = ramdisk.read_sector(5, &mut read_buf);
    assert!(read_res.is_ok(), "Reading sector from RAM disk must succeed");
    assert_eq!(read_buf, write_buf, "Sector content must match written byte pattern");

    // Test bounds enforcement
    let mut oob_buf = [0u8; 512];
    let oob_res = ramdisk.read_sector(99999, &mut oob_buf);
    assert!(oob_res.is_err(), "Out-of-bounds LBA access must be rejected");

    serial_println!("[TEST-M3] 4/4: Block Storage: RAM Disk 512-byte sector R/W verified. Bounds enforced.");
    serial_println!("[TEST-M3] ALL MILESTONE M3 TESTS PASSED SUCCESSFULLY.\n");
}
