//! NovaOS Zero — NovaUI Declarative GUI Toolkit
//! Reusable desktop UI components: Button, Label, ProgressBar, Panel, and State Management.

use super::compositor::{Color, Rect, Surface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}

pub struct Button {
    pub label: &'static str,
    pub rect: Rect,
    pub state: WidgetState,
}

impl Button {
    pub const fn new(label: &'static str, rect: Rect) -> Self {
        Button {
            label,
            rect,
            state: WidgetState::Normal,
        }
    }

    pub fn render(&self, dest: &mut Surface) {
        let (bg, border, text_color) = match self.state {
            WidgetState::Normal => (Color::rgb(0x3B, 0x82, 0xF6), Color::rgb(0x25, 0x63, 0xEB), Color::WHITE),
            WidgetState::Hover => (Color::rgb(0x60, 0xA5, 0xFA), Color::WHITE, Color::WHITE),
            WidgetState::Pressed => (Color::rgb(0x1D, 0x4E, 0xD8), Color::WHITE, Color::rgb(0xE2, 0xE8, 0xF0)),
            WidgetState::Disabled => (Color::rgb(0x47, 0x55, 0x69), Color::rgb(0x33, 0x41, 0x55), Color::rgb(0x94, 0xA3, 0xB8)),
        };

        dest.fill_rect(self.rect, bg);
        dest.draw_rect_outline(self.rect, 1, border);

        // Center text roughly
        let text_x = self.rect.x + (self.rect.width as i32 / 2) - (self.label.len() as i32 * 4);
        let text_y = self.rect.y + (self.rect.height as i32 / 2) - 8;
        dest.draw_text(text_x, text_y, self.label, text_color);
    }
}

pub struct Label {
    pub text: &'static str,
    pub x: i32,
    pub y: i32,
    pub color: Color,
}

impl Label {
    pub const fn new(text: &'static str, x: i32, y: i32, color: Color) -> Self {
        Label { text, x, y, color }
    }

    pub fn render(&self, dest: &mut Surface) {
        dest.draw_text(self.x, self.y, self.text, self.color);
    }
}

pub struct ProgressBar {
    pub rect: Rect,
    pub percent: u8, // 0..100
}

impl ProgressBar {
    pub const fn new(rect: Rect, percent: u8) -> Self {
        ProgressBar { rect, percent }
    }

    pub fn render(&self, dest: &mut Surface) {
        // Track
        dest.fill_rect(self.rect, Color::rgb(0x1E, 0x29, 0x3B));
        dest.draw_rect_outline(self.rect, 1, Color::rgb(0x47, 0x55, 0x69));

        // Filled portion
        let fill_width = ((self.rect.width as u32 * self.percent.min(100) as u32) / 100) as u32;
        if fill_width > 2 {
            dest.fill_rect(
                Rect::new(self.rect.x + 1, self.rect.y + 1, fill_width - 2, self.rect.height - 2),
                Color::GREEN,
            );
        }
    }
}

pub struct Panel {
    pub rect: Rect,
    pub bg_color: Color,
    pub border_color: Color,
}

impl Panel {
    pub const fn new(rect: Rect, bg: Color, border: Color) -> Self {
        Panel {
            rect,
            bg_color: bg,
            border_color: border,
        }
    }

    pub fn render(&self, dest: &mut Surface) {
        dest.fill_rect(self.rect, self.bg_color);
        dest.draw_rect_outline(self.rect, 1, self.border_color);
    }
}
