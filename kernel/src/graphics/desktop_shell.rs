//! NovaOS Zero — Desktop Shell Subsystem
//! Implements Nova Bar (Taskbar), Nova Launcher, Snap Assist, Virtual Workspaces, and Notification Center.

use super::compositor::{Color, Rect, Surface};
use super::window_server::{Window, WindowServer, TITLEBAR_HEIGHT};

pub const TASKBAR_HEIGHT: u32 = 40;
pub const MAX_WORKSPACES: usize = 4;
pub const MAX_NOTIFICATIONS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapState {
    None,
    LeftHalf,
    RightHalf,
    Maximized,
    TopLeftQuarter,
    TopRightQuarter,
    BottomLeftQuarter,
    BottomRightQuarter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy)]
pub struct Notification {
    pub title: &'static str,
    pub message: &'static str,
    pub level: NotificationLevel,
    pub timestamp_ms: u64,
    pub is_active: bool,
}

impl Notification {
    pub const fn empty() -> Self {
        Notification {
            title: "",
            message: "",
            level: NotificationLevel::Info,
            timestamp_ms: 0,
            is_active: false,
        }
    }
}

pub struct DesktopShell {
    pub screen_width: u32,
    pub screen_height: u32,
    pub active_workspace: usize, // 0..3 (Workspaces 1..4)
    pub launcher_open: bool,
    pub notifications: [Notification; MAX_NOTIFICATIONS],
    pub window_workspaces: [usize; 8], // maps window ID to workspace index
}

impl DesktopShell {
    pub const fn new(screen_width: u32, screen_height: u32) -> Self {
        DesktopShell {
            screen_width,
            screen_height,
            active_workspace: 0,
            launcher_open: false,
            notifications: [Notification::empty(); MAX_NOTIFICATIONS],
            window_workspaces: [0; 8], // default all windows to workspace 0
        }
    }

    /// Calculate window geometry based on Snap Assist state
    pub fn calculate_snap_rect(&self, snap: SnapState) -> Rect {
        let work_height = self.screen_height.saturating_sub(TASKBAR_HEIGHT);
        let half_width = self.screen_width / 2;
        let half_height = work_height / 2;

        match snap {
            SnapState::None => Rect::new(50, 50, 400, 300),
            SnapState::LeftHalf => Rect::new(0, 0, half_width, work_height),
            SnapState::RightHalf => Rect::new(half_width as i32, 0, half_width, work_height),
            SnapState::Maximized => Rect::new(0, 0, self.screen_width, work_height),
            SnapState::TopLeftQuarter => Rect::new(0, 0, half_width, half_height),
            SnapState::TopRightQuarter => Rect::new(half_width as i32, 0, half_width, half_height),
            SnapState::BottomLeftQuarter => Rect::new(0, half_height as i32, half_width, half_height),
            SnapState::BottomRightQuarter => Rect::new(half_width as i32, half_height as i32, half_width, half_height),
        }
    }

    /// Apply snap layout to a window
    pub fn snap_window(&self, win: &mut Window, snap: SnapState) {
        let target = self.calculate_snap_rect(snap);
        win.rect = target;
        // Resize surface to match new client area
        let client_height = target.height.saturating_sub(TITLEBAR_HEIGHT);
        win.surface.width = target.width;
        win.surface.height = client_height;
    }

    /// Switch active virtual workspace
    pub fn switch_workspace(&mut self, workspace_idx: usize) {
        if workspace_idx < MAX_WORKSPACES {
            self.active_workspace = workspace_idx;
        }
    }

    /// Assign a window to a specific workspace
    pub fn assign_window_to_workspace(&mut self, win_id: u32, workspace_idx: usize) {
        let idx = (win_id as usize).wrapping_sub(1);
        if idx < self.window_workspaces.len() && workspace_idx < MAX_WORKSPACES {
            self.window_workspaces[idx] = workspace_idx;
        }
    }

    /// Post a new desktop toast notification
    pub fn post_notification(&mut self, title: &'static str, message: &'static str, level: NotificationLevel, time_ms: u64) {
        for slot in self.notifications.iter_mut() {
            if !slot.is_active {
                *slot = Notification {
                    title,
                    message,
                    level,
                    timestamp_ms: time_ms,
                    is_active: true,
                };
                return;
            }
        }
        // Evict oldest slot
        self.notifications[0] = Notification {
            title,
            message,
            level,
            timestamp_ms: time_ms,
            is_active: true,
        };
    }

