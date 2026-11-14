//! NovaOS Apex — Nova Bar (Taskbar & System Status Island)
//! Centered modern app launcher, taskbar pills, workspace switcher, and system tray.

use super::color::Color;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;
use super::window::{Window, WindowId, WindowState};

pub const NOVA_BAR_HEIGHT: u32 = 48;
pub const START_BUTTON_WIDTH: u32 = 36;
pub const START_BUTTON_HEIGHT: u32 = 36;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarAction {
    ToggleLauncher,
    SwitchWorkspace(u8),
    ActivateWindow(WindowId),
    TrayClick,
}

pub struct NovaBar {
    pub height: u32,
    pub is_launcher_open: bool,
}

impl NovaBar {
    pub const fn new() -> Self {
        Self {
            height: NOVA_BAR_HEIGHT,
            is_launcher_open: false,
        }
    }

    #[inline(always)]
    pub fn rect(&self, screen_w: u32, screen_h: u32) -> Rect {
        let y = (screen_h.saturating_sub(self.height)) as i32;
        Rect::new(0, y, screen_w, self.height)
    }

    pub fn start_button_rect(&self, screen_h: u32) -> Rect {
        let bar_y = (screen_h.saturating_sub(self.height)) as i32;
        Rect::new(12, bar_y + 6, START_BUTTON_WIDTH, START_BUTTON_HEIGHT)
    }

    pub fn workspace_button_rect(&self, screen_h: u32, ws_index: u8) -> Rect {
        let bar_y = (screen_h.saturating_sub(self.height)) as i32;
        let x = 156 + ((ws_index.saturating_sub(1)) as i32 * 30);
        Rect::new(x, bar_y + 10, 24, 28)
    }

