//! UEFI GOP Linear Framebuffer Graphics Driver

use crate::boot::info::FramebufferInfo;
use crate::sync::Spinlock;

pub struct Framebuffer {
    base_ptr: *mut u8,
    width: usize,
    height: usize,
    stride: usize,
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    pub fn new(info: FramebufferInfo) -> Self {
        Framebuffer {
            base_ptr: info.address as *mut u8,
            width: info.width as usize,
            height: info.height as usize,
            stride: info.stride as usize,
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= self.width || y >= self.height {
            return;
        }

        let pixel_offset = (y * self.stride + x) * 4;
        unsafe {
            let ptr = self.base_ptr.add(pixel_offset);
            *ptr.add(0) = b;
            *ptr.add(1) = g;
            *ptr.add(2) = r;
            *ptr.add(3) = 0xFF; // Alpha
        }
    }

    pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.draw_pixel(x, y, r, g, b);
            }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

pub static FRAMEBUFFER: Spinlock<Option<Framebuffer>> = Spinlock::new(None);

pub fn init(info: FramebufferInfo) {
    let fb = Framebuffer::new(info);
    *FRAMEBUFFER.lock() = Some(fb);
}
