//! NovaOS Apex — Hardware-Calibrated Software Mouse Cursor Engine
//! 16x16 high-contrast arrow sprite with dirty backdrop saving and non-destructive restoration.

use super::color::Color;
use super::geometry::{Point, Rect};
use super::surface::Surface;

pub const CURSOR_WIDTH: u32 = 16;
pub const CURSOR_HEIGHT: u32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorType {
    Arrow,
    Pointer,
    Text,
}

pub struct Cursor {
    pub cursor_type: CursorType,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    saved_under: [u32; (CURSOR_WIDTH * CURSOR_HEIGHT) as usize],
    saved_rect: Rect,
    has_saved: bool,
}

// 16x16 standard high-visibility cursor sprite:
// 0: Transparent, 1: Black Outline, 2: White Interior
static ARROW_SPRITE: [u8; 16 * 16] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
];

impl Cursor {
    pub const fn new() -> Self {
        Self {
            cursor_type: CursorType::Arrow,
            x: 0,
            y: 0,
            visible: true,
            saved_under: [0; (CURSOR_WIDTH * CURSOR_HEIGHT) as usize],
            saved_rect: Rect::ZERO,
            has_saved: false,
        }
    }

    #[inline(always)]
    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    #[inline(always)]
    pub fn current_rect(&self) -> Rect {
        Rect::new(self.x, self.y, CURSOR_WIDTH, CURSOR_HEIGHT)
    }

    /// Restore the underlying pixels back to the surface
    pub fn restore_background(&mut self, surface: &mut Surface) {
        if !self.has_saved || self.saved_rect.is_empty() {
            return;
        }

        for sy in 0..self.saved_rect.height as i32 {
            let dy = self.saved_rect.y + sy;
            if dy < 0 || (dy as u32) >= surface.height {
                continue;
            }
            for sx in 0..self.saved_rect.width as i32 {
                let dx = self.saved_rect.x + sx;
                if dx < 0 || (dx as u32) >= surface.width {
                    continue;
                }
                let saved_idx = (sy as usize) * (CURSOR_WIDTH as usize) + (sx as usize);
                surface.set_pixel(dx as u32, dy as u32, Color { raw: self.saved_under[saved_idx] });
            }
        }
        self.has_saved = false;
    }

    /// Save the underlying pixels and draw the cursor sprite
    pub fn render_to(&mut self, surface: &mut Surface) {
        if !self.visible {
            return;
        }

        // Clamp cursor rectangle to surface bounds
        let cursor_rect = self.current_rect();
        let clip_opt = cursor_rect.intersection(&surface.bounds());
        if clip_opt.is_none() {
            return;
        }

        // 1. Save underlying pixels
        self.saved_rect = cursor_rect;
        for sy in 0..CURSOR_HEIGHT as i32 {
            let screen_y = self.y + sy;
            for sx in 0..CURSOR_WIDTH as i32 {
                let screen_x = self.x + sx;
                let saved_idx = (sy as usize) * (CURSOR_WIDTH as usize) + (sx as usize);
                if screen_x >= 0 && (screen_x as u32) < surface.width && screen_y >= 0 && (screen_y as u32) < surface.height {
                    self.saved_under[saved_idx] = surface.get_pixel(screen_x as u32, screen_y as u32).raw;
                } else {
                    self.saved_under[saved_idx] = Color::BLACK.raw;
                }
            }
        }
        self.has_saved = true;

        // 2. Draw sprite overlay
        for sy in 0..CURSOR_HEIGHT as i32 {
            let dy = self.y + sy;
            if dy < 0 || (dy as u32) >= surface.height {
                continue;
            }
            for sx in 0..CURSOR_WIDTH as i32 {
                let dx = self.x + sx;
                if dx < 0 || (dx as u32) >= surface.width {
                    continue;
                }

                let sprite_idx = (sy as usize) * (CURSOR_WIDTH as usize) + (sx as usize);
                match ARROW_SPRITE[sprite_idx] {
                    1 => surface.set_pixel(dx as u32, dy as u32, Color::BLACK),
                    2 => surface.set_pixel(dx as u32, dy as u32, Color::WHITE),
                    _ => {} // Transparent pixel: do not overwrite
                }
            }
        }
    }
}
