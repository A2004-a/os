//! NovaOS Apex — Windows 11 Task Switcher (Alt+Tab Surface)
//! Visual app thumbnail cards, active window selection, and backdrop blur shield.

use super::color::Color;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;
use super::window::{Window, WindowId};

pub struct TaskSwitcher {
    pub visible: bool,
    pub selected_index: usize,
}

impl TaskSwitcher {
    pub const fn new() -> Self {
        Self {
            visible: false,
            selected_index: 0,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.selected_index = 0;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn next(&mut self, total_windows: usize) {
        if total_windows > 0 {
            self.selected_index = (self.selected_index + 1) % total_windows;
        }
    }

    pub fn prev(&mut self, total_windows: usize) {
        if total_windows > 0 {
            if self.selected_index == 0 {
                self.selected_index = total_windows - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected_window_id<'a>(&self, windows: &'a [Window]) -> Option<WindowId> {
        windows.get(self.selected_index).map(|w| w.id)
    }

    pub fn render(&self, target: &mut Surface, screen_w: u32, screen_h: u32, windows: &[Window]) {
        if !self.visible || windows.is_empty() {
            return;
        }

        // 1. Semi-translucent dark backdrop shield
        let screen_rect = Rect::new(0, 0, screen_w, screen_h);
        target.fill_rect(screen_rect, Color::from_argb(90, 0, 0, 0));

        // 2. Switcher Island Bounds
        let card_w = 160;
        let card_h = 120;
        let gap = 16;
        let count = windows.len().min(6);
        let total_w = (count as u32 * card_w) + ((count as u32 - 1) * gap as u32) + 32;
        let total_h = card_h + 32;

        let island_x = (screen_w.saturating_sub(total_w) / 2) as i32;
        let island_y = (screen_h.saturating_sub(total_h) / 2) as i32;
        let island_rect = Rect::new(island_x, island_y, total_w, total_h);

        // Island acrylic panel
        target.fill_rect(island_rect, Color::NOVA_SURFACE);
        target.draw_rect_outline(island_rect, Color::NOVA_BORDER, 1);

        // 3. Render Cards
        let mut cx = island_x + 16;
        let cy = island_y + 16;

        for (i, win) in windows.iter().take(count).enumerate() {
            let card_rect = Rect::new(cx, cy, card_w, card_h);
            let is_sel = i == self.selected_index;

            let bg = if is_sel { Color::from_hex(0x28303E) } else { Color::NOVA_CARD };
            let border = if is_sel { Color::NOVA_ACCENT } else { Color::NOVA_BORDER };

            target.fill_rect(card_rect, bg);
            target.draw_rect_outline(card_rect, border, if is_sel { 2 } else { 1 });

            // Card titlebar snippet
            let tb_rect = Rect::new(cx + 4, cy + 4, card_w - 8, 20);
            target.fill_rect(tb_rect, Color::NOVA_SURFACE);
            let title_bytes = win.title.as_bytes();
            let copy_len = title_bytes.len().min(14);
            let mut snippet = [b' '; 14];
            snippet[..copy_len].copy_from_slice(&title_bytes[..copy_len]);
            let label = core::str::from_utf8(&snippet[..copy_len]).unwrap_or("App");
            draw_text(target, label, cx + 8, cy + 8, Color::TEXT_PRIMARY);

            // Thumbnail preview area
            let thumb_rect = Rect::new(cx + 8, cy + 28, card_w - 16, card_h - 36);
            target.fill_rect(thumb_rect, Color::from_hex(0x13171F));
            draw_text(target, "[Window Preview]", cx + 16, cy + 56, Color::TEXT_SECONDARY);

            cx += card_w as i32 + gap;
        }
    }
}
