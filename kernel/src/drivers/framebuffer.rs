//! NovaOS Apex — GOP Linear Framebuffer Graphics Driver (#2)
//! Direct linear 32-bit rasterizer supporting high-resolution display drawing and blitting.

use super::{Driver, DriverError, DriverState};
use crate::boot::BootInfo;
use crate::serial;
use core::ptr::addr_of_mut;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub raw: u32,
}

impl Color {
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const NOVA_BLUE: Self = Self::from_rgb(10, 14, 23);
    pub const NOVA_CYAN: Self = Self::from_rgb(0, 212, 255);
    pub const NOVA_PURPLE: Self = Self::from_rgb(120, 80, 255);
    pub const DARK_GRAY: Self = Self::from_rgb(40, 44, 52);
    pub const GREEN: Self = Self::from_rgb(80, 250, 123);
    pub const RED: Self = Self::from_rgb(255, 85, 85);

    #[inline(always)]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            raw: ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }
}

pub struct FramebufferDriver {
    pub base_ptr: *mut u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub size_bytes: usize,
    state: DriverState,
}

static mut FRAMEBUFFER_DRIVER: FramebufferDriver = FramebufferDriver {
    base_ptr: core::ptr::null_mut(),
    width: 0,
    height: 0,
    stride: 0,
    size_bytes: 0,
    state: DriverState::Uninitialized,
};

impl Driver for FramebufferDriver {
    fn name(&self) -> &'static str {
        "UEFI GOP Framebuffer"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        Ok(!self.base_ptr.is_null() && self.width > 0 && self.height > 0)
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if !self.probe()? {
            self.state = DriverState::Error;
            return Err(DriverError::DeviceNotFound);
        }
        self.state = DriverState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Initialized && self.state != DriverState::Stopped {
            return Err(DriverError::NotStarted);
        }
        self.clear(Color::NOVA_BLUE);
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.clear(Color::BLACK);
        self.state = DriverState::Stopped;
        Ok(())
    }

    fn state(&self) -> DriverState {
        self.state
    }
}

impl FramebufferDriver {
    pub fn clear(&mut self, color: Color) {
        if self.base_ptr.is_null() {
            return;
        }
        let total_pixels = (self.stride * self.height) as usize;
        unsafe {
            for i in 0..total_pixels {
                *self.base_ptr.add(i) = color.raw;
            }
        }
    }

    #[inline(always)]
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height || self.base_ptr.is_null() {
            return;
        }
        let offset = (y * self.stride + x) as usize;
        unsafe {
            *self.base_ptr.add(offset) = color.raw;
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for cy in y..y_end {
            let row_offset = (cy * self.stride) as usize;
            for cx in x..x_end {
                unsafe {
                    *self.base_ptr.add(row_offset + cx as usize) = color.raw;
                }
            }
        }
    }

    pub fn blit(&mut self, x: u32, y: u32, w: u32, h: u32, src: &[u32]) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for cy in y..y_end {
            let row_idx = (cy - y) as usize;
            let dest_offset = (cy * self.stride + x) as usize;
            let src_offset = row_idx * w as usize;
            let copy_w = (x_end - x) as usize;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    src.as_ptr().add(src_offset),
                    self.base_ptr.add(dest_offset),
                    copy_w,
                );
            }
        }
    }
}

pub fn init(boot_info: &BootInfo) {
    let fb = unsafe { &mut *addr_of_mut!(FRAMEBUFFER_DRIVER) };
    if boot_info.framebuffer_base != 0 && boot_info.horizontal_resolution > 0 {
        fb.base_ptr = boot_info.framebuffer_base as *mut u32;
        fb.width = boot_info.horizontal_resolution;
        fb.height = boot_info.vertical_resolution;
        fb.stride = boot_info.pixels_per_scan_line;
        fb.size_bytes = boot_info.framebuffer_size as usize;

        if let Ok(_) = fb.init() {
            let _ = fb.start();
            serial::write_str("[DRIVER 2/10] GOP Linear Framebuffer Driver initialized & active (");
            print_u64(fb.width as u64);
            serial::write_str("x");
            print_u64(fb.height as u64);
            serial::write_str(")\n");
        }
    } else {
        serial::write_str("[DRIVER 2/10 WARN] Framebuffer not present or headless mode active.\n");
    }
}

pub fn get_driver() -> &'static mut FramebufferDriver {
    unsafe { &mut *addr_of_mut!(FRAMEBUFFER_DRIVER) }
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