    pub fn render(
        &self,
        target: &mut Surface,
        screen_w: u32,
        screen_h: u32,
        windows: &[Window],
        active_win: Option<WindowId>,
        active_ws: u8,
        uptime_ms: u64,
    ) {
        let bar_rect = self.rect(screen_w, screen_h);

        // 1. Taskbar backdrop (Mica / acrylic dark glass)
        target.fill_rect(bar_rect, Color::NOVA_SURFACE);
        target.draw_line(0, bar_rect.y, screen_w as i32, bar_rect.y, Color::NOVA_BORDER);

        // 2. Start / Nova Launcher Button (Windows 11-style 4-tile logo)
        let sbtn = self.start_button_rect(screen_h);
        let sbtn_bg = if self.is_launcher_open { Color::from_hex(0x2D333B) } else { Color::from_hex(0x21262D) };
        target.fill_rect(sbtn, sbtn_bg);
        target.draw_rect_outline(sbtn, Color::from_hex(0x30363D), 1);

        // 4-tile Windows 11 / Nova geometric logo
        let tile_size = 7;
        let gap = 3;
        let t_x = sbtn.x + 9;
        let t_y = sbtn.y + 9;
        let logo_color = if self.is_launcher_open { Color::NOVA_ACCENT } else { Color::from_hex(0x388BFD) };
        target.fill_rect(Rect::new(t_x, t_y, tile_size, tile_size), logo_color);
        target.fill_rect(Rect::new(t_x + tile_size as i32 + gap, t_y, tile_size, tile_size), logo_color);
        target.fill_rect(Rect::new(t_x, t_y + tile_size as i32 + gap, tile_size, tile_size), logo_color);
        target.fill_rect(Rect::new(t_x + tile_size as i32 + gap, t_y + tile_size as i32 + gap, tile_size, tile_size), logo_color);

        // 2b. Windows 11 Quick Search Pill
        let search_pill = Rect::new(sbtn.right() + 8, bar_rect.y + 8, 88, 32);
        target.fill_rect(search_pill, Color::from_hex(0x21262D));
        target.draw_rect_outline(search_pill, Color::from_hex(0x30363D), 1);
        draw_text(target, "Search", search_pill.x + 18, search_pill.y + 8, Color::TEXT_SECONDARY);

        // 3. Workspace Switcher Buttons (1..4)
        for ws in 1..=4 {
            let ws_rect = self.workspace_button_rect(screen_h, ws);
            let is_active = ws == active_ws;
            let bg = if is_active { Color::from_hex(0x283244) } else { Color::NOVA_SURFACE };
            target.fill_rect(ws_rect, bg);

            if is_active {
                target.draw_line(ws_rect.x + 4, ws_rect.bottom() - 2, ws_rect.right() - 4, ws_rect.bottom() - 2, Color::NOVA_ACCENT);
            }

            let ch = (b'0' + ws) as char;
            let mut buf = [0u8; 1];
            buf[0] = ch as u8;
            let label = core::str::from_utf8(&buf).unwrap_or("1");
            let text_color = if is_active { Color::NOVA_ACCENT } else { Color::TEXT_SECONDARY };
            draw_text(target, label, ws_rect.x + 8, ws_rect.y + 6, text_color);
        }

        // 4. Centered Application Island (Running Apps)
        let app_button_w = 124;
        let app_button_h = 32;
        let total_apps_w = (windows.len() as i32) * (app_button_w + 8);
        let mut app_x = (screen_w as i32 - total_apps_w) / 2;
        if app_x < 240 {
            app_x = 240; // clamp past start / workspaces
        }

        let app_y = bar_rect.y + 8;
        for win in windows {
            let is_active = Some(win.id) == active_win;
            let app_rect = Rect::new(app_x, app_y, app_button_w as u32, app_button_h as u32);

            let bg = if is_active { Color::from_hex(0x28303E) } else { Color::NOVA_CARD };
            target.fill_rect(app_rect, bg);
            target.draw_rect_outline(app_rect, Color::NOVA_BORDER, 1);

            // Active indicator pill
            if is_active {
                target.draw_line(app_x + 10, app_rect.bottom() - 2, app_x + app_button_w - 10, app_rect.bottom() - 2, Color::NOVA_ACCENT);
            } else if win.state == WindowState::Minimized {
                // Subtle dot for minimized
                target.fill_rect(Rect::new(app_x + app_button_w / 2 - 2, app_rect.bottom() - 3, 4, 2), Color::TEXT_SECONDARY);
            }

            // App icon square badge (14x14)
            let icon_col = match win.title.as_str() {
                "Nova Files" => Color::from_hex(0x388BFD),
                "Terminal" | "Nova Terminal" => Color::from_hex(0x58A6FF),
                "System Monitor" | "Nova System Monitor" => Color::from_hex(0x2EA043),
                "Settings" | "Nova Settings" => Color::from_hex(0x8B949E),
                "Calculator" => Color::from_hex(0xA371F7),
                "Notepad" => Color::from_hex(0xD29922),
                _ => Color::NOVA_ACCENT,
            };
            let icon_rect = Rect::new(app_x + 8, app_y + 9, 14, 14);
            target.fill_rect(icon_rect, icon_col);

            // Window title snippet (max 12 chars)
            let mut snippet = [b' '; 12];
            let bytes = win.title.as_bytes();
            let copy_len = bytes.len().min(12);
            snippet[..copy_len].copy_from_slice(&bytes[..copy_len]);
            let label = core::str::from_utf8(&snippet[..copy_len]).unwrap_or("App");
            let text_col = if is_active { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };
            draw_text(target, label, app_x + 28, app_y + 8, text_col);

            app_x += app_button_w + 8;
        }

        // 5. System Status Island (Clock, Date, Network, Audio, Power)
        let clock_w = 70;
        let clock_x = screen_w as i32 - clock_w - 20;
        let clock_y = bar_rect.y + 10;

        // Format uptime into HH:MM:SS
        let total_seconds = uptime_ms / 1000;
        let hours = (total_seconds / 3600) % 24;
        let minutes = (total_seconds / 60) % 60;
        let seconds = total_seconds % 60;

        let mut time_buf = [b'0'; 8];
        time_buf[0] = b'0' + (hours / 10) as u8;
        time_buf[1] = b'0' + (hours % 10) as u8;
        time_buf[2] = b':';
        time_buf[3] = b'0' + (minutes / 10) as u8;
        time_buf[4] = b'0' + (minutes % 10) as u8;
        time_buf[5] = b':';
        time_buf[6] = b'0' + (seconds / 10) as u8;
        time_buf[7] = b'0' + (seconds % 10) as u8;
        let time_str = core::str::from_utf8(&time_buf).unwrap_or("00:00:00");

        draw_text(target, time_str, clock_x, clock_y, Color::TEXT_PRIMARY);
        draw_text(target, "Nov 14", clock_x + 6, clock_y + 16, Color::TEXT_SECONDARY);

        // Network status indicator pill: [ETH]
        let net_x = clock_x - 48;
        target.fill_rect(Rect::new(net_x, bar_rect.y + 12, 40, 24), Color::NOVA_CARD);
        target.draw_rect_outline(Rect::new(net_x, bar_rect.y + 12, 40, 24), Color::from_hex(0x30363D), 1);
        draw_text(target, "ETH", net_x + 8, bar_rect.y + 16, Color::SUCCESS_GREEN);

        // Audio Volume pill: [VOL]
        let vol_x = net_x - 48;
        target.fill_rect(Rect::new(vol_x, bar_rect.y + 12, 42, 24), Color::NOVA_CARD);
        target.draw_rect_outline(Rect::new(vol_x, bar_rect.y + 12, 42, 24), Color::from_hex(0x30363D), 1);
        draw_text(target, "VOL", vol_x + 8, bar_rect.y + 16, Color::from_hex(0x388BFD));

        // Power status indicator pill: [AC]
        let pwr_x = vol_x - 38;
        target.fill_rect(Rect::new(pwr_x, bar_rect.y + 12, 32, 24), Color::NOVA_CARD);
        target.draw_rect_outline(Rect::new(pwr_x, bar_rect.y + 12, 32, 24), Color::from_hex(0x30363D), 1);
        draw_text(target, "AC", pwr_x + 8, bar_rect.y + 16, Color::TEXT_SECONDARY);

        // Windows 11 Desktop Peek Line (far right)
        target.draw_line(screen_w as i32 - 4, bar_rect.y + 8, screen_w as i32 - 4, bar_rect.bottom() - 8, Color::from_hex(0x404854));
    }

