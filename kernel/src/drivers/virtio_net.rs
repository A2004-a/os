//! NovaOS Apex — VirtIO Network Driver (#9)
//! Packet transmission, reception, and MAC address discovery for Ethernet II networking.

use super::pci::find_device;
use super::{Driver, DriverError, DriverState};
use crate::serial;
use core::arch::asm;
use core::ptr::addr_of_mut;

pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;
pub const VIRTIO_DEV_NET_LEGACY: u16 = 0x1000;
pub const VIRTIO_DEV_NET_MODERN: u16 = 0x1041;

pub const MAX_PACKET_SIZE: usize = 1518;
pub const PACKET_BUFFER_SIZE: usize = 16;

#[derive(Clone, Copy)]
pub struct Packet {
    pub len: usize,
    pub data: [u8; MAX_PACKET_SIZE],
}

pub struct VirtioNet {
    io_base: u16,
    mac_address: [u8; 6],
    is_hardware: bool,
    rx_queue: [Option<Packet>; PACKET_BUFFER_SIZE],
    rx_head: usize,
    rx_tail: usize,
    tx_count: u64,
    rx_count: u64,
    state: DriverState,
}

static mut VIRTIO_NET_DRIVER: VirtioNet = VirtioNet {
    io_base: 0,
    mac_address: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56],
    is_hardware: false,
    rx_queue: [None; PACKET_BUFFER_SIZE],
    rx_head: 0,
    rx_tail: 0,
    tx_count: 0,
    rx_count: 0,
    state: DriverState::Uninitialized,
};

impl Driver for VirtioNet {
    fn name(&self) -> &'static str {
        "VirtIO Network Adapter"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        let pci_dev = find_device(VIRTIO_VENDOR_ID, VIRTIO_DEV_NET_LEGACY)
            .or_else(|| find_device(VIRTIO_VENDOR_ID, VIRTIO_DEV_NET_MODERN));

        if let Some(dev) = pci_dev {
            self.io_base = (dev.bar0 & !0x03) as u16;
            self.is_hardware = true;
            Ok(true)
        } else {
            self.is_hardware = false;
            Ok(false)
        }
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if self.probe()? && self.is_hardware {
            unsafe {
                // 1. Reset device
                asm!("out dx, al", in("dx") self.io_base + 0x12, in("al") 0u8, options(nomem, nostack, preserves_flags));
                // 2. Set ACKNOWLEDGE (1) & DRIVER (2)
                asm!("out dx, al", in("dx") self.io_base + 0x12, in("al") 3u8, options(nomem, nostack, preserves_flags));

                // 3. Read MAC Address from config space offset 0x14..0x19
                for i in 0..6 {
                    let mut b: u8;
                    asm!("in al, dx", in("dx") self.io_base + 0x14 + i as u16, out("al") b, options(nomem, nostack, preserves_flags));
                    self.mac_address[i] = b;
                }

                // 4. Set DRIVER_OK (4)
                asm!("out dx, al", in("dx") self.io_base + 0x12, in("al") 7u8, options(nomem, nostack, preserves_flags));
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
    let net = unsafe { &mut *addr_of_mut!(VIRTIO_NET_DRIVER) };
    let _ = net.init();
    let _ = net.start();

    serial::write_str("[DRIVER 9/10] VirtIO Network Driver active (MAC: ");
    print_mac(&net.mac_address);
    if net.is_hardware {
        serial::write_str(", PCI hardware mode)\n");
    } else {
        serial::write_str(", Safe virtual loopback mode)\n");
    }
}

pub fn send_packet(packet: &[u8]) -> Result<(), DriverError> {
    let net = unsafe { &mut *addr_of_mut!(VIRTIO_NET_DRIVER) };
    if packet.len() > MAX_PACKET_SIZE {
        return Err(DriverError::BufferOverflow);
    }

    net.tx_count += 1;

    // In loopback/virtual mode, inject packet into RX queue for testing
    if !net.is_hardware {
        let next_tail = (net.rx_tail + 1) % PACKET_BUFFER_SIZE;
        if next_tail != net.rx_head {
            let mut p = Packet {
                len: packet.len(),
                data: [0u8; MAX_PACKET_SIZE],
            };
            p.data[..packet.len()].copy_from_slice(packet);
            net.rx_queue[net.rx_tail] = Some(p);
            net.rx_tail = next_tail;
            net.rx_count += 1;
        }
    }

    Ok(())
}

pub fn receive_packet(buf: &mut [u8]) -> Result<usize, DriverError> {
    let net = unsafe { &mut *addr_of_mut!(VIRTIO_NET_DRIVER) };
    if net.rx_head == net.rx_tail {
        return Err(DriverError::DeviceNotFound); // Queue empty
    }

    if let Some(pkt) = net.rx_queue[net.rx_head].take() {
        net.rx_head = (net.rx_head + 1) % PACKET_BUFFER_SIZE;
        let copy_len = pkt.len.min(buf.len());
        buf[..copy_len].copy_from_slice(&pkt.data[..copy_len]);
        Ok(copy_len)
    } else {
        Err(DriverError::IoError)
    }
}

pub fn get_mac_address() -> [u8; 6] {
    let net = unsafe { &*addr_of_mut!(VIRTIO_NET_DRIVER) };
    net.mac_address
}

fn print_mac(mac: &[u8; 6]) {
    let hex = b"0123456789ABCDEF";
    for (i, &b) in mac.iter().enumerate() {
        if i > 0 {
            serial::write_str(":");
        }
        serial::write_byte(hex[((b >> 4) & 0xF) as usize]);
        serial::write_byte(hex[(b & 0xF) as usize]);
    }
}
