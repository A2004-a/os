//! NovaOS Apex — PCI Bus Enumeration Driver (#7)
//! Discovers PCI endpoints, bridges, BARs, and interrupt line routing via I/O configuration space.

use super::{Driver, DriverError, DriverState};
use core::arch::asm;
use core::ptr::addr_of_mut;
use crate::serial;

pub const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
pub const PCI_CONFIG_DATA: u16 = 0xCFC;
pub const MAX_PCI_DEVICES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,
    pub irq_line: u8,
}

pub struct PciBusEnumerator {
    devices: [Option<PciDevice>; MAX_PCI_DEVICES],
    device_count: usize,
    state: DriverState,
}

static mut PCI_DRIVER: PciBusEnumerator = PciBusEnumerator {
    devices: [None; MAX_PCI_DEVICES],
    device_count: 0,
    state: DriverState::Uninitialized,
};

pub fn pci_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        asm!("out dx, eax", in("dx") PCI_CONFIG_ADDRESS, in("eax") address, options(nomem, nostack, preserves_flags));
        let mut val: u32;
        asm!("in eax, dx", in("dx") PCI_CONFIG_DATA, out("eax") val, options(nomem, nostack, preserves_flags));
        val
    }
}

pub fn pci_write_u32(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        asm!("out dx, eax", in("dx") PCI_CONFIG_ADDRESS, in("eax") address, options(nomem, nostack, preserves_flags));
        asm!("out dx, eax", in("dx") PCI_CONFIG_DATA, in("eax") val, options(nomem, nostack, preserves_flags));
    }
}

impl Driver for PciBusEnumerator {
    fn name(&self) -> &'static str {
        "PCI Bus Enumerator"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        // Bus 0, Slot 0 (Host Bridge) usually responds with non-0xFFFF vendor
        let vendor = pci_read_u32(0, 0, 0, 0) & 0xFFFF;
        Ok(vendor != 0xFFFF)
    }

    fn init(&mut self) -> Result<(), DriverError> {
        self.device_count = 0;
        for dev in self.devices.iter_mut() {
            *dev = None;
        }

        // Scan buses 0..8, slots 0..31, functions 0..7
        for bus in 0..8 {
            for slot in 0..32 {
                let vendor = pci_read_u32(bus, slot, 0, 0) & 0xFFFF;
                if vendor == 0xFFFF {
                    continue;
                }

                let header_type = ((pci_read_u32(bus, slot, 0, 0x0C) >> 16) & 0xFF) as u8;
                let max_func = if (header_type & 0x80) != 0 { 8 } else { 1 };

                for func in 0..max_func {
                    let id_reg = pci_read_u32(bus, slot, func, 0);
                    let vendor_id = (id_reg & 0xFFFF) as u16;
                    let device_id = ((id_reg >> 16) & 0xFFFF) as u16;

                    if vendor_id == 0xFFFF {
                        continue;
                    }

                    let class_reg = pci_read_u32(bus, slot, func, 0x08);
                    let _revision = (class_reg & 0xFF) as u8;
                    let prog_if = ((class_reg >> 8) & 0xFF) as u8;
                    let subclass = ((class_reg >> 16) & 0xFF) as u8;
                    let class_code = ((class_reg >> 24) & 0xFF) as u8;

                    let bar0 = pci_read_u32(bus, slot, func, 0x10);
                    let bar1 = pci_read_u32(bus, slot, func, 0x14);
                    let bar2 = pci_read_u32(bus, slot, func, 0x18);
                    let bar3 = pci_read_u32(bus, slot, func, 0x1C);
                    let bar4 = pci_read_u32(bus, slot, func, 0x20);
                    let bar5 = pci_read_u32(bus, slot, func, 0x24);

                    let intr_reg = pci_read_u32(bus, slot, func, 0x3C);
                    let irq_line = (intr_reg & 0xFF) as u8;

                    let dev = PciDevice {
                        bus,
                        slot,
                        func,
                        vendor_id,
                        device_id,
                        class_code,
                        subclass,
                        prog_if,
                        bar0,
                        bar1,
                        bar2,
                        bar3,
                        bar4,
                        bar5,
                        irq_line,
                    };

                    if self.device_count < MAX_PCI_DEVICES {
                        self.devices[self.device_count] = Some(dev);
                        self.device_count += 1;
                    }
                }
            }
        }

        self.state = DriverState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Stopped;
        Ok(())
    }

    fn state(&self) -> DriverState {
        self.state
    }
}

