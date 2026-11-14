//! NovaOS Apex — 2D Software Compositor & Window Server Core
//! True double-buffered multi-surface compositor with damage tracking and alpha blending.

use super::color::Color;
use super::cursor::Cursor;
use super::damage::DamageRegion;
use super::events::WindowEvent;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;
use super::window::{Window, WindowId};
use crate::drivers::framebuffer::get_driver;
use alloc::vec::Vec;

pub struct Compositor {
    pub width: u32,
    pub height: u32,
    pub backbuffer: Surface,
    pub windows: Vec<Window>,
    pub next_window_id: u32,
    pub active_window: Option<WindowId>,
    pub cursor: Cursor,
    pub damage: DamageRegion,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        let mut backbuffer = Surface::new(width, height);
        init_wallpaper_surface(&mut backbuffer);

        let mut damage = DamageRegion::new();
        damage.add_rect(Rect::new(0, 0, width, height));

        Self {
            width,
            height,
            backbuffer,
            windows: Vec::new(),
            next_window_id: 1,
            active_window: None,
            cursor: Cursor::new(),
            damage,
        }
    }

    pub fn render_wallpaper_rect(&mut self, r: Rect) {
        draw_bloom_wallpaper_to(&mut self.backbuffer, Some(r));
    }

    pub fn create_window(
        &mut self,
        title: &str,
        x: i32,
        y: i32,
        client_w: u32,
        client_h: u32,
        flags: u32,
        owner_pid: u32,
    ) -> WindowId {
        let id = WindowId(self.next_window_id);
        self.next_window_id += 1;

        let mut win = Window::new(id, title, x, y, client_w, client_h, flags, owner_pid);
        win.z_order = self.windows.len();

        let win_rect = win.rect;
        self.windows.push(win);

        self.damage.add_rect(win_rect);
        self.bring_to_front(id);

        id
    }

    pub fn destroy_window(&mut self, id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|w| w.id == id) {
            let rect = self.windows[pos].rect;
            self.damage.add_rect(rect);
            self.windows.remove(pos);

            // Re-normalize Z-orders
            for (idx, w) in self.windows.iter_mut().enumerate() {
                w.z_order = idx;
            }

            if self.active_window == Some(id) {
                self.active_window = self.windows.last().map(|w| w.id);
                if let Some(new_active) = self.active_window {
                    let rect = if let Some(w) = self.get_window_mut(new_active) {
                        w.active = true;
                        w.push_event(WindowEvent::Focus);
                        Some(w.rect)
                    } else {
                        None
                    };
                    if let Some(r) = rect {
                        self.damage.add_rect(r);
                    }
                }
            }
        }
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub fn bring_to_front(&mut self, id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|w| w.id == id) {
            let mut win = self.windows.remove(pos);

            // Deactivate old active window
            if let Some(old_id) = self.active_window {
                if old_id != id {
                    let rect = if let Some(old_win) = self.get_window_mut(old_id) {
                        old_win.active = false;
                        old_win.push_event(WindowEvent::Blur);
                        Some(old_win.rect)
                    } else {
                        None
                    };
                    if let Some(r) = rect {
                        self.damage.add_rect(r);
                    }
                }
            }

            win.active = true;
            win.push_event(WindowEvent::Focus);
            self.damage.add_rect(win.rect);

            self.windows.push(win);
            self.active_window = Some(id);

            // Re-normalize Z-orders
            for (idx, w) in self.windows.iter_mut().enumerate() {
                w.z_order = idx;
            }
        }
    }

    pub fn set_window_position(&mut self, id: WindowId, new_x: i32, new_y: i32) {
        if let Some(win) = self.get_window_mut(id) {
            let old_rect = win.rect;
            win.rect.x = new_x;
            win.rect.y = new_y;
            let new_rect = win.rect;

            self.damage.add_rect(old_rect);
            self.damage.add_rect(new_rect);
        }
    }

    pub fn damage_window(&mut self, id: WindowId) {
        if let Some(win) = self.get_window(id) {
            self.damage.add_rect(win.rect);
        }
    }

    pub fn window_at(&self, x: i32, y: i32) -> Option<WindowId> {
        // Top-down search in descending Z-order
        for win in self.windows.iter().rev() {
            if win.visible && win.rect.contains_coords(x, y) {
                return Some(win.id);
            }
        }
        None
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        let clamped_x = x.clamp(0, self.width.saturating_sub(1) as i32);
        let clamped_y = y.clamp(0, self.height.saturating_sub(1) as i32);

        let old_cursor_rect = self.cursor.current_rect();
        self.cursor.x = clamped_x;
        self.cursor.y = clamped_y;
        let new_cursor_rect = self.cursor.current_rect();

        self.damage.add_rect(old_cursor_rect);
        self.damage.add_rect(new_cursor_rect);

        // Forward mouse move to target window
        if let Some(win_id) = self.window_at(clamped_x, clamped_y) {
            if let Some(win) = self.get_window_mut(win_id) {
                let cr = win.client_rect();
                let client_x = clamped_x - cr.x;
                let client_y = clamped_y - cr.y;
                win.push_event(WindowEvent::MouseMove {
                    x: client_x,
                    y: client_y,
                    buttons: 0,
                });
            }
        }
    }

    pub fn on_mouse_down(&mut self, x: i32, y: i32, button: u8) {
        if let Some(win_id) = self.window_at(x, y) {
            self.bring_to_front(win_id);
            if let Some(win) = self.get_window_mut(win_id) {
                let cr = win.client_rect();
                let client_x = x - cr.x;
                let client_y = y - cr.y;
                win.push_event(WindowEvent::MouseDown {
                    x: client_x,
                    y: client_y,
                    button,
                });
            }
        }
    }

    pub fn on_mouse_up(&mut self, x: i32, y: i32, button: u8) {
        if let Some(win_id) = self.active_window {
            if let Some(win) = self.get_window_mut(win_id) {
                let cr = win.client_rect();
                let client_x = x - cr.x;
                let client_y = y - cr.y;
                win.push_event(WindowEvent::MouseUp {
                    x: client_x,
                    y: client_y,
                    button,
                });
            }
        }
    }

    pub fn on_key_event(&mut self, scancode: u8, key: u8, is_down: bool) {
        if let Some(win_id) = self.active_window {
            if let Some(win) = self.get_window_mut(win_id) {
                if is_down {
                    win.push_event(WindowEvent::KeyDown { scancode, key });
                    if key >= 32 && key <= 126 {
                        win.push_event(WindowEvent::Char(key as char));
                    }
                } else {
                    win.push_event(WindowEvent::KeyUp { scancode, key });
                }
            }
        }
    }

    /// Composite all damaged areas into backbuffer and flush directly to GOP framebuffer hardware
    pub fn composite(&mut self) {
        if self.damage.is_empty() {
            return;
        }

        // Collect damaged bounding box
        let bbox = match self.damage.bounding_box() {
            Some(b) => b,
            None => return,
        };

        // 1. Restore wallpaper for damaged regions
        let rects_copy: Vec<Rect> = self.damage.rects().to_vec();
        for r in rects_copy {
            self.render_wallpaper_rect(r);
        }

        // 2. Render all visible windows in ascending Z-order
        for win in &self.windows {
            if !win.visible {
                continue;
            }
            if win.rect.intersects(&bbox) {
                win.render(&mut self.backbuffer);
            }
        }

        // 3. Render mouse cursor overlay
        self.cursor.render_to(&mut self.backbuffer);

        // 4. Blit damaged regions to hardware GOP framebuffer
        let fb = get_driver();
        if !fb.base_ptr.is_null() {
            for r in self.damage.rects() {
                self.backbuffer.blit_clipped_to_raw(fb.base_ptr, fb.stride, fb.width, fb.height, *r);
            }
        }

        // 5. Clear accumulated damage
        self.damage.clear();
    }
}

