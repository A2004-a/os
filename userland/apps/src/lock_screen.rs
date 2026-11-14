//! NovaOS Apex — Lock Screen Native Application
//! Windows 11-class lock screen with clock, user avatar, and PIN/password unlock.

#![allow(dead_code)]

use novaos_apex_toolkit::*;

pub struct NovaLockScreenApp {
    pub bounds: Rect,
    pub is_locked: bool,
    pub pin_entered: [u8; 8],
    pub pin_len: usize,
    pub unlock_button: Button,
}

impl NovaLockScreenApp {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let btn_w = 120;
        let btn_h = 32;
        let bx = x + (width as i32 - btn_w as i32) / 2;
        let by = y + (height as i32) - 80;

        let mut app = Self {
            bounds: Rect::new(x, y, width, height),
            is_locked: true,
            pin_entered: [0u8; 8],
            pin_len: 0,
            unlock_button: Button::new(bx, by, btn_w, btn_h, "Sign In"),
        };
        app.pin_len = 4; // simulated pin
        app
    }

    pub fn render(&self, buf: &mut [u32], stride: u32) {
        // 1. Dark Acrylic Mica background
        fill_rect(buf, stride, self.bounds, Color::from_hex(0x0A0E14));

        // 2. Large Centered Time Display (HH:MM)
        let time_str = "12:00";
        let date_str = "Friday, November 14, 2026";
        let cx = self.bounds.x + (self.bounds.width as i32) / 2;
        let ty = self.bounds.y + 40;

        draw_text(buf, stride, time_str, cx - 20, ty, Color::WHITE);
        draw_text(buf, stride, date_str, cx - 90, ty + 24, Color::TEXT_SECONDARY);

        // 3. User Avatar Circle/Card
        let avatar_y = ty + 70;
        let avatar_rect = Rect::new(cx - 24, avatar_y, 48, 48);
        fill_rect(buf, stride, avatar_rect, Color::from_hex(0x1F6FEB));
        draw_rect_outline(buf, stride, avatar_rect, Color::NOVA_ACCENT, 2);
        draw_text(buf, stride, "APEX", cx - 16, avatar_y + 18, Color::WHITE);

        // User Name
        draw_text(buf, stride, "Apex Administrator", cx - 64, avatar_y + 56, Color::TEXT_PRIMARY);

        // 4. PIN Dots Input Field
        let pin_rect = Rect::new(cx - 60, avatar_y + 80, 120, 28);
        fill_rect(buf, stride, pin_rect, Color::NOVA_CARD);
        draw_rect_outline(buf, stride, pin_rect, Color::NOVA_BORDER, 1);

        // Render entered PIN dots
        let dots_str = "****";
        draw_text(buf, stride, dots_str, cx - 16, avatar_y + 88, Color::NOVA_ACCENT);

        // 5. Sign In / Unlock Button
        self.unlock_button.render(buf, stride);
    }
}
