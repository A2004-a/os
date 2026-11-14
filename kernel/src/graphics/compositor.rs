//! NovaOS Zero — 2D Software Graphics Compositor
//! RGBA8888 Color Blending, Surface Backbuffers, Clipping Rectangles, and Geometric Primitives

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self       = Self { r: 0x00, g: 0x00, b: 0x00, a: 0xFF };
    pub const WHITE: Self       = Self { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
    pub const RED: Self         = Self { r: 0xEF, g: 0x44, b: 0x44, a: 0xFF };
    pub const GREEN: Self       = Self { r: 0x10, g: 0xB9, b: 0x81, a: 0xFF };
    pub const BLUE: Self        = Self { r: 0x3B, g: 0x82, b: 0xF6, a: 0xFF };
    pub const DARK_SLATE: Self  = Self { r: 0x0F, g: 0x17, b: 0x2A, a: 0xFF }; // Modern desktop bg
    pub const WINDOW_BG: Self   = Self { r: 0x1E, g: 0x29, b: 0x3B, a: 0xFF }; // Modern panel bg
    pub const TITLEBAR_BG: Self = Self { r: 0x33, g: 0x41, b: 0x55, a: 0xFF };
    pub const BORDER: Self      = Self { r: 0x47, g: 0x55, b: 0x69, a: 0xFF };
    pub const TRANSPARENT: Self = Self { r: 0x00, g: 0x00, b: 0x00, a: 0x00 };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Linear alpha blending: src over dst
    pub fn blend(src: Color, dst: Color) -> Color {
        if src.a == 255 {
            return src;
        }
        if src.a == 0 {
            return dst;
        }

        let alpha = src.a as u32;
        let inv_alpha = 255 - alpha;

        let r = ((src.r as u32 * alpha) + (dst.r as u32 * inv_alpha)) / 255;
        let g = ((src.g as u32 * alpha) + (dst.g as u32 * inv_alpha)) / 255;
        let b = ((src.b as u32 * alpha) + (dst.b as u32 * inv_alpha)) / 255;

        Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: 255,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + self.width as i32
            && py >= self.y
            && py < self.y + self.height as i32
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }
}

pub const MAX_SURFACE_PIXELS: usize = 1024 * 768;

/// Offscreen Software Backbuffer Surface
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub pixels: [Color; 256 * 256], // Standard fixed-size surface for window backing store
}

impl Surface {
    pub const fn new(width: u32, height: u32) -> Self {
        Surface {
            width,
            height,
            pixels: [Color::TRANSPARENT; 256 * 256],
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as usize * self.width as usize) + x as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = Color::blend(color, self.pixels[idx]);
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as usize * self.width as usize) + x as usize;
            if idx < self.pixels.len() {
                return self.pixels[idx];
            }
        }
        Color::TRANSPARENT
    }

    pub fn clear(&mut self, color: Color) {
        let count = (self.width * self.height) as usize;
        let limit = count.min(self.pixels.len());
        for p in self.pixels[..limit].iter_mut() {
            *p = color;
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(0);
        let x1 = (rect.x + rect.width as i32).min(self.width as i32);
        let y1 = (rect.y + rect.height as i32).min(self.height as i32);

        for y in y0..y1 {
            for x in x0..x1 {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn draw_rect_outline(&mut self, rect: Rect, border_width: u32, color: Color) {
        // Top
        self.fill_rect(Rect::new(rect.x, rect.y, rect.width, border_width), color);
        // Bottom
        self.fill_rect(
            Rect::new(rect.x, rect.y + rect.height as i32 - border_width as i32, rect.width, border_width),
            color,
        );
        // Left
        self.fill_rect(Rect::new(rect.x, rect.y, border_width, rect.height), color);
        // Right
        self.fill_rect(
            Rect::new(rect.x + rect.width as i32 - border_width as i32, rect.y, border_width, rect.height),
            color,
        );
    }

    /// Bresenham's Line Algorithm with bounds protection
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel(x0, y0, color);
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

    /// Midpoint Circle Algorithm
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx + y, cy + x, color);
            self.set_pixel(cx - y, cy + x, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx - x, cy - y, color);
            self.set_pixel(cx - y, cy - x, color);
            self.set_pixel(cx + y, cy - x, color);
            self.set_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Draw text using built-in bitmap font
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color) {
        let mut cur_x = x;
        for c in text.chars() {
            let glyph = super::font::get_glyph(c);
            for row in 0..super::font::FONT_HEIGHT {
                let row_byte = glyph[row];
                for col in 0..super::font::FONT_WIDTH {
                    if (row_byte & (0x80 >> col)) != 0 {
                        self.set_pixel(cur_x + col as i32, y + row as i32, color);
                    }
                }
            }
            cur_x += super::font::FONT_WIDTH as i32;
        }
    }

    /// Blit another surface onto this surface at target (dst_x, dst_y)
    pub fn blit(&mut self, src: &Surface, dst_x: i32, dst_y: i32) {
        for sy in 0..src.height as i32 {
            for sx in 0..src.width as i32 {
                let color = src.get_pixel(sx, sy);
                if color.a > 0 {
                    self.set_pixel(dst_x + sx, dst_y + sy, color);
                }
            }
        }
    }
}
