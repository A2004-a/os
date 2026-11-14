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
    pub restored_rect: Rect,
    pub surface: Surface,
    pub z_order: usize,
    pub active: bool,
    pub visible: bool,
    pub opacity: u8,
    pub flags: u32,
    pub state: WindowState,
    pub workspace: u8,
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
        let rect = Rect::new(x, y, outer_width, outer_height);

        Self {
            id,
            title: String::from(title),
            rect,
            restored_rect: rect,
            surface: Surface::new(client_width, client_height),
            z_order: 0,
            active: false,
            visible: true,
            opacity: 255,
            flags,
            state: WindowState::Normal,
            workspace: 1,
            owner_pid,
            event_queue: VecDeque::new(),
        }
    }

    #[inline(always)]
    pub fn is_frameless(&self) -> bool {
        (self.flags & WINDOW_FLAG_FRAMELESS) != 0
    }

    #[inline(always)]
    pub fn is_always_on_top(&self) -> bool {
        (self.flags & WINDOW_FLAG_ALWAYS_ON_TOP) != 0
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

    pub fn maximize_button_rect(&self) -> Option<Rect> {
        if self.is_frameless() {
            None
        } else {
            Some(Rect::new(
                self.rect.right() - (BUTTON_WIDTH as i32 * 2) - 8,
                self.rect.y + 4,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
            ))
        }
    }

    pub fn minimize_button_rect(&self) -> Option<Rect> {
        if self.is_frameless() {
            None
        } else {
            Some(Rect::new(
                self.rect.right() - (BUTTON_WIDTH as i32 * 3) - 12,
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
            // 1. Ambient Drop Shadow (elevation level 2)
            if self.active {
                for i in 1..=4 {
                    let s_rect = Rect::new(
                        self.rect.x - i,
                        self.rect.y - i + 2,
                        self.rect.width + (i as u32 * 2),
                        self.rect.height + (i as u32 * 2),
                    );
                    let shadow_color = Color::from_argb((30 / (i + 1)) as u8, 0x05, 0x07, 0x0F);
                    target.draw_rect_outline(s_rect, shadow_color, 1);
                }
            } else {
                let s_rect = Rect::new(
                    self.rect.x - 1,
                    self.rect.y + 1,
                    self.rect.width + 2,
                    self.rect.height + 2,
                );
                target.draw_rect_outline(s_rect, Color::from_argb(20, 0x00, 0x00, 0x00), 1);
            }

            // 2. Window Frame Background
            target.fill_rect(self.rect, Color::NOVA_SURFACE);

            // 3. Modern Windows 11 style border
            let border_color = if self.active { Color::from_hex(0x0078D4) } else { Color::from_hex(0x30363D) };
            target.draw_rect_outline(self.rect, border_color, 1);

            // 4. Title Bar (Mica Acrylic)
            let tb_color = if self.active { Color::from_hex(0x21262D) } else { Color::from_hex(0x161B22) };
            let tb_rect = Rect::new(self.rect.x, self.rect.y, self.rect.width, TITLE_BAR_HEIGHT);
            target.fill_rect(tb_rect, tb_color);

            // Subtle top highlight line (acrylic glass edge)
            target.draw_line(self.rect.x + 1, self.rect.y + 1, self.rect.right() - 1, self.rect.y + 1, Color::from_hex(0x30363D));

            // Dividing line between title bar and client area
            target.draw_line(self.rect.x, self.rect.y + TITLE_BAR_HEIGHT as i32, self.rect.right(), self.rect.y + TITLE_BAR_HEIGHT as i32, Color::from_hex(0x2D333B));

            // 5. App Icon Badge
            let icon_color = match self.title.as_str() {
                "Nova Files" => Color::from_hex(0x388BFD),
                "Nova System Monitor" => Color::from_hex(0x2EA043),
                "Nova Settings" => Color::from_hex(0x8B949E),
                "Nova Package Manager" => Color::from_hex(0xA371F7),
                "Nova Terminal" => Color::from_hex(0x58A6FF),
                "NovaOS Setup Wizard" => Color::from_hex(0x1F6FEB),
                "Nova Recovery Center" => Color::from_hex(0xF0883E),
                "Nova Device Manager" => Color::from_hex(0x56D364),
                _ => Color::NOVA_ACCENT,
            };
            let icon_rect = Rect::new(self.rect.x + 8, self.rect.y + 8, 16, 16);
            target.fill_rect(icon_rect, icon_color);
            let icon_symbol = match self.title.as_str() {
                "Nova Files" => "F",
                "Nova System Monitor" => "M",
                "Nova Settings" => "S",
                "Nova Package Manager" => "P",
                "Nova Terminal" => ">",
                "NovaOS Setup Wizard" => "W",
                "Nova Recovery Center" => "R",
                "Nova Device Manager" => "D",
                _ => "N",
            };
            draw_text(target, icon_symbol, icon_rect.x + 4, icon_rect.y, Color::TEXT_PRIMARY);

            // 6. Title Text
            let text_color = if self.active { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };
            draw_text(target, &self.title, self.rect.x + 32, self.rect.y + 8, text_color);

            // 7. Caption Buttons (Windows 11 style)
            if let Some(mbtn) = self.minimize_button_rect() {
                target.fill_rect(mbtn, Color::from_hex(0x1E242B));
                target.draw_rect_outline(mbtn, Color::from_hex(0x30363D), 1);
                target.draw_line(mbtn.x + 8, mbtn.y + 12, mbtn.right() - 8, mbtn.y + 12, Color::TEXT_SECONDARY);
            }
            if let Some(xbtn) = self.maximize_button_rect() {
                target.fill_rect(xbtn, Color::from_hex(0x1E242B));
                target.draw_rect_outline(xbtn, Color::from_hex(0x30363D), 1);
                if self.state == WindowState::Maximized {
                    target.draw_rect_outline(Rect::new(xbtn.x + 7, xbtn.y + 6, 10, 10), Color::TEXT_SECONDARY, 1);
                    target.draw_rect_outline(Rect::new(xbtn.x + 11, xbtn.y + 10, 10, 10), Color::TEXT_SECONDARY, 1);
                } else {
                    target.draw_rect_outline(Rect::new(xbtn.x + 8, xbtn.y + 7, 12, 10), Color::TEXT_SECONDARY, 1);
                }
            }
            if let Some(cbtn) = self.close_button_rect() {
                let c_bg = if self.active { Color::from_hex(0xC42B1C) } else { Color::from_hex(0x282E38) };
                target.fill_rect(cbtn, c_bg);
                target.draw_rect_outline(cbtn, Color::from_hex(0x404854), 1);
                draw_text(target, "X", cbtn.x + 10, cbtn.y + 4, Color::TEXT_PRIMARY);
            }
        }

        // 8. Blit Window Client Surface
        let cr = self.client_rect();
        target.blit_alpha(cr.x, cr.y, &self.surface, None, self.opacity);
    }
}
