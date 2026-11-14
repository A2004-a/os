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
        // 1. Midnight base fill
        self.backbuffer.fill_rect(r, Color::NOVA_MIDNIGHT);

        // 2. Subtle horizontal mesh lines
        let step = 40;
        let y_start = (r.y / step) * step;
        for y in (y_start..=r.bottom()).step_by(step as usize) {
            if y >= 0 && (y as u32) < self.height {
                self.backbuffer.draw_line(r.x, y, r.right(), y, Color::from_hex(0x111722));
            }
        }
        // 3. Subtle vertical mesh lines
        let x_start = (r.x / step) * step;
        for x in (x_start..=r.right()).step_by(step as usize) {
            if x >= 0 && (x as u32) < self.width {
                self.backbuffer.draw_line(x, r.y, x, r.bottom(), Color::from_hex(0x111722));
            }
        }

        // 4. Center NovaOS Apex Emblem & Brand Banner
        let cx = self.width as i32 / 2;
        let cy = self.height as i32 / 2;
        let badge_rect = Rect::new(cx - 120, cy - 50, 240, 100);
        if r.intersects(&badge_rect) {
            self.backbuffer.fill_rect(badge_rect, Color::NOVA_SURFACE);
            self.backbuffer.draw_rect_outline(badge_rect, Color::NOVA_ACCENT, 2);
            draw_text(&mut self.backbuffer, "NovaOS Apex", cx - 44, cy - 20, Color::NOVA_ACCENT);
            draw_text(&mut self.backbuffer, "Milestone 7: NovaUI & Window Server", cx - 110, cy + 8, Color::TEXT_SECONDARY);
        }
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
    // 1. Deep midnight gradient base
    surface.clear(Color::NOVA_MIDNIGHT);

    // 2. Subtle horizontal mesh lines
    for y in (0..surface.height).step_by(40) {
        surface.draw_line(0, y as i32, surface.width as i32, y as i32, Color::from_hex(0x111722));
    }
    // 3. Subtle vertical mesh lines
    for x in (0..surface.width).step_by(40) {
        surface.draw_line(x as i32, 0, x as i32, surface.height as i32, Color::from_hex(0x111722));
    }

    // 4. Center NovaOS Apex Emblem & Brand Banner
    let cx = surface.width as i32 / 2;
    let cy = surface.height as i32 / 2;

    // Glowing cyan geometric badge
    let badge_rect = Rect::new(cx - 120, cy - 50, 240, 100);
    surface.fill_rect(badge_rect, Color::NOVA_SURFACE);
    surface.draw_rect_outline(badge_rect, Color::NOVA_ACCENT, 2);

    draw_text(surface, "NovaOS Apex", cx - 44, cy - 20, Color::NOVA_ACCENT);
    draw_text(surface, "Milestone 7: NovaUI & Window Server", cx - 110, cy + 8, Color::TEXT_SECONDARY);
}