    pub fn hit_test(
        &self,
        x: i32,
        y: i32,
        screen_w: u32,
        screen_h: u32,
        windows: &[Window],
    ) -> Option<BarAction> {
        let bar_rect = self.rect(screen_w, screen_h);
        if !bar_rect.contains_coords(x, y) {
            return None;
        }

        // 1. Check start button
        let sbtn = self.start_button_rect(screen_h);
        if sbtn.contains_coords(x, y) {
            return Some(BarAction::ToggleLauncher);
        }

        // 2. Check workspace buttons (1..4)
        for ws in 1..=4 {
            let ws_rect = self.workspace_button_rect(screen_h, ws);
            if ws_rect.contains_coords(x, y) {
                return Some(BarAction::SwitchWorkspace(ws));
            }
        }

        // 3. Check running application pills
        let app_button_w = 110;
        let app_button_h = 32;
        let total_apps_w = (windows.len() as i32) * (app_button_w + 8);
        let mut app_x = (screen_w as i32 - total_apps_w) / 2;
        if app_x < 240 {
            app_x = 240;
        }
        let app_y = bar_rect.y + 8;

        for win in windows {
            let app_rect = Rect::new(app_x, app_y, app_button_w as u32, app_button_h as u32);
            if app_rect.contains_coords(x, y) {
                return Some(BarAction::ActivateWindow(win.id));
            }
            app_x += app_button_w + 8;
        }

        Some(BarAction::TrayClick)
    }
}
