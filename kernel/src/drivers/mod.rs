//! NovaOS Zero — Hardware Driver Subsystem

pub mod ahci;
pub mod audio;
pub mod drivers_test;
pub mod framebuffer;
pub mod hardware_test;
pub mod nvme;
pub mod pci;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod serial;
pub mod vga;
pub mod virtio_net;
pub mod xhci;

pub fn init(fb_info: Option<crate::boot::info::FramebufferInfo>) {
    serial::init();
    ps2_keyboard::init();
    ps2_mouse::init();
    if let Some(info) = fb_info {
        framebuffer::init(info);
    }
}
