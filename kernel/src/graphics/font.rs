//! NovaOS Apex — Bitmap Font Rasterizer & Text Rendering Engine
//! Renders 8x16 monospace text with alpha blending and clipping into surfaces.

use super::color::Color;
use super::geometry::{Rect, Size};
use super::surface::Surface;
use crate::drivers::console::get_glyph_data;

pub const FONT_WIDTH: u32 = 8;
pub const FONT_HEIGHT: u32 = 16;

pub fn measure_text(text: &str) -> Size {
    Size::new(text.len() as u32 * FONT_WIDTH, FONT_HEIGHT)
}

pub fn draw_char(surface: &mut Surface, c: char, x: i32, y: i32, color: Color) {
    let byte_val = if (c as u32) < 128 { c as u8 } else { b'?' };
    let glyph = get_glyph_data(byte_val);

    for row in 0..16 {
        let py = y + row as i32;
        if py < 0 || (py as u32) >= surface.height {
            continue;
        }

        let row_bits = glyph[row];
        for bit in 0..8 {
            let px = x + bit as i32;
            if px < 0 || (px as u32) >= surface.width {
                continue;
            }

            if (row_bits & (0x80 >> bit)) != 0 {
                surface.blend_pixel(px as u32, py as u32, color);
            }
        }
    }
}

pub fn draw_text(surface: &mut Surface, text: &str, mut x: i32, y: i32, color: Color) {
    for ch in text.chars() {
        if ch == '\n' {
            // Newlines can be handled if needed, or caller aligns
            continue;
        }
        draw_char(surface, ch, x, y, color);
        x += FONT_WIDTH as i32;
    }
}

pub fn draw_text_with_bg(surface: &mut Surface, text: &str, x: i32, y: i32, fg: Color, bg: Color) {
    let text_size = measure_text(text);
    surface.fill_rect(Rect::new(x, y, text_size.width, text_size.height), bg);
    draw_text(surface, text, x, y, fg);
}
