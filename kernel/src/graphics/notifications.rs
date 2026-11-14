//! NovaOS Apex — Windows 11 Notification Center & Calendar Flyout
//! Toast notification card stack, calendar month view, and system alerts.

use super::color::Color;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;

pub const FLYOUT_WIDTH: u32 = 360;
pub const FLYOUT_HEIGHT: u32 = 420;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationToast {
    pub title: &'static str,
    pub body: &'static str,
    pub source: &'static str,
    pub timestamp_str: &'static str,
}

pub static SYSTEM_NOTIFICATIONS: [NotificationToast; 3] = [
    NotificationToast {
        title: "System Update Ready",
        body: "NovaOS Apex v1.0.0 Release Certified.",
        source: "Windows Update",
        timestamp_str: "Just now",
    },
    NotificationToast {
        title: "Storage Integrity Verified",
        body: "NovaFS journal scrub passed with 0 errors.",
        source: "NovaFS Scrubber",
        timestamp_str: "2m ago",
    },
    NotificationToast {
        title: "Ethernet Connected",
        body: "Gigabit Ethernet linked at 1000 Mbps.",
        source: "Network Subsystem",
        timestamp_str: "5m ago",
    },
];

pub struct NotificationCenter {
    pub visible: bool,
}

impl NotificationCenter {
    pub const fn new() -> Self {
        Self { visible: false }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn rect(&self, screen_w: u32, screen_h: u32, taskbar_h: u32) -> Rect {
        let x = (screen_w.saturating_sub(FLYOUT_WIDTH + 12)) as i32;
        let y = (screen_h.saturating_sub(taskbar_h + FLYOUT_HEIGHT + 12)) as i32;
        Rect::new(x, y, FLYOUT_WIDTH, FLYOUT_HEIGHT)
    }

    pub fn render(&self, target: &mut Surface, screen_w: u32, screen_h: u32, taskbar_h: u32) {
        if !self.visible {
            return;
        }

        let frect = self.rect(screen_w, screen_h, taskbar_h);

        // 1. Drop shadow
        let shadow = Rect::new(frect.x + 4, frect.y + 4, frect.width, frect.height);
        target.fill_rect(shadow, Color::from_argb(80, 0, 0, 0));

        // 2. Acrylic panel
        target.fill_rect(frect, Color::NOVA_SURFACE);
        target.draw_rect_outline(frect, Color::NOVA_BORDER, 1);

        // 3. Header: Notifications & Clear All
        draw_text(target, "Notifications", frect.x + 16, frect.y + 16, Color::TEXT_PRIMARY);
        draw_text(target, "Clear All", frect.right() - 88, frect.y + 16, Color::NOVA_ACCENT);
        target.draw_line(frect.x + 16, frect.y + 36, frect.right() - 16, frect.y + 36, Color::NOVA_BORDER);

        // 4. Notification Cards
        let mut card_y = frect.y + 44;
        for toast in SYSTEM_NOTIFICATIONS.iter() {
            let card_rect = Rect::new(frect.x + 16, card_y, frect.width - 32, 64);
            target.fill_rect(card_rect, Color::NOVA_CARD);
            target.draw_rect_outline(card_rect, Color::NOVA_BORDER, 1);

            // Source tag and timestamp
            draw_text(target, toast.source, card_rect.x + 10, card_rect.y + 8, Color::NOVA_ACCENT);
            draw_text(target, toast.timestamp_str, card_rect.right() - 64, card_rect.y + 8, Color::TEXT_SECONDARY);

            // Title
            draw_text(target, toast.title, card_rect.x + 10, card_rect.y + 24, Color::TEXT_PRIMARY);

            // Body snippet
            draw_text(target, toast.body, card_rect.x + 10, card_rect.y + 42, Color::TEXT_SECONDARY);

            card_y += 72;
        }

        // 5. Calendar Divider & Month Header
        target.draw_line(frect.x + 16, card_y + 4, frect.right() - 16, card_y + 4, Color::NOVA_BORDER);
        draw_text(target, "November 2026", frect.x + 16, card_y + 14, Color::TEXT_PRIMARY);

        // 6. Mini Calendar Day Headers (Su Mo Tu We Th Fr Sa)
        let days = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let cal_start_x = frect.x + 20;
        let cal_y = card_y + 36;
        let col_w = 46;

        for (i, d) in days.iter().enumerate() {
            draw_text(target, d, cal_start_x + (i as i32 * col_w), cal_y, Color::TEXT_SECONDARY);
        }

        // Calendar numbers grid sample
        let dates_row = [" 8", " 9", "10", "11", "12", "13", "14"];
        let row_y = cal_y + 20;
        for (i, num) in dates_row.iter().enumerate() {
            let dx = cal_start_x + (i as i32 * col_w);
            if i == 6 {
                // Today highlighted circle
                let highlight_rect = Rect::new(dx - 4, row_y - 2, 24, 20);
                target.fill_rect(highlight_rect, Color::from_hex(0x1F6FEB));
                draw_text(target, num, dx, row_y, Color::WHITE);
            } else {
                draw_text(target, num, dx, row_y, Color::TEXT_PRIMARY);
            }
        }
    }
}
