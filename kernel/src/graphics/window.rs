//! NovaOS Apex — Desktop Window Structure & Frame Renderer
//! Manages window geometry, client surface, Z-order, state, and frame decoration.

use super::color::Color;
use super::events::WindowEvent;
use super::font::draw_text;
use super::geometry::Rect;
use super::surface::Surface;
use alloc::collections::VecDeque;
use alloc::string::String;

pub const TITLE_BAR_HEIGHT: u32 = 32;
pub const WINDOW_BORDER_WIDTH: u32 = 1;
pub const BUTTON_WIDTH: u32 = 28;
pub const BUTTON_HEIGHT: u32 = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(pub u32);

pub const WINDOW_FLAG_FRAMELESS: u32 = 1 << 0;
pub const WINDOW_FLAG_TRANSPARENT: u32 = 1 << 1;
pub const WINDOW_FLAG_MODAL: u32 = 1 << 2;
pub const WINDOW_FLAG_RESIZABLE: u32 = 1 << 3;
pub const WINDOW_FLAG_ALWAYS_ON_TOP: u32 = 1 << 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
}

pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub rect: Rect,
    pub surface: Surface,
    pub z_order: usize,
    pub active: bool,
    pub visible: bool,
    pub opacity: u8,
    pub flags: u32,
    pub state: WindowState,
    pub owner_pid: u32,
    pub event_queue: VecDeque<WindowEvent>,
}

impl Window {
    pub fn new(
        id: WindowId,
        title: &str,
        x: i32,
        y: i32,
        client_width: u32,
        client_height: u32,
        flags: u32,
        owner_pid: u32,
    ) -> Self {
        let is_frameless = (flags & WINDOW_FLAG_FRAMELESS) != 0;
        let outer_width = if is_frameless { client_width } else { client_width + WINDOW_BORDER_WIDTH * 2 };
        let outer_height = if is_frameless { client_height } else { client_height + TITLE_BAR_HEIGHT + WINDOW_BORDER_WIDTH * 2 };

        Self {
            id,
            title: String::from(title),
            rect: Rect::new(x, y, outer_width, outer_height),
            surface: Surface::new(client_width, client_height),
            z_order: 0,
            active: false,
            visible: true,
            opacity: 255,
            flags,
            state: WindowState::Normal,
            owner_pid,
            event_queue: VecDeque::new(),
        }
    }

    #[inline(always)]
    pub fn is_frameless(&self) -> bool {
        (self.flags & WINDOW_FLAG_FRAMELESS) != 0
    }

    pub fn client_rect(&self) -> Rect {
        if self.is_frameless() {
            Rect::new(self.rect.x, self.rect.y, self.rect.width, self.rect.height)
        } else {
            Rect::new(
                self.rect.x + WINDOW_BORDER_WIDTH as i32,
                self.rect.y + TITLE_BAR_HEIGHT as i32,
                self.rect.width.saturating_sub(WINDOW_BORDER_WIDTH * 2),
                self.rect.height.saturating_sub(TITLE_BAR_HEIGHT + WINDOW_BORDER_WIDTH * 2),
            )
        }
    }

    pub fn title_bar_rect(&self) -> Option<Rect> {
        if self.is_frameless() {
            None
        } else {
            Some(Rect::new(self.rect.x, self.rect.y, self.rect.width, TITLE_BAR_HEIGHT))
        }
    }

    pub fn close_button_rect(&self) -> Option<Rect> {
        if self.is_frameless() {
            None
        } else {
            Some(Rect::new(
                self.rect.right() - BUTTON_WIDTH as i32 - 4,
                self.rect.y + 4,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
            ))
        }
    }

    pub fn resize(&mut self, new_client_w: u32, new_client_h: u32) {
        let is_frameless = self.is_frameless();
        let outer_width = if is_frameless { new_client_w } else { new_client_w + WINDOW_BORDER_WIDTH * 2 };
        let outer_height = if is_frameless { new_client_h } else { new_client_h + TITLE_BAR_HEIGHT + WINDOW_BORDER_WIDTH * 2 };

        self.rect.width = outer_width;
        self.rect.height = outer_height;
        self.surface = Surface::new(new_client_w, new_client_h);
    }

    pub fn push_event(&mut self, ev: WindowEvent) {
        if self.event_queue.len() < 64 {
            self.event_queue.push_back(ev);
        }
    }

    pub fn pop_event(&mut self) -> Option<WindowEvent> {
        self.event_queue.pop_front()
    }

    /// Render the window frame, title bar, controls, and client surface into target surface
    pub fn render(&self, target: &mut Surface) {
        if !self.visible {
            return;
        }

        if !self.is_frameless() {
            // 1. Window Frame Background
            target.fill_rect(self.rect, Color::NOVA_SURFACE);

            // 2. Subtle 1-pixel border
            let border_color = if self.active { Color::NOVA_BORDER } else { Color::from_hex(0x21262D) };
            target.draw_rect_outline(self.rect, border_color, 1);

            // 3. Title Bar
            let tb_color = if self.active { Color::TITLE_BAR_ACTIVE } else { Color::TITLE_BAR_INACTIVE };
            target.fill_rect(Rect::new(self.rect.x, self.rect.y, self.rect.width, TITLE_BAR_HEIGHT), tb_color);

            // 4. Title Text
            let text_color = if self.active { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };
            draw_text(target, &self.title, self.rect.x + 12, self.rect.y + 8, text_color);

            // 5. Close Button (Red hover/accent or subtle box)
            if let Some(cbtn) = self.close_button_rect() {
                target.fill_rect(cbtn, Color::from_hex(0x282E38));
                draw_text(target, "X", cbtn.x + 10, cbtn.y + 4, Color::TEXT_PRIMARY);
            }
        }

        // 6. Blit Window Client Surface
        let cr = self.client_rect();
        target.blit_alpha(cr.x, cr.y, &self.surface, None, self.opacity);
    }
}
