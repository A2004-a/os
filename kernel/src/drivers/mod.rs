//! NovaOS Apex — First-Party Device Driver Subsystem & Hardware Lifecycle Manager
//! Implements strict probe/init/start/stop lifecycles for all system peripherals.

pub mod ahci;
pub mod audio;
pub mod console;
pub mod framebuffer;
pub mod hardware_tests;
pub mod hda;
pub mod input_tests;
pub mod keyboard;
pub mod mouse;
pub mod nvme;
pub mod pci;
pub mod pic;
pub mod serial;
pub mod tests;
pub mod timer;
pub mod usb_hid;
pub mod virtio_block;
pub mod virtio_net;
pub mod xhci;

use crate::boot::BootInfo;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DriverError {
    DeviceNotFound,
    InitFailed,
    IoError,
    UnsupportedDevice,
    AlreadyStarted,
    NotStarted,
    BufferOverflow,
    Timeout,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DriverState {
    Uninitialized,
    Initialized,
    Running,
    Stopped,
    Error,
}

pub trait Driver {
    fn name(&self) -> &'static str;
    fn probe(&mut self) -> Result<bool, DriverError>;
    fn init(&mut self) -> Result<(), DriverError>;
    fn start(&mut self) -> Result<(), DriverError>;
    fn stop(&mut self) -> Result<(), DriverError>;
    fn state(&self) -> DriverState;
}

pub fn init(boot_info: &BootInfo) {
    // 1. UART Serial Driver
    serial::init();

    // 2. GOP Framebuffer Driver
    framebuffer::init(boot_info);

    // 3. Text Console Driver
    console::init();

    // 4. 8259 PIC & 8254 PIT Timer Driver (1000 Hz)
    pic::init();
    timer::init();

    // 5. PS/2 Keyboard Driver
    keyboard::init();

    // 6. PS/2 Mouse Driver
    mouse::init(boot_info.horizontal_resolution, boot_info.vertical_resolution);

    // 7. PCI Bus Enumeration Driver
    pci::init();

    // 8. VirtIO Block Storage Driver
    virtio_block::init();

    // 9. VirtIO Network Driver
    virtio_net::init();

    // 10. Basic Audio Abstraction (PC Speaker)
    audio::init();

    // 11. AHCI SATA 1.3 Storage Driver
    ahci::init();

    // 12. NVMe 1.4 Storage Driver
    nvme::init();

    // 13. USB 3.0 xHCI & USB HID Driver
    xhci::init();
    usb_hid::init();

    // 14. Intel High Definition Audio Driver
    hda::init();
}