    /// Render the entire modern desktop shell: wallpaper, windows on active workspace,
    /// Nova Bar (taskbar), Nova Launcher, toast notifications, and cursor
    pub fn render_shell(&self, ws: &WindowServer, dest: &mut Surface) {
        // 1. Desktop Wallpaper (Dark Slate / Deep Navy theme)
        dest.clear(Color::DARK_SLATE);

        // 2. Render Windows that belong to active workspace
        for slot in ws.windows.iter() {
            if let Some(ref win) = slot {
                let win_ws_idx = (win.id as usize).wrapping_sub(1);
                let assigned_ws = if win_ws_idx < self.window_workspaces.len() {
                    self.window_workspaces[win_ws_idx]
                } else {
                    0
                };

                if assigned_ws == self.active_workspace {
                    self.render_window_decorations(win, dest);
                }
            }
        }

        // 3. Render Nova Bar (Taskbar)
        self.render_novabar(ws, dest);

        // 4. Render Nova Launcher (if open)
        if self.launcher_open {
            self.render_launcher(dest);
        }

        // 5. Render Notifications
        self.render_notifications(dest);

        // 6. Render Mouse Pointer Overlay
        self.render_cursor(dest, ws.mouse_x, ws.mouse_y);
    }

    fn render_novabar(&self, ws: &WindowServer, dest: &mut Surface) {
        let bar_y = self.screen_height.saturating_sub(TASKBAR_HEIGHT) as i32;
        let bar_rect = Rect::new(0, bar_y, self.screen_width, TASKBAR_HEIGHT);

        // Taskbar background with top accent border
        dest.fill_rect(bar_rect, Color::rgb(0x0F, 0x17, 0x2A));
        dest.fill_rect(Rect::new(0, bar_y, self.screen_width, 1), Color::rgb(0x33, 0x41, 0x55));

        // Start / Nova Launcher Button
        let start_color = if self.launcher_open { Color::rgb(0x25, 0x63, 0xEB) } else { Color::rgb(0x1E, 0x29, 0x3B) };
        let start_rect = Rect::new(8, bar_y + 4, 76, 32);
        dest.fill_rect(start_rect, start_color);
        dest.draw_rect_outline(start_rect, 1, Color::rgb(0x3B, 0x82, 0xF6));
        dest.draw_text(16, bar_y + 12, "Nova OS", Color::WHITE);

        // Active Window Tabs
        let mut tab_x = 92i32;
        for slot in ws.windows.iter().flatten() {
            let win_ws_idx = (slot.id as usize).wrapping_sub(1);
            let assigned_ws = if win_ws_idx < self.window_workspaces.len() {
                self.window_workspaces[win_ws_idx]
            } else {
                0
            };

            if assigned_ws == self.active_workspace {
                let tab_rect = Rect::new(tab_x, bar_y + 4, 110, 32);
                let tab_bg = if slot.is_focused { Color::rgb(0x33, 0x41, 0x55) } else { Color::rgb(0x1E, 0x29, 0x3B) };
                dest.fill_rect(tab_rect, tab_bg);
                dest.draw_rect_outline(tab_rect, 1, Color::rgb(0x47, 0x55, 0x69));

                let title = if slot.title.len() > 10 { &slot.title[..10] } else { slot.title };
                dest.draw_text(tab_x + 8, bar_y + 12, title, Color::WHITE);
                tab_x += 116;
            }
        }

        // Workspace Indicators (Right side)
        let ws_x = self.screen_width as i32 - 220;
        dest.draw_text(ws_x, bar_y + 12, "WS:", Color::rgb(0x94, 0xA3, 0xB8));
        for i in 0..MAX_WORKSPACES {
            let btn_x = ws_x + 28 + (i as i32 * 22);
            let btn_rect = Rect::new(btn_x, bar_y + 8, 18, 24);
            let btn_color = if i == self.active_workspace { Color::BLUE } else { Color::rgb(0x1E, 0x29, 0x3B) };
            dest.fill_rect(btn_rect, btn_color);

            let label = match i {
                0 => "1",
                1 => "2",
                2 => "3",
                _ => "4",
            };
            dest.draw_text(btn_x + 5, bar_y + 12, label, Color::WHITE);
        }

        // System Tray: Network & Clock
        let tray_x = self.screen_width as i32 - 110;
        dest.draw_text(tray_x, bar_y + 12, "eth0:UP", Color::GREEN);
        dest.draw_text(tray_x + 60, bar_y + 12, "10:42", Color::WHITE);
    }