pub fn init() {
    let pci = unsafe { &mut *addr_of_mut!(PCI_DRIVER) };
    if let Ok(_) = pci.init() {
        let _ = pci.start();
        serial::write_str("[DRIVER 7/10] PCI Bus Enumerator active. Discovered ");
        print_u64(pci.device_count as u64);
        serial::write_str(" peripheral devices.\n");

        for i in 0..pci.device_count {
            if let Some(ref d) = pci.devices[i] {
                serial::write_str("  PCI [");
                print_u64(d.bus as u64); serial::write_str(":");
                print_u64(d.slot as u64); serial::write_str(":");
                print_u64(d.func as u64); serial::write_str("] Vendor: 0x");
                print_hex_u16(d.vendor_id);
                serial::write_str(", Device: 0x");
                print_hex_u16(d.device_id);
                serial::write_str(" (Class ");
                print_hex_u8(d.class_code);
                serial::write_str(":");
                print_hex_u8(d.subclass);
                serial::write_str(")\n");
            }
        }
    }
}

pub fn find_device(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    let pci = unsafe { &*addr_of_mut!(PCI_DRIVER) };
    for i in 0..pci.device_count {
        if let Some(ref d) = pci.devices[i] {
            if d.vendor_id == vendor_id && d.device_id == device_id {
                return Some(*d);
            }
        }
    }
    None
}

pub fn find_by_class(class_code: u8, subclass: u8) -> Option<PciDevice> {
    let pci = unsafe { &*addr_of_mut!(PCI_DRIVER) };
    for i in 0..pci.device_count {
        if let Some(ref d) = pci.devices[i] {
            if d.class_code == class_code && d.subclass == subclass {
                return Some(*d);
            }
        }
    }
    None
}

pub fn find_by_class_and_progif(class_code: u8, subclass: u8, prog_if: u8) -> Option<PciDevice> {
    let pci = unsafe { &*addr_of_mut!(PCI_DRIVER) };
    for i in 0..pci.device_count {
        if let Some(ref d) = pci.devices[i] {
            if d.class_code == class_code && d.subclass == subclass && d.prog_if == prog_if {
                return Some(*d);
            }
        }
    }
    None
}

pub fn pci_enable_bus_master(bus: u8, slot: u8, func: u8) {
    let cmd = pci_read_u32(bus, slot, func, 0x04);
    // Bit 0: I/O Space, Bit 1: Memory Space, Bit 2: Bus Master
    let new_cmd = cmd | 0x07;
    pci_write_u32(bus, slot, func, 0x04, new_cmd);
}

pub fn list_all_devices(out: &mut [PciDevice]) -> usize {
    let pci = unsafe { &*addr_of_mut!(PCI_DRIVER) };
    let count = pci.device_count.min(out.len());
    let mut copied = 0;
    for i in 0..pci.device_count {
        if copied >= count {
            break;
        }
        if let Some(ref d) = pci.devices[i] {
            out[copied] = *d;
            copied += 1;
        }
    }
    copied
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

fn print_hex_u16(n: u16) {
    let hex = b"0123456789ABCDEF";
    serial::write_byte(hex[((n >> 12) & 0xF) as usize]);
    serial::write_byte(hex[((n >> 8) & 0xF) as usize]);
    serial::write_byte(hex[((n >> 4) & 0xF) as usize]);
    serial::write_byte(hex[(n & 0xF) as usize]);
}

fn print_hex_u8(n: u8) {
    let hex = b"0123456789ABCDEF";
    serial::write_byte(hex[((n >> 4) & 0xF) as usize]);
    serial::write_byte(hex[(n & 0xF) as usize]);
}