/// Initialize modern Windows 11-inspired NovaOS Apex Desktop wallpaper
fn init_wallpaper_surface(surface: &mut Surface) {
    draw_bloom_wallpaper_to(surface, None);
}

/// Renders the Windows 11-class "Nova Bloom" gradient wallpaper, ribbon waves, and desktop icons
fn draw_bloom_wallpaper_to(surface: &mut Surface, clip: Option<Rect>) {
    let w = surface.width as i32;
    let h = surface.height as i32;

    let y_start = clip.map(|c| c.y.max(0)).unwrap_or(0);
    let y_end = clip.map(|c| (c.bottom() + 1).min(h)).unwrap_or(h);
    let x_start = clip.map(|c| c.x.max(0)).unwrap_or(0);
    let x_end = clip.map(|c| (c.right() + 1).min(w)).unwrap_or(w);

    // 1. Vertical Gradient: Deep Obsidian (0x0A0D15) -> Royal Sapphire (0x121E36) -> Deep Navy (0x0E1729)
    for y in (y_start..y_end).step_by(4) {
        let t = (y as u32 * 255) / (h as u32).max(1);
        let (r, g, b) = if t < 128 {
            let factor = t * 2;
            (
                (10 + (8 * factor) / 255) as u8,
                (13 + (17 * factor) / 255) as u8,
                (21 + (33 * factor) / 255) as u8,
            )
        } else {
            let factor = (t - 128) * 2;
            (
                (18 - (4 * factor) / 255) as u8,
                (30 - (7 * factor) / 255) as u8,
                (54 - (13 * factor) / 255) as u8,
            )
        };
        let band_color = Color::from_rgb(r, g, b);
        let band_h = 4.min(y_end - y) as u32;
        surface.fill_rect(Rect::new(x_start, y, (x_end - x_start) as u32, band_h), band_color);
    }

    // 2. Windows 11 "Nova Bloom" Abstract Ribbon Wave (Center-Right)
    let cx = w / 2 + 100;
    let cy = h / 2 - 30;

    // Outer Ribbon (Deep Royal Blue)
    for i in 0..16 {
        let r_h = (200 - i * 6).max(1) as u32;
        let r_rect = Rect::new(cx - 160 + i * 14, cy - 90 + (i * i) / 8, 140, r_h);
        if clip.map_or(true, |c| c.intersects(&r_rect)) {
            let alpha = (45 - i * 2) as u8;
            surface.fill_rect(r_rect, Color::from_argb(alpha, 0x1E, 0x4B, 0x8A));
        }
    }

    // Main Luminous Bloom Petal (Electric Cyan / Azure)
    for i in 0..12 {
        let p_h = (160 - i * 8).max(1) as u32;
        let p_rect = Rect::new(cx - 100 + i * 16, cy - 60 + i * 5, 90, p_h);
        if clip.map_or(true, |c| c.intersects(&p_rect)) {
            let alpha = (60 - i * 3) as u8;
            surface.fill_rect(p_rect, Color::from_argb(alpha, 0x38, 0x8B, 0xFD));
        }
    }

    // Core Highlight Petal (Bright Azure 0x4CC2FF)
    for i in 0..8 {
        let h_h = (110 - i * 8).max(1) as u32;
        let h_rect = Rect::new(cx - 50 + i * 12, cy - 30 + i * 4, 60, h_h);
        if clip.map_or(true, |c| c.intersects(&h_rect)) {
            let alpha = (70 - i * 5) as u8;
            surface.fill_rect(h_rect, Color::from_argb(alpha, 0x78, 0xD5, 0xFF));
        }
    }

    // 3. Desktop Shortcut Icons (Left column)
    struct DesktopIcon {
        name: &'static str,
        sym: &'static str,
        bg: Color,
        x: i32,
        y: i32,
    }
    let icons: [DesktopIcon; 6] = [
        DesktopIcon { name: "Nova Files", sym: "F", bg: Color::from_hex(0x0078D4), x: 24, y: 24 },
        DesktopIcon { name: "Sys Monitor", sym: "M", bg: Color::from_hex(0x107C41), x: 24, y: 96 },
        DesktopIcon { name: "Settings", sym: "S", bg: Color::from_hex(0x737373), x: 24, y: 168 },
        DesktopIcon { name: "Pkg Store", sym: "P", bg: Color::from_hex(0x8764B8), x: 24, y: 240 },
        DesktopIcon { name: "Terminal", sym: ">", bg: Color::from_hex(0x24292E), x: 24, y: 312 },
        DesktopIcon { name: "Device Mgr", sym: "D", bg: Color::from_hex(0xD83B01), x: 24, y: 384 },
    ];

    for icon in &icons {
        let icon_tile = Rect::new(icon.x, icon.y, 44, 40);
        let icon_total = Rect::new(icon.x - 10, icon.y, 80, 64);
        if clip.map_or(true, |c| c.intersects(&icon_total)) {
            // Icon Card Background
            surface.fill_rect(icon_tile, Color::from_argb(60, 0x1E, 0x24, 0x30));
            surface.draw_rect_outline(icon_tile, Color::from_argb(80, 0x48, 0x54, 0x68), 1);

            // Icon Badge
            let badge_rect = Rect::new(icon.x + 10, icon.y + 6, 24, 20);
            surface.fill_rect(badge_rect, icon.bg);
            draw_text(surface, icon.sym, badge_rect.x + 8, badge_rect.y + 2, Color::TEXT_PRIMARY);

            // Text Label with dark drop shadow for readability
            draw_text(surface, icon.name, icon.x - 6, icon.y + 45, Color::from_hex(0x000000));
            draw_text(surface, icon.name, icon.x - 7, icon.y + 44, Color::TEXT_PRIMARY);
        }
    }

    // 4. Discreet Windows 11-style Desktop Watermark (Bottom-Right)
    let wm_rect = Rect::new(w - 280, h - 80, 270, 24);
    if clip.map_or(true, |c| c.intersects(&wm_rect)) {
        draw_text(surface, "NovaOS Apex v1.0.0 (x86_64)", w - 260, h - 76, Color::from_hex(0x485468));
    }
}