    fn render_launcher(&self, dest: &mut Surface) {
        let launcher_x = 8i32;
        let launcher_y = self.screen_height as i32 - TASKBAR_HEIGHT as i32 - 240;
        let launcher_rect = Rect::new(launcher_x, launcher_y, 220, 235);

        // Menu container with shadow and modern dark background
        dest.fill_rect(Rect::new(launcher_x + 4, launcher_y + 4, 220, 235), Color::rgba(0, 0, 0, 100));
        dest.fill_rect(launcher_rect, Color::rgb(0x1E, 0x29, 0x3B));
        dest.draw_rect_outline(launcher_rect, 1, Color::rgb(0x47, 0x55, 0x69));

        // Search bar
        let search_rect = Rect::new(launcher_x + 8, launcher_y + 8, 204, 26);
        dest.fill_rect(search_rect, Color::rgb(0x0F, 0x17, 0x2A));
        dest.draw_rect_outline(search_rect, 1, Color::rgb(0x3B, 0x82, 0xF6));
        dest.draw_text(launcher_x + 14, launcher_y + 14, "Search apps...", Color::rgb(0x94, 0xA3, 0xB8));

        // App list
        let apps = [
            "> Terminal Shell",
            "> File Manager",
            "> System Monitor",
            "> Settings Panel",
            "> Network Center",
        ];

        for (i, app) in apps.iter().enumerate() {
            let item_y = launcher_y + 44 + (i as i32 * 36);
            let item_rect = Rect::new(launcher_x + 8, item_y, 204, 30);
            dest.fill_rect(item_rect, Color::rgb(0x33, 0x41, 0x55));
            dest.draw_text(launcher_x + 16, item_y + 8, app, Color::WHITE);
        }
    }

    fn render_notifications(&self, dest: &mut Surface) {
        let mut toast_y = 20i32;
        for notif in self.notifications.iter() {
            if notif.is_active {
                let toast_x = self.screen_width as i32 - 240;
                let toast_rect = Rect::new(toast_x, toast_y, 220, 60);

                // Toast background and border
                let border_color = match notif.level {
                    NotificationLevel::Info => Color::BLUE,
                    NotificationLevel::Warning => Color::rgb(0xF5, 0x9E, 0x0B),
                    NotificationLevel::Error => Color::RED,
                };

                dest.fill_rect(toast_rect, Color::rgb(0x1E, 0x29, 0x3B));
                dest.draw_rect_outline(toast_rect, 1, border_color);

                dest.draw_text(toast_x + 8, toast_y + 8, notif.title, Color::WHITE);
                dest.draw_text(toast_x + 8, toast_y + 30, notif.message, Color::rgb(0x94, 0xA3, 0xB8));

                toast_y += 68;
            }
        }
    }

    fn render_window_decorations(&self, win: &Window, dest: &mut Surface) {
        let x = win.rect.x;
        let y = win.rect.y;
        let w = win.rect.width;
        let h = win.rect.height;

        // Window border outline
        let border_color = if win.is_focused { Color::BLUE } else { Color::BORDER };
        dest.draw_rect_outline(Rect::new(x, y, w, h), 1, border_color);

        // Titlebar
        let titlebar_color = if win.is_focused { Color::TITLEBAR_BG } else { Color::WINDOW_BG };
        dest.fill_rect(Rect::new(x + 1, y + 1, w - 2, TITLEBAR_HEIGHT), titlebar_color);

        // Title text
        dest.draw_text(x + 8, y + 5, win.title, Color::WHITE);

        // Close Button [x]
        let btn_x = x + w as i32 - 18;
        let btn_y = y + 4;
        dest.fill_rect(Rect::new(btn_x, btn_y, 14, 14), Color::RED);
        dest.draw_text(btn_x + 3, btn_y, "x", Color::WHITE);

        // Blit Client Area Surface
        dest.blit(&win.surface, x + 1, y + TITLEBAR_HEIGHT as i32);
    }

    fn render_cursor(&self, dest: &mut Surface, cx: i32, cy: i32) {
        const CURSOR_MASK: [u16; 12] = [
            0b1000_0000_0000,
            0b1100_0000_0000,
            0b1110_0000_0000,
            0b1111_0000_0000,
            0b1111_1000_0000,
            0b1111_1100_0000,
            0b1111_1110_0000,
            0b1111_0000_0000,
            0b1101_1000_0000,
            0b1000_1100_0000,
            0b0000_1100_0000,
            0b0000_0110_0000,
        ];

        for (row, &bits) in CURSOR_MASK.iter().enumerate() {
            for col in 0..12 {
                if (bits & (0x8000 >> col)) != 0 {
                    dest.set_pixel(cx + col, cy + row as i32, Color::WHITE);
                }
            }
        }
    }
}
