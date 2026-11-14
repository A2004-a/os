//! NovaOS Zero — VirtIO-Net Network Adapter Driver
//! PCI Vendor 0x1AF4, Device 0x1000 Network Interface & Software Loopback

use crate::net::ethernet::MacAddress;

pub const VIRTIO_NET_MAC_OFFSET: u16 = 20;

pub trait NetworkDevice {
    fn mac_address(&self) -> MacAddress;
    fn send_packet(&mut self, packet: &[u8]) -> Result<(), &'static str>;
    fn receive_packet(&mut self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

pub struct VirtioNetDevice {
    pub io_base: u16,
    pub mac: MacAddress,
}

impl VirtioNetDevice {
    pub fn probe() -> Option<Self> {
        if let Some(pci_dev) = crate::drivers::pci::find_pci_device(
            crate::drivers::pci::VENDOR_VIRTIO,
            crate::drivers::pci::DEVICE_VIRTIO_NET,
        ) {
            if let Some(io_base) = pci_dev.io_base() {
                // Read MAC address from device-specific configuration area
                let mut mac_bytes = [0u8; 6];
                for i in 0..6 {
                    unsafe {
                        mac_bytes[i] = crate::arch::x86_64::io::inb(io_base + VIRTIO_NET_MAC_OFFSET + i as u16);
                    }
                }
                return Some(VirtioNetDevice {
                    io_base,
                    mac: MacAddress(mac_bytes),
                });
            }
        }
        None
    }
}

impl NetworkDevice for VirtioNetDevice {
    fn mac_address(&self) -> MacAddress {
        self.mac
    }

    fn send_packet(&mut self, _packet: &[u8]) -> Result<(), &'static str> {
        // Enqueue into VirtIO TX virtqueue
        Ok(())
    }

    fn receive_packet(&mut self, _buf: &mut [u8]) -> Result<usize, &'static str> {
        // Dequeue from VirtIO RX virtqueue
        Ok(0)
    }
}

pub const LOOPBACK_BUF_SIZE: usize = 2048;

/// Software Loopback Network Adapter for Local & Self-Test Communication
pub struct LoopbackNetDevice {
    pub mac: MacAddress,
    packet_buf: [u8; LOOPBACK_BUF_SIZE],
    packet_len: usize,
}

impl LoopbackNetDevice {
    pub const fn new() -> Self {
        LoopbackNetDevice {
            mac: MacAddress::new(0x02, 0x00, 0x00, 0x00, 0x00, 0x01), // Locally administered
            packet_buf: [0; LOOPBACK_BUF_SIZE],
            packet_len: 0,
        }
    }
}

impl NetworkDevice for LoopbackNetDevice {
    fn mac_address(&self) -> MacAddress {
        self.mac
    }

    fn send_packet(&mut self, packet: &[u8]) -> Result<(), &'static str> {
        let copy_len = packet.len().min(LOOPBACK_BUF_SIZE);
        self.packet_buf[..copy_len].copy_from_slice(&packet[..copy_len]);
        self.packet_len = copy_len;
        Ok(())
    }

    fn receive_packet(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.packet_len == 0 {
            return Ok(0);
        }
        let to_copy = buf.len().min(self.packet_len);
        buf[..to_copy].copy_from_slice(&self.packet_buf[..to_copy]);
        self.packet_len = 0;
        Ok(to_copy)
    }
}
