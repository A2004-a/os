//! NovaOS Apex — 2D Drawing Surface & Blitter Engine
//! Provides memory-safe pixel buffers, alpha blending, primitive rasterization, and clipping.

use super::color::Color;
use super::geometry::Rect;
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixels: Vec<u32>,
}

impl Surface {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let mut pixels = Vec::with_capacity(size);
        pixels.resize(size, Color::TRANSPARENT.raw);
        Self {
            width,
            height,
            stride: width,
            pixels,
        }
    }

    pub fn with_color(width: u32, height: u32, color: Color) -> Self {
        let size = (width * height) as usize;
        let mut pixels = Vec::with_capacity(size);
        pixels.resize(size, color.raw);
        Self {
            width,
            height,
            stride: width,
            pixels,
        }
    }

    #[inline(always)]
    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.raw);
    }

    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            let idx = (y * self.stride + x) as usize;
            Color { raw: self.pixels[idx] }
        } else {
            Color::TRANSPARENT
        }
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.stride + x) as usize;
            self.pixels[idx] = color.raw;
        }
    }

    #[inline(always)]
    pub fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.stride + x) as usize;
            let dst = Color { raw: self.pixels[idx] };
            self.pixels[idx] = Color::blend(color, dst).raw;
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        if let Some(clipped) = rect.intersection(&self.bounds()) {
            if color.a() == 255 {
                // Fast opaque fill
                for cy in clipped.y..clipped.bottom() {
                    let row_start = (cy as u32 * self.stride + clipped.x as u32) as usize;
                    let row_end = row_start + clipped.width as usize;
                    self.pixels[row_start..row_end].fill(color.raw);
                }
            } else if color.a() > 0 {
                // Alpha blend fill
                for cy in clipped.y..clipped.bottom() {
                    for cx in clipped.x..clipped.right() {
                        self.blend_pixel(cx as u32, cy as u32, color);
                    }
                }
            }
        }
    }

    pub fn fill_rect_alpha(&mut self, rect: Rect, color: Color, opacity: u8) {
        if opacity == 0 {
            return;
        }
        if let Some(clipped) = rect.intersection(&self.bounds()) {
            for cy in clipped.y..clipped.bottom() {
                for cx in clipped.x..clipped.right() {
                    let idx = (cy as u32 * self.stride + cx as u32) as usize;
                    let dst = Color { raw: self.pixels[idx] };
                    self.pixels[idx] = Color::blend_with_opacity(color, dst, opacity).raw;
                }
            }
        }
    }

    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color, thickness: u32) {
        if thickness == 0 || rect.is_empty() {
            return;
        }
        // Top
        self.fill_rect(Rect::new(rect.x, rect.y, rect.width, thickness), color);
        // Bottom
        self.fill_rect(Rect::new(rect.x, rect.bottom() - thickness as i32, rect.width, thickness), color);
        // Left
        self.fill_rect(Rect::new(rect.x, rect.y, thickness, rect.height), color);
        // Right
        self.fill_rect(Rect::new(rect.right() - thickness as i32, rect.y, thickness, rect.height), color);
    }

    pub fn draw_rounded_rect(&mut self, rect: Rect, radius: u32, color: Color) {
        if radius == 0 {
            self.fill_rect(rect, color);
            return;
        }

        let r = radius as i32;
        let r_sq = r * r;

        // Fill center body
        let inner_rect = Rect::new(rect.x + r, rect.y, rect.width.saturating_sub(radius * 2), rect.height);
        self.fill_rect(inner_rect, color);

        // Fill left and right strips (excluding corners)
        let left_strip = Rect::new(rect.x, rect.y + r, radius, rect.height.saturating_sub(radius * 2));
        let right_strip = Rect::new(rect.right() - r, rect.y + r, radius, rect.height.saturating_sub(radius * 2));
        self.fill_rect(left_strip, color);
        self.fill_rect(right_strip, color);

        // Corner circles: Top-Left, Top-Right, Bottom-Left, Bottom-Right
        let cx_tl = rect.x + r;
        let cy_tl = rect.y + r;

        let cx_tr = rect.right() - r - 1;
        let cy_tr = rect.y + r;

        let cx_bl = rect.x + r;
        let cy_bl = rect.bottom() - r - 1;

        let cx_br = rect.right() - r - 1;
        let cy_br = rect.bottom() - r - 1;

        for dy in 0..r {
            for dx in 0..r {
                let dist_sq = (r - 1 - dx) * (r - 1 - dx) + (r - 1 - dy) * (r - 1 - dy);
                if dist_sq <= r_sq {
                    // Top-Left
                    self.blend_pixel((cx_tl - r + dx) as u32, (cy_tl - r + dy) as u32, color);
                    // Top-Right
                    self.blend_pixel((cx_tr + r - dx) as u32, (cy_tr - r + dy) as u32, color);
                    // Bottom-Left
                    self.blend_pixel((cx_bl - r + dx) as u32, (cy_bl + r - dy) as u32, color);
                    // Bottom-Right
                    self.blend_pixel((cx_br + r - dx) as u32, (cy_br + r - dy) as u32, color);
                }
            }
        }
    }

    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && (x0 as u32) < self.width && y0 >= 0 && (y0 as u32) < self.height {
                self.blend_pixel(x0 as u32, y0 as u32, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Blit an entire or sub-rect of source surface onto this surface (with alpha blending)
    pub fn blit(&mut self, dest_x: i32, dest_y: i32, src: &Surface, src_rect: Option<Rect>) {
        self.blit_alpha(dest_x, dest_y, src, src_rect, 255);
    }

    /// Blit with per-pixel alpha and additional global opacity
    pub fn blit_alpha(&mut self, dest_x: i32, dest_y: i32, src: &Surface, src_rect: Option<Rect>, opacity: u8) {
        if opacity == 0 {
            return;
        }

        let srect = src_rect.unwrap_or_else(|| src.bounds());
        if let Some(clipped_src) = srect.intersection(&src.bounds()) {
            for sy in 0..clipped_src.height as i32 {
                let dy = dest_y + sy;
                if dy < 0 || (dy as u32) >= self.height {
                    continue;
                }

                for sx in 0..clipped_src.width as i32 {
                    let dx = dest_x + sx;
                    if dx < 0 || (dx as u32) >= self.width {
                        continue;
                    }

                    let src_px = (clipped_src.x + sx) as u32;
                    let src_py = (clipped_src.y + sy) as u32;
                    let src_color = src.get_pixel(src_px, src_py);

                    if src_color.a() > 0 {
                        let dst_color = self.get_pixel(dx as u32, dy as u32);
                        let final_color = Color::blend_with_opacity(src_color, dst_color, opacity);
                        self.set_pixel(dx as u32, dy as u32, final_color);
                    }
                }
            }
        }
    }

    /// Blit a clipped region directly to a raw framebuffer memory pointer (e.g. GOP framebuffer)
    pub fn blit_clipped_to_raw(&self, dest_base: *mut u32, dest_stride: u32, dest_w: u32, dest_h: u32, clip: Rect) {
        if dest_base.is_null() {
            return;
        }
        let screen_bounds = Rect::new(0, 0, dest_w, dest_h);
        if let Some(c) = clip.intersection(&screen_bounds) {
            for cy in c.y..c.bottom() {
                let src_row = (cy as u32 * self.stride + c.x as u32) as usize;
                let dst_row = (cy as u32 * dest_stride + c.x as u32) as usize;
                let copy_len = c.width as usize;

                unsafe {
                    core::ptr::copy_nonoverlapping(
                        self.pixels.as_ptr().add(src_row),
                        dest_base.add(dst_row),
                        copy_len,
                    );
                }
            }
        }
    }
}
