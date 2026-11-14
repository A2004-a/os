//! NovaOS Apex — Desktop Context Menu (Right-Click Surface)
//! Windows 11-class desktop context menu with mica acrylic backdrop, hover states,
//! and shell commands (View, Sort, Refresh, New File/Folder, Display Settings, Terminal).

use super::color::Color;
use super::font::draw_text;
use super::geometry::{Point, Rect};
use super::surface::Surface;

pub const CONTEXT_MENU_WIDTH: u32 = 200;
pub const ITEM_HEIGHT: u32 = 26;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextAction {
    ViewLargeIcons,
    ViewMediumIcons,
    SortByName,
    SortBySize,
    Refresh,
    NewTextFile,
    NewDirectory,
    DisplaySettings,
    OpenTerminal,
    AboutNovaOS,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub label: &'static str,
    pub action: ContextAction,
    pub is_separator: bool,
}

pub static DESKTOP_MENU_ITEMS: [MenuItem; 8] = [
    MenuItem { label: "View Icons", action: ContextAction::ViewMediumIcons, is_separator: false },
    MenuItem { label: "Sort By...", action: ContextAction::SortByName, is_separator: false },
    MenuItem { label: "Refresh", action: ContextAction::Refresh, is_separator: true },
    MenuItem { label: "New File", action: ContextAction::NewTextFile, is_separator: false },
    MenuItem { label: "New Folder", action: ContextAction::NewDirectory, is_separator: true },
    MenuItem { label: "Open in Terminal", action: ContextAction::OpenTerminal, is_separator: false },
    MenuItem { label: "Display Settings", action: ContextAction::DisplaySettings, is_separator: false },
    MenuItem { label: "About NovaOS", action: ContextAction::AboutNovaOS, is_separator: false },
];

pub struct ContextMenu {
    pub visible: bool,
    pub position: Point,
    pub hovered_index: Option<usize>,
}

impl ContextMenu {
    pub const fn new() -> Self {
        Self {
            visible: false,
            position: Point::ZERO,
            hovered_index: None,
        }
    }

    pub fn show_at(&mut self, x: i32, y: i32, screen_w: u32, screen_h: u32) {
        let menu_h = (DESKTOP_MENU_ITEMS.len() as u32 * ITEM_HEIGHT) + 12;
        let mut final_x = x;
        let mut final_y = y;

        if final_x + CONTEXT_MENU_WIDTH as i32 > screen_w as i32 {
            final_x = (screen_w as i32 - CONTEXT_MENU_WIDTH as i32).max(0);
        }
        if final_y + menu_h as i32 > screen_h as i32 - 48 {
            final_y = (screen_h as i32 - 48 - menu_h as i32).max(0);
        }

        self.position = Point::new(final_x, final_y);
        self.visible = true;
        self.hovered_index = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.hovered_index = None;
    }

    pub fn rect(&self) -> Rect {
        let menu_h = (DESKTOP_MENU_ITEMS.len() as u32 * ITEM_HEIGHT) + 12;
        Rect::new(self.position.x, self.position.y, CONTEXT_MENU_WIDTH, menu_h)
    }

    pub fn hit_test(&self, x: i32, y: i32) -> Option<ContextAction> {
        if !self.visible {
            return None;
        }
        let r = self.rect();
        if !r.contains_coords(x, y) {
            return None;
        }

        let rel_y = y - (self.position.y + 6);
        if rel_y < 0 {
            return None;
        }
        let index = (rel_y as u32 / ITEM_HEIGHT) as usize;
        if index < DESKTOP_MENU_ITEMS.len() {
            Some(DESKTOP_MENU_ITEMS[index].action)
        } else {
            None
        }
    }

    pub fn render(&self, target: &mut Surface) {
        if !self.visible {
            return;
        }

        let mrect = self.rect();

        // 1. Drop shadow
        let shadow_rect = Rect::new(mrect.x + 3, mrect.y + 3, mrect.width, mrect.height);
        target.fill_rect(shadow_rect, Color::from_argb(70, 0, 0, 0));

        // 2. Acrylic Mica panel background
        target.fill_rect(mrect, Color::NOVA_SURFACE);
        target.draw_rect_outline(mrect, Color::NOVA_BORDER, 1);

        // 3. Render items
        let mut cur_y = mrect.y + 6;
        for (i, item) in DESKTOP_MENU_ITEMS.iter().enumerate() {
            let item_rect = Rect::new(mrect.x + 6, cur_y, mrect.width - 12, ITEM_HEIGHT);
            let is_hovered = self.hovered_index == Some(i);

            if is_hovered {
                target.fill_rect(item_rect, Color::NOVA_CARD);
                target.draw_rect_outline(item_rect, Color::from_hex(0x388BFD), 1);
            }

            let text_color = if is_hovered { Color::WHITE } else { Color::TEXT_PRIMARY };
            draw_text(target, item.label, item_rect.x + 10, item_rect.y + 7, text_color);

            if item.is_separator {
                target.draw_line(mrect.x + 10, cur_y + ITEM_HEIGHT as i32 - 1, mrect.right() - 10, cur_y + ITEM_HEIGHT as i32 - 1, Color::NOVA_BORDER);
            }

            cur_y += ITEM_HEIGHT as i32;
        }
    }
}
