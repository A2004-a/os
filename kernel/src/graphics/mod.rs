//! NovaOS Zero — Graphical Subsystem (Compositor, Window Server, NovaUI, Font)

pub mod compositor;
pub mod desktop_shell;
pub mod font;
pub mod graphics_test;
pub mod novaui;
pub mod shell_test;
pub mod window_server;

pub use compositor::{Color, Rect, Surface};
pub use desktop_shell::{
    DesktopShell, Notification, NotificationLevel, SnapState, MAX_NOTIFICATIONS, MAX_WORKSPACES,
    TASKBAR_HEIGHT,
};
pub use novaui::{Button, Label, Panel, ProgressBar, WidgetState};
pub use window_server::{Window, WindowServer};

use crate::drivers::framebuffer::FRAMEBUFFER;

pub fn draw_rect(x: usize, y: usize, w: usize, h: usize, r: u8, g: u8, b: u8) {
    if let Some(ref mut fb) = *FRAMEBUFFER.lock() {
        for dy in 0..h {
            for dx in 0..w {
                fb.draw_pixel(x + dx, y + dy, r, g, b);
            }
        }
    }
}

pub fn draw_char(x: usize, y: usize, c: char, r: u8, g: u8, b: u8) {
    let glyph = font::get_glyph(c);
    if let Some(ref mut fb) = *FRAMEBUFFER.lock() {
        for row in 0..font::FONT_HEIGHT {
            let row_byte = glyph[row];
            for col in 0..font::FONT_WIDTH {
                if (row_byte & (0x80 >> col)) != 0 {
                    fb.draw_pixel(x + col, y + row, r, g, b);
                }
            }
        }
    }
}

pub fn draw_string(x: usize, y: usize, s: &str, r: u8, g: u8, b: u8) {
    let mut current_x = x;
    for c in s.chars() {
        draw_char(current_x, y, c, r, g, b);
        current_x += font::FONT_WIDTH;
    }
}
