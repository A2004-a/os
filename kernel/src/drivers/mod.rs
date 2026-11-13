pub mod framebuffer;
pub mod pci;
pub mod ps2_keyboard;
pub mod serial;
pub mod vga;

pub fn init(fb_info: Option<crate::boot::info::FramebufferInfo>) {
    serial::init();
    ps2_keyboard::init();
    if let Some(info) = fb_info {
        framebuffer::init(info);
    }
}
