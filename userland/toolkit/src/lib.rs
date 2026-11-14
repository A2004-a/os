//! NovaOS Apex — Graphical Widget & UI Toolkit (NovaUI)
//! Windows 11-class modern widget hierarchy, theme engine, and event dispatcher.

#![no_std]

extern crate novaos_apex_runtime;

use novaos_apex_runtime::*;

// 1. Geometry Primitives
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const ZERO: Self = Self { width: 0, height: 0 };
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const ZERO: Self = Self { x: 0, y: 0, width: 0, height: 0 };

    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub const fn left(&self) -> i32 {
        self.x
    }
    pub const fn top(&self) -> i32 {
        self.y
    }
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
}

// 2. Color Palette & Themes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u32);

impl Color {
    pub const TRANSPARENT: Self = Self(0);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);

    // NovaOS Apex Windows 11-class Dark/Mica Identity
    pub const NOVA_MIDNIGHT: Self = Self::from_hex(0x0D1117);
    pub const NOVA_SURFACE: Self = Self::from_hex(0x161B22);
    pub const NOVA_CARD: Self = Self::from_hex(0x21262D);
    pub const NOVA_BORDER: Self = Self::from_hex(0x30363D);
    pub const NOVA_ACCENT: Self = Self::from_hex(0x00D4FF);
    pub const NOVA_ACCENT_HOVER: Self = Self::from_hex(0x33DDFF);
    pub const NOVA_ACCENT_PRESSED: Self = Self::from_hex(0x00A3C4);
    pub const TEXT_PRIMARY: Self = Self::from_hex(0xF0F6FC);
    pub const TEXT_SECONDARY: Self = Self::from_hex(0x8B949E);
    pub const SUCCESS_GREEN: Self = Self::from_hex(0x238636);
    pub const DANGER_RED: Self = Self::from_hex(0xDA3633);

    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_argb(255, r, g, b)
    }

    pub const fn from_hex(hex: u32) -> Self {
        Self::from_rgb(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }
}

// 3. Rasterization Helpers (Surface memory writer)
pub fn fill_rect(buf: &mut [u32], stride: u32, rect: Rect, color: Color) {
    let buf_h = (buf.len() as u32) / stride;
    let x_end = (rect.x + rect.width as i32).clamp(0, stride as i32) as u32;
    let y_end = (rect.y + rect.height as i32).clamp(0, buf_h as i32) as u32;
    let x_start = rect.x.clamp(0, stride as i32) as u32;
    let y_start = rect.y.clamp(0, buf_h as i32) as u32;

    for y in y_start..y_end {
        let row = (y * stride) as usize;
        for x in x_start..x_end {
            buf[row + x as usize] = color.0;
        }
    }
}

pub fn draw_rect_outline(buf: &mut [u32], stride: u32, rect: Rect, color: Color, thickness: u32) {
    // Top
    fill_rect(buf, stride, Rect::new(rect.x, rect.y, rect.width, thickness), color);
    // Bottom
    fill_rect(buf, stride, Rect::new(rect.x, rect.bottom() - thickness as i32, rect.width, thickness), color);
    // Left
    fill_rect(buf, stride, Rect::new(rect.x, rect.y, thickness, rect.height), color);
    // Right
    fill_rect(buf, stride, Rect::new(rect.right() - thickness as i32, rect.y, thickness, rect.height), color);
}

// 4. Widget Abstraction
pub trait Widget {
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, bounds: Rect);
    fn render(&self, buf: &mut [u32], stride: u32);
    fn handle_event(&mut self, ev: &UserEvent) -> bool;
}

// 5. Button Widget
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}

pub struct Button {
    pub bounds: Rect,
    pub label: [u8; 32],
    pub label_len: usize,
    pub state: ButtonState,
    pub clicked: bool,
}

impl Button {
    pub fn new(x: i32, y: i32, width: u32, height: u32, text: &str) -> Self {
        let mut label = [0u8; 32];
        let len = text.len().min(32);
        label[..len].copy_from_slice(&text.as_bytes()[..len]);
        Self {
            bounds: Rect::new(x, y, width, height),
            label,
            label_len: len,
            state: ButtonState::Normal,
            clicked: false,
        }
    }
}

impl Widget for Button {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn render(&self, buf: &mut [u32], stride: u32) {
        let bg_color = match self.state {
            ButtonState::Normal => Color::NOVA_CARD,
            ButtonState::Hover => Color::NOVA_ACCENT_HOVER,
            ButtonState::Pressed => Color::NOVA_ACCENT_PRESSED,
            ButtonState::Disabled => Color::from_hex(0x181C24),
        };
        fill_rect(buf, stride, self.bounds, bg_color);
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 1);
    }

    fn handle_event(&mut self, ev: &UserEvent) -> bool {
        match ev.event_type {
            EVENT_MOUSE_MOVE => {
                if self.bounds.contains(ev.arg1, ev.arg2) {
                    if self.state != ButtonState::Pressed {
                        self.state = ButtonState::Hover;
                    }
                    true
                } else {
                    if self.state == ButtonState::Hover {
                        self.state = ButtonState::Normal;
                    }
                    false
                }
            }
            EVENT_MOUSE_DOWN => {
                if self.bounds.contains(ev.arg1, ev.arg2) {
                    self.state = ButtonState::Pressed;
                    true
                } else {
                    false
                }
            }
            EVENT_MOUSE_UP => {
                if self.bounds.contains(ev.arg1, ev.arg2) && self.state == ButtonState::Pressed {
                    self.state = ButtonState::Hover;
                    self.clicked = true;
                    true
                } else {
                    self.state = ButtonState::Normal;
                    false
                }
            }
            _ => false,
        }
    }
}

// 6. ProgressBar Widget
pub struct ProgressBar {
    pub bounds: Rect,
    pub min: u32,
    pub max: u32,
    pub value: u32,
    pub fill_color: Color,
}

impl ProgressBar {
    pub fn new(x: i32, y: i32, width: u32, height: u32, max: u32) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            min: 0,
            max: if max > 0 { max } else { 100 },
            value: 0,
            fill_color: Color::NOVA_ACCENT,
        }
    }

    pub fn set_value(&mut self, val: u32) {
        self.value = val.clamp(self.min, self.max);
    }
}

impl Widget for ProgressBar {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn render(&self, buf: &mut [u32], stride: u32) {
        // Track background
        fill_rect(buf, stride, self.bounds, Color::from_hex(0x161B22));
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 1);

        // Filled progress portion
        let range = self.max - self.min;
        if range > 0 && self.value > 0 {
            let fill_w = ((self.bounds.width as u64 * self.value as u64) / range as u64) as u32;
            let fill_rect_bounds = Rect::new(self.bounds.x + 1, self.bounds.y + 1, fill_w.saturating_sub(2), self.bounds.height.saturating_sub(2));
            fill_rect(buf, stride, fill_rect_bounds, self.fill_color);
        }
    }

    fn handle_event(&mut self, _ev: &UserEvent) -> bool {
        false
    }
}

// 7. Panel / Card Container
pub struct Panel {
    pub bounds: Rect,
    pub bg_color: Color,
    pub border_color: Color,
}

impl Panel {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            bg_color: Color::NOVA_SURFACE,
            border_color: Color::NOVA_BORDER,
        }
    }
}

impl Widget for Panel {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn render(&self, buf: &mut [u32], stride: u32) {
        fill_rect(buf, stride, self.bounds, self.bg_color);
        draw_rect_outline(buf, stride, self.bounds, self.border_color, 1);
    }

    fn handle_event(&mut self, _ev: &UserEvent) -> bool {
        false
    }
}
