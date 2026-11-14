//! NovaOS Zero — Hardware Driver Subsystem

pub mod drivers_test;
pub mod framebuffer;
pub mod pci;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod serial;
pub mod vga;

pub fn init(fb_info: Option<crate::boot::info::FramebufferInfo>) {
    serial::init();
    ps2_keyboard::init();
    ps2_mouse::init();
    if let Some(info) = fb_info {
        framebuffer::init(info);
    }
}
