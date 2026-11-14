//! NovaOS Apex — Windows 11 Quick Settings & Action Center
//! Wi-Fi, Bluetooth, Airplane mode, Night light, Battery saver, Volume and Brightness controls.

use super::color::Color;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;

pub const QUICK_SETTINGS_WIDTH: u32 = 340;
pub const QUICK_SETTINGS_HEIGHT: u32 = 280;

pub struct QuickSettings {
    pub visible: bool,
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_mode: bool,
    pub night_light: bool,
    pub battery_saver: bool,
    pub volume_level: u8,      // 0 - 100
    pub brightness_level: u8,  // 0 - 100
    pub battery_percentage: u8,
}

impl QuickSettings {
    pub const fn new() -> Self {
        Self {
            visible: false,
            wifi_enabled: true,
            bluetooth_enabled: false,
            airplane_mode: false,
            night_light: false,
            battery_saver: false,
            volume_level: 80,
            brightness_level: 100,
            battery_percentage: 95,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn rect(&self, screen_w: u32, screen_h: u32, taskbar_h: u32) -> Rect {
        let x = (screen_w.saturating_sub(QUICK_SETTINGS_WIDTH + 12)) as i32;
        let y = (screen_h.saturating_sub(taskbar_h + QUICK_SETTINGS_HEIGHT + 12)) as i32;
        Rect::new(x, y, QUICK_SETTINGS_WIDTH, QUICK_SETTINGS_HEIGHT)
    }

    pub fn render(&self, target: &mut Surface, screen_w: u32, screen_h: u32, taskbar_h: u32) {
        if !self.visible {
            return;
        }

        let qrect = self.rect(screen_w, screen_h, taskbar_h);

        // 1. Drop shadow
        let shadow = Rect::new(qrect.x + 4, qrect.y + 4, qrect.width, qrect.height);
        target.fill_rect(shadow, Color::from_argb(80, 0, 0, 0));

        // 2. Acrylic Mica glass panel
        target.fill_rect(qrect, Color::NOVA_SURFACE);
        target.draw_rect_outline(qrect, Color::NOVA_BORDER, 1);

        // 3. Quick Action Tiles (2 rows x 3 columns)
        let tiles: [(&'static str, bool); 6] = [
            ("Wi-Fi", self.wifi_enabled),
            ("Bluetooth", self.bluetooth_enabled),
            ("Airplane", self.airplane_mode),
            ("Night Light", self.night_light),
            ("Battery Saver", self.battery_saver),
            ("Hotspot", false),
        ];

        let tile_w = 96;
        let tile_h = 44;
        let gap_x = 8;
        let gap_y = 8;
        let start_x = qrect.x + 16;
        let start_y = qrect.y + 16;

        for (i, (label, active)) in tiles.iter().enumerate() {
            let col = (i % 3) as i32;
            let row = (i / 3) as i32;
            let tx = start_x + col * (tile_w as i32 + gap_x);
            let ty = start_y + row * (tile_h as i32 + gap_y);
            let t_rect = Rect::new(tx, ty, tile_w, tile_h);

            let bg = if *active { Color::from_hex(0x1F6FEB) } else { Color::NOVA_CARD };
            let border = if *active { Color::NOVA_ACCENT } else { Color::NOVA_BORDER };
            let fg = if *active { Color::WHITE } else { Color::TEXT_PRIMARY };

            target.fill_rect(t_rect, bg);
            target.draw_rect_outline(t_rect, border, 1);
            draw_text(target, label, tx + 10, ty + 16, fg);
        }

        // 4. Sliders (Volume and Brightness)
        let slider_y1 = start_y + 2 * (tile_h as i32 + gap_y) + 16;
        draw_text(target, "Volume", qrect.x + 16, slider_y1, Color::TEXT_SECONDARY);

        let s1_track = Rect::new(qrect.x + 80, slider_y1 + 4, qrect.width - 100, 6);
        target.fill_rect(s1_track, Color::from_hex(0x21262D));
        let s1_fill = Rect::new(qrect.x + 80, slider_y1 + 4, ((qrect.width - 100) * self.volume_level as u32) / 100, 6);
        target.fill_rect(s1_fill, Color::NOVA_ACCENT);

        let slider_y2 = slider_y1 + 30;
        draw_text(target, "Display", qrect.x + 16, slider_y2, Color::TEXT_SECONDARY);

        let s2_track = Rect::new(qrect.x + 80, slider_y2 + 4, qrect.width - 100, 6);
        target.fill_rect(s2_track, Color::from_hex(0x21262D));
        let s2_fill = Rect::new(qrect.x + 80, slider_y2 + 4, ((qrect.width - 100) * self.brightness_level as u32) / 100, 6);
        target.fill_rect(s2_fill, Color::from_hex(0x388BFD));

        // 5. Bottom Footer (Battery Status + Settings gear)
        let footer_y = qrect.bottom() - 36;
        target.draw_line(qrect.x, footer_y, qrect.right(), footer_y, Color::NOVA_BORDER);
        draw_text(target, "Battery: 95% (AC Connected)", qrect.x + 16, footer_y + 10, Color::SUCCESS_GREEN);
        draw_text(target, "[Settings]", qrect.right() - 88, footer_y + 10, Color::TEXT_SECONDARY);
    }
}
