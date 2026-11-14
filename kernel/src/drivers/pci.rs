//! NovaOS Zero — Freestanding PCI Bus Enumerator & Device Discovery (0xCF8 / 0xCFC)

use crate::arch::x86_64::io::{inl, outl};

pub const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
pub const PCI_CONFIG_DATA: u16    = 0xCFC;

pub const VENDOR_VIRTIO: u16      = 0x1AF4;
pub const DEVICE_VIRTIO_NET: u16  = 0x1000;
pub const DEVICE_VIRTIO_BLK: u16  = 0x1001;

#[derive(Debug, Clone, Copy, Default)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u8,
    pub subclass_id: u8,
    pub bar0: u32,
    pub irq_line: u8,
}

impl PciDevice {
    /// Return I/O port base address if BAR0 is an I/O port
    pub fn io_base(&self) -> Option<u16> {
        if (self.bar0 & 0x01) != 0 {
            Some((self.bar0 & 0xFFFC) as u16)
        } else {
            None
        }
    }
}

pub fn pci_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
        inl(PCI_CONFIG_DATA)
    }
}

pub fn pci_write_u32(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    unsafe {
        outl(PCI_CONFIG_ADDRESS, address);
        outl(PCI_CONFIG_DATA, value);
    }
}

/// Scan PCI buses 0..8 for attached peripheral controllers
pub fn scan_pci_bus() -> [Option<PciDevice>; 32] {
    let mut devices = [None; 32];
    let mut count = 0;

    for bus in 0..=8 {
        for slot in 0..32 {
            let data = pci_read_u32(bus, slot, 0, 0);
            let vendor_id = (data & 0xFFFF) as u16;

            // 0xFFFF indicates no hardware present
            if vendor_id != 0xFFFF && count < 32 {
                let device_id = ((data >> 16) & 0xFFFF) as u16;
                let class_data = pci_read_u32(bus, slot, 0, 8);
                let class_id = ((class_data >> 24) & 0xFF) as u8;
                let subclass_id = ((class_data >> 16) & 0xFF) as u8;

                let bar0 = pci_read_u32(bus, slot, 0, 0x10);
                let irq_data = pci_read_u32(bus, slot, 0, 0x3C);
                let irq_line = (irq_data & 0xFF) as u8;

                devices[count] = Some(PciDevice {
                    bus,
                    device: slot,
                    function: 0,
                    vendor_id,
                    device_id,
                    class_id,
                    subclass_id,
                    bar0,
                    irq_line,
                });
                count += 1;
            }
        }
    }

    devices
}

/// Find a specific device by vendor and device ID
pub fn find_pci_device(vendor: u16, device: u16) -> Option<PciDevice> {
    for dev_opt in scan_pci_bus() {
        if let Some(dev) = dev_opt {
            if dev.vendor_id == vendor && dev.device_id == device {
                return Some(dev);
            }
        }
    }
    None
}
