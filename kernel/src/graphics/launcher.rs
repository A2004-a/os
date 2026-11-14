//! NovaOS Apex — Nova Launcher (Start Menu & App Drawer Surface)
//! Searchable application grid, categorized filtering, and session power controls.

use super::color::Color;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;

pub const LAUNCHER_WIDTH: u32 = 520;
pub const LAUNCHER_HEIGHT: u32 = 440;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LauncherApp {
    pub name: &'static str,
    pub desc: &'static str,
    pub category: &'static str,
    pub icon_char: char,
}

pub static APPS_CATALOG: [LauncherApp; 6] = [
    LauncherApp { name: "Nova Files", desc: "VFS file manager", category: "System", icon_char: 'F' },
    LauncherApp { name: "Terminal", desc: "Ring 3 shell", category: "Utilities", icon_char: 'T' },
    LauncherApp { name: "System Monitor", desc: "Tasks & memory", category: "System", icon_char: 'M' },
    LauncherApp { name: "Settings", desc: "Configuration", category: "System", icon_char: 'S' },
    LauncherApp { name: "Package Manager", desc: "NVP store", category: "Utilities", icon_char: 'P' },
    LauncherApp { name: "Network Center", desc: "Sockets & ports", category: "Utilities", icon_char: 'N' },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LauncherAction {
    LaunchApp(&'static str),
    PowerAction(&'static str),
    SelectTab(usize),
}

pub struct NovaLauncher {
    pub visible: bool,
    pub search_query: [u8; 32],
    pub query_len: usize,
    pub selected_tab: usize, // 0: All, 1: System, 2: Utilities
}

impl NovaLauncher {
    pub const fn new() -> Self {
        Self {
            visible: false,
            search_query: [0u8; 32],
            query_len: 0,
            selected_tab: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.clear_search();
        }
    }

    pub fn type_char(&mut self, c: char) {
        if self.query_len < 32 && (c as u32) < 128 {
            self.search_query[self.query_len] = (c as u8).to_ascii_lowercase();
            self.query_len += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.query_len > 0 {
            self.query_len -= 1;
        }
    }

    pub fn clear_search(&mut self) {
        self.query_len = 0;
    }

    pub fn search_query_str(&self) -> &str {
        core::str::from_utf8(&self.search_query[..self.query_len]).unwrap_or("")
    }

    pub fn rect(&self, screen_w: u32, screen_h: u32, taskbar_h: u32) -> Rect {
        let x = (screen_w.saturating_sub(LAUNCHER_WIDTH) / 2) as i32;
        let y = (screen_h.saturating_sub(taskbar_h + LAUNCHER_HEIGHT + 12)) as i32;
        Rect::new(x, y, LAUNCHER_WIDTH, LAUNCHER_HEIGHT)
    }

    pub fn search_box_rect(&self, launcher_rect: Rect) -> Rect {
        Rect::new(launcher_rect.x + 24, launcher_rect.y + 20, launcher_rect.width - 48, 36)
    }

    pub fn footer_rect(&self, launcher_rect: Rect) -> Rect {
        Rect::new(launcher_rect.x, launcher_rect.bottom() - 48, launcher_rect.width, 48)
    }

    pub fn render(&self, target: &mut Surface, screen_w: u32, screen_h: u32, taskbar_h: u32) {
        if !self.visible {
            return;
        }

        let lrect = self.rect(screen_w, screen_h, taskbar_h);

        // 1. Acrylic dark card backdrop with subtle rounded outline
        target.fill_rect(lrect, Color::NOVA_SURFACE);
        target.draw_rect_outline(lrect, Color::NOVA_BORDER, 1);

        // 2. Search Input Box
        let sbox = self.search_box_rect(lrect);
        target.fill_rect(sbox, Color::NOVA_CARD);
        target.draw_rect_outline(sbox, Color::NOVA_ACCENT, 1);

        let query = self.search_query_str();
        if query.is_empty() {
            draw_text(target, "Search apps, settings, and files...", sbox.x + 12, sbox.y + 10, Color::TEXT_SECONDARY);
        } else {
            draw_text(target, query, sbox.x + 12, sbox.y + 10, Color::TEXT_PRIMARY);
            // Blinking/static cursor bar
            let cursor_x = sbox.x + 12 + (query.len() as i32 * 8);
            target.fill_rect(Rect::new(cursor_x, sbox.y + 8, 2, 20), Color::NOVA_ACCENT);
        }

        // 3. Category Filter Tabs: [All] [System] [Utilities]
        let tabs = ["All", "System", "Utilities"];
        let mut tab_x = lrect.x + 24;
        let tab_y = sbox.bottom() + 12;

        for (idx, &tab) in tabs.iter().enumerate() {
            let is_selected = idx == self.selected_tab;
            let tab_w: u32 = 70;
            let trect = Rect::new(tab_x, tab_y, tab_w, 24);

            let bg = if is_selected { Color::NOVA_CARD } else { Color::NOVA_SURFACE };
            target.fill_rect(trect, bg);
            if is_selected {
                target.draw_line(trect.x + 4, trect.bottom() - 1, trect.right() - 4, trect.bottom() - 1, Color::NOVA_ACCENT);
            }

            let text_col = if is_selected { Color::NOVA_ACCENT } else { Color::TEXT_SECONDARY };
            draw_text(target, tab, trect.x + 10, trect.y + 4, text_col);

            tab_x += tab_w as i32 + 8;
        }

        // 4. Pinned Application Grid (2 columns x 3 rows)
        let grid_y = tab_y + 36;
        let card_w = (lrect.width - 64) / 2;
        let card_h = 56;

        let query = self.search_query_str();
        let target_category = match self.selected_tab {
            1 => "System",
            2 => "Utilities",
            _ => "",
        };

        let mut displayed_count = 0;
        for (_i, app) in APPS_CATALOG.iter().enumerate() {
            // Category filter
            if !target_category.is_empty() && app.category != target_category {
                continue;
            }

            // Search query filter (simple substring matching)
            if !query.is_empty() {
                let mut matches = false;
                for byte in app.name.bytes() {
                    if byte.to_ascii_lowercase() == query.as_bytes()[0] {
                        matches = true;
                        break;
                    }
                }
                if !matches {
                    continue;
                }
            }

            let col = displayed_count % 2;
            let row = displayed_count / 2;
            let card_x = lrect.x + 24 + (col as i32 * (card_w as i32 + 16));
            let card_cur_y = grid_y + (row as i32 * (card_h as i32 + 12));

            let crect = Rect::new(card_x, card_cur_y, card_w, card_h);
            target.fill_rect(crect, Color::NOVA_CARD);
            target.draw_rect_outline(crect, Color::NOVA_BORDER, 1);

            // App Icon Circle / Square
            let icon_rect = Rect::new(card_x + 8, card_cur_y + 8, 40, 40);
            target.fill_rect(icon_rect, Color::from_hex(0x283244));
            target.draw_rect_outline(icon_rect, Color::NOVA_ACCENT, 1);

            let mut icon_buf = [0u8; 1];
            icon_buf[0] = app.icon_char as u8;
            let icon_str = core::str::from_utf8(&icon_buf).unwrap_or("?");
            draw_text(target, icon_str, icon_rect.x + 16, icon_rect.y + 12, Color::NOVA_ACCENT);

            // App Name & Description
            draw_text(target, app.name, card_x + 56, card_cur_y + 10, Color::TEXT_PRIMARY);
            draw_text(target, app.desc, card_x + 56, card_cur_y + 28, Color::TEXT_SECONDARY);

            displayed_count += 1;
            if displayed_count >= 6 {
                break;
            }
        }

        // 5. User Profile & Power Actions Footer
        let frect = self.footer_rect(lrect);
        target.fill_rect(frect, Color::from_hex(0x11161D));
        target.draw_line(frect.x, frect.y, frect.right(), frect.y, Color::NOVA_BORDER);

        // User Avatar Circle & Username
        let avatar_rect = Rect::new(frect.x + 16, frect.y + 8, 32, 32);
        target.fill_rect(avatar_rect, Color::from_hex(0x283244));
        target.draw_rect_outline(avatar_rect, Color::NOVA_ACCENT, 1);
        draw_text(target, "A", avatar_rect.x + 12, avatar_rect.y + 8, Color::NOVA_ACCENT);
        draw_text(target, "apex (Admin)", frect.x + 56, frect.y + 16, Color::TEXT_PRIMARY);

        // Power Options [Lock] [Restart] [Shut Down]
        let power_x = frect.right() - 220;
        draw_text(target, "[Lock]", power_x, frect.y + 16, Color::TEXT_SECONDARY);
        draw_text(target, "[Restart]", power_x + 60, frect.y + 16, Color::WARNING_AMBER);
        draw_text(target, "[Shutdown]", power_x + 140, frect.y + 16, Color::DANGER_RED);
    }

    pub fn hit_test(
        &self,
        x: i32,
        y: i32,
        screen_w: u32,
        screen_h: u32,
        taskbar_h: u32,
    ) -> Option<LauncherAction> {
        if !self.visible {
            return None;
        }

        let lrect = self.rect(screen_w, screen_h, taskbar_h);
        if !lrect.contains_coords(x, y) {
            return None;
        }

        // Check footer power buttons
        let frect = self.footer_rect(lrect);
        if frect.contains_coords(x, y) {
            let power_x = frect.right() - 220;
            if x >= power_x && x < power_x + 50 {
                return Some(LauncherAction::PowerAction("lock"));
            } else if x >= power_x + 60 && x < power_x + 130 {
                return Some(LauncherAction::PowerAction("restart"));
            } else if x >= power_x + 140 && x < power_x + 220 {
                return Some(LauncherAction::PowerAction("shutdown"));
            }
        }

        // Check category tabs
        let sbox = self.search_box_rect(lrect);
        let tab_y = sbox.bottom() + 12;
        if y >= tab_y && y <= tab_y + 24 {
            let rel_x = x - (lrect.x + 24);
            if rel_x >= 0 && rel_x < 70 {
                return Some(LauncherAction::SelectTab(0));
            } else if rel_x >= 78 && rel_x < 148 {
                return Some(LauncherAction::SelectTab(1));
            } else if rel_x >= 156 && rel_x < 226 {
                return Some(LauncherAction::SelectTab(2));
            }
        }

        // Check application cards
        let grid_y = tab_y + 36;
        let card_w = (lrect.width - 64) / 2;
        let card_h = 56;

        let mut displayed_count = 0;
        for app in APPS_CATALOG.iter() {
            let target_category = match self.selected_tab {
                1 => "System",
                2 => "Utilities",
                _ => "",
            };
            if !target_category.is_empty() && app.category != target_category {
                continue;
            }

            let col = displayed_count % 2;
            let row = displayed_count / 2;
            let card_x = lrect.x + 24 + (col as i32 * (card_w as i32 + 16));
            let card_cur_y = grid_y + (row as i32 * (card_h as i32 + 12));

            let crect = Rect::new(card_x, card_cur_y, card_w, card_h);
            if crect.contains_coords(x, y) {
                return Some(LauncherAction::LaunchApp(app.name));
            }

            displayed_count += 1;
            if displayed_count >= 6 {
                break;
            }
        }

        None
    }
}
