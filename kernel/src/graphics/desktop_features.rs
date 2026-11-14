//! NovaOS Apex — Desktop Features & Shell Utilities
//! Marquee selection, Desktop Icon Grid, System Clipboard, Wallpaper Themes,
//! High Contrast Accessibility, and Aero Shake window minimization.

use super::color::Color;
use super::geometry::{Point, Rect};
use super::surface::Surface;
use super::window::{Window, WindowId, WindowState};
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallpaperTheme {
    NovaBloom,
    DarkMica,
    SlateMinimal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopIcon {
    pub label: &'static str,
    pub x: i32,
    pub y: i32,
    pub selected: bool,
}

pub struct DesktopFeatures {
    pub marquee_start: Option<Point>,
    pub marquee_current: Option<Point>,
    pub clipboard_buffer: [u8; 1024],
    pub clipboard_len: usize,
    pub current_theme: WallpaperTheme,
    pub high_contrast: bool,
    pub icons: [DesktopIcon; 4],
}

impl DesktopFeatures {
    pub const fn new() -> Self {
        Self {
            marquee_start: None,
            marquee_current: None,
            clipboard_buffer: [0u8; 1024],
            clipboard_len: 0,
            current_theme: WallpaperTheme::NovaBloom,
            high_contrast: false,
            icons: [
                DesktopIcon { label: "This PC", x: 24, y: 24, selected: false },
                DesktopIcon { label: "Files", x: 24, y: 104, selected: false },
                DesktopIcon { label: "Terminal", x: 24, y: 184, selected: false },
                DesktopIcon { label: "Trash", x: 24, y: 264, selected: false },
            ],
        }
    }

    pub fn start_marquee(&mut self, p: Point) {
        self.marquee_start = Some(p);
        self.marquee_current = Some(p);
    }

    pub fn update_marquee(&mut self, p: Point) {
        if self.marquee_start.is_some() {
            self.marquee_current = Some(p);
        }
    }

    pub fn end_marquee(&mut self) -> Option<Rect> {
        let r = self.marquee_rect();
        self.marquee_start = None;
        self.marquee_current = None;
        r
    }

    pub fn marquee_rect(&self) -> Option<Rect> {
        if let (Some(s), Some(c)) = (self.marquee_start, self.marquee_current) {
            let x = s.x.min(c.x);
            let y = s.y.min(c.y);
            let w = (s.x - c.x).abs() as u32;
            let h = (s.y - c.y).abs() as u32;
            if w > 2 && h > 2 {
                return Some(Rect::new(x, y, w, h));
            }
        }
        None
    }

    pub fn copy_to_clipboard(&mut self, text: &[u8]) {
        let len = text.len().min(1024);
        self.clipboard_buffer[..len].copy_from_slice(&text[..len]);
        self.clipboard_len = len;
    }

    pub fn read_clipboard(&self) -> &[u8] {
        &self.clipboard_buffer[..self.clipboard_len]
    }

    pub fn aero_shake(&self, shaken_window: WindowId, windows: &mut [Window]) -> Vec<WindowId> {
        let mut minimized = Vec::new();
        for w in windows.iter_mut() {
            if w.id != shaken_window && w.visible && w.state != WindowState::Minimized {
                w.state = WindowState::Minimized;
                w.visible = false;
                minimized.push(w.id);
            }
        }
        minimized
    }

    pub fn render_marquee(&self, target: &mut Surface) {
        if let Some(r) = self.marquee_rect() {
            // Translucent accent blue fill
            let fill_color = Color::from_argb(40, 0, 160, 255);
            target.fill_rect(r, fill_color);
            // 1-px accent outline
            target.draw_rect_outline(r, Color::from_hex(0x388BFD), 1);
        }
    }
}
