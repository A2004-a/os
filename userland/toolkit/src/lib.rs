//! NovaOS Apex — Graphical Widget & UI Toolkit (NovaUI)
//! Windows 11-class modern widget hierarchy, theme engine, and event dispatcher.

#![no_std]

extern crate novaos_apex_runtime;

pub mod font;
pub use font::{draw_char, draw_text, measure_text, FONT_HEIGHT, FONT_WIDTH};

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

    pub const fn as_u32(&self) -> u32 {
        self.0
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

        if self.label_len > 0 {
            if let Ok(label_str) = core::str::from_utf8(&self.label[..self.label_len]) {
                let text_w = self.label_len as u32 * FONT_WIDTH;
                let tx = self.bounds.x + (self.bounds.width.saturating_sub(text_w) / 2) as i32;
                let ty = self.bounds.y + (self.bounds.height.saturating_sub(FONT_HEIGHT) / 2) as i32;
                let text_col = if self.state == ButtonState::Disabled {
                    Color::TEXT_SECONDARY
                } else {
                    Color::TEXT_PRIMARY
                };
                draw_text(buf, stride, label_str, tx, ty, text_col);
            }
        }
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

// 8. Label Widget
pub struct Label {
    pub bounds: Rect,
    pub text: [u8; 64],
    pub text_len: usize,
    pub color: Color,
}

impl Label {
    pub fn new(x: i32, y: i32, width: u32, height: u32, text: &str, color: Color) -> Self {
        let mut buf = [0u8; 64];
        let len = text.len().min(64);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        Self {
            bounds: Rect::new(x, y, width, height),
            text: buf,
            text_len: len,
            color,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text_len = text.len().min(64);
        self.text[..self.text_len].copy_from_slice(&text.as_bytes()[..self.text_len]);
    }
}

impl Widget for Label {
    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
    fn render(&self, buf: &mut [u32], stride: u32) {
        if self.text_len > 0 {
            if let Ok(s) = core::str::from_utf8(&self.text[..self.text_len]) {
                draw_text(buf, stride, s, self.bounds.x, self.bounds.y, self.color);
            }
        }
    }
    fn handle_event(&mut self, _ev: &UserEvent) -> bool {
        false
    }
}

// 9. TextBox Widget
pub struct TextBox {
    pub bounds: Rect,
    pub buffer: [u8; 64],
    pub len: usize,
    pub focused: bool,
    pub placeholder: &'static str,
}

impl TextBox {
    pub fn new(x: i32, y: i32, width: u32, height: u32, placeholder: &'static str) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            buffer: [0u8; 64],
            len: 0,
            focused: false,
            placeholder,
        }
    }

    pub fn text(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("")
    }

    pub fn set_text(&mut self, text: &str) {
        self.len = text.len().min(64);
        self.buffer[..self.len].copy_from_slice(&text.as_bytes()[..self.len]);
    }

    pub fn push_char(&mut self, c: char) {
        if self.len < 64 && (c as u32) >= 32 && (c as u32) <= 126 {
            self.buffer[self.len] = c as u8;
            self.len += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.len > 0 {
            self.len -= 1;
            self.buffer[self.len] = 0;
        }
    }
}

impl Widget for TextBox {
    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
    fn render(&self, buf: &mut [u32], stride: u32) {
        fill_rect(buf, stride, self.bounds, Color::NOVA_CARD);
        let border_col = if self.focused { Color::NOVA_ACCENT } else { Color::NOVA_BORDER };
        draw_rect_outline(buf, stride, self.bounds, border_col, 1);

        let tx = self.bounds.x + 8;
        let ty = self.bounds.y + (self.bounds.height.saturating_sub(FONT_HEIGHT) / 2) as i32;

        if self.len > 0 {
            if let Ok(s) = core::str::from_utf8(&self.buffer[..self.len]) {
                draw_text(buf, stride, s, tx, ty, Color::TEXT_PRIMARY);
            }
        } else if !self.placeholder.is_empty() {
            draw_text(buf, stride, self.placeholder, tx, ty, Color::TEXT_SECONDARY);
        }

        if self.focused {
            let cx = tx + (self.len as u32 * FONT_WIDTH) as i32;
            fill_rect(buf, stride, Rect::new(cx, ty, 2, FONT_HEIGHT), Color::NOVA_ACCENT);
        }
    }
    fn handle_event(&mut self, ev: &UserEvent) -> bool {
        match ev.event_type {
            EVENT_MOUSE_DOWN => {
                let inside = self.bounds.contains(ev.arg1, ev.arg2);
                self.focused = inside;
                inside
            }
            EVENT_CHAR => {
                if self.focused {
                    let ch = ev.arg1 as u8 as char;
                    if ch == '\x08' {
                        self.backspace();
                    } else if (ch as u32) >= 32 {
                        self.push_char(ch);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

// 10. ListView Widget
pub struct ListView {
    pub bounds: Rect,
    pub items: [[u8; 48]; 16],
    pub items_len: [usize; 16],
    pub count: usize,
    pub selected: Option<usize>,
    pub item_height: u32,
}

impl ListView {
    pub fn new(x: i32, y: i32, width: u32, height: u32, item_height: u32) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            items: [[0u8; 48]; 16],
            items_len: [0usize; 16],
            count: 0,
            selected: None,
            item_height: if item_height > 0 { item_height } else { 24 },
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.selected = None;
    }

    pub fn add_item(&mut self, text: &str) -> bool {
        if self.count < 16 {
            let len = text.len().min(48);
            self.items[self.count][..len].copy_from_slice(&text.as_bytes()[..len]);
            self.items_len[self.count] = len;
            self.count += 1;
            true
        } else {
            false
        }
    }

    pub fn selected_text(&self) -> Option<&str> {
        let idx = self.selected?;
        if idx < self.count {
            core::str::from_utf8(&self.items[idx][..self.items_len[idx]]).ok()
        } else {
            None
        }
    }
}

impl Widget for ListView {
    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
    fn render(&self, buf: &mut [u32], stride: u32) {
        fill_rect(buf, stride, self.bounds, Color::NOVA_SURFACE);
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 1);

        for i in 0..self.count {
            let row_y = self.bounds.y + (i as u32 * self.item_height) as i32;
            if row_y + self.item_height as i32 > self.bounds.bottom() {
                break;
            }
            let row_rect = Rect::new(self.bounds.x + 1, row_y, self.bounds.width - 2, self.item_height);
            let is_sel = self.selected == Some(i);

            if is_sel {
                fill_rect(buf, stride, row_rect, Color::NOVA_CARD);
                fill_rect(buf, stride, Rect::new(row_rect.x, row_rect.y + 2, 3, row_rect.height - 4), Color::NOVA_ACCENT);
            } else if i % 2 == 1 {
                fill_rect(buf, stride, row_rect, Color::from_hex(0x13171E));
            }

            if let Ok(s) = core::str::from_utf8(&self.items[i][..self.items_len[i]]) {
                let tx = row_rect.x + 8;
                let ty = row_y + (self.item_height.saturating_sub(FONT_HEIGHT) / 2) as i32;
                let col = if is_sel { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };
                draw_text(buf, stride, s, tx, ty, col);
            }
        }
    }
    fn handle_event(&mut self, ev: &UserEvent) -> bool {
        if ev.event_type == EVENT_MOUSE_DOWN && self.bounds.contains(ev.arg1, ev.arg2) {
            let rel_y = ev.arg2 - self.bounds.y;
            let clicked_idx = (rel_y as u32 / self.item_height) as usize;
            if clicked_idx < self.count {
                self.selected = Some(clicked_idx);
                return true;
            }
        }
        false
    }
}

// 11. TabBar / NavRail Widget
pub struct TabBar {
    pub bounds: Rect,
    pub tabs: [&'static str; 8],
    pub tab_count: usize,
    pub selected: usize,
    pub is_vertical: bool,
}

impl TabBar {
    pub fn new(x: i32, y: i32, width: u32, height: u32, tabs: &[&'static str], is_vertical: bool) -> Self {
        let mut t_arr = [""; 8];
        let count = tabs.len().min(8);
        for i in 0..count {
            t_arr[i] = tabs[i];
        }
        Self {
            bounds: Rect::new(x, y, width, height),
            tabs: t_arr,
            tab_count: count,
            selected: 0,
            is_vertical,
        }
    }
}

impl Widget for TabBar {
    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
    fn render(&self, buf: &mut [u32], stride: u32) {
        fill_rect(buf, stride, self.bounds, Color::NOVA_SURFACE);
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 1);

        if self.is_vertical {
            let item_h = 36;
            for i in 0..self.tab_count {
                let ty = self.bounds.y + (i as u32 * item_h) as i32;
                let r = Rect::new(self.bounds.x + 2, ty + 2, self.bounds.width - 4, item_h - 4);
                let is_sel = i == self.selected;

                if is_sel {
                    fill_rect(buf, stride, r, Color::NOVA_CARD);
                    fill_rect(buf, stride, Rect::new(r.x, r.y + 4, 3, r.height - 8), Color::NOVA_ACCENT);
                }
                let col = if is_sel { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };
                draw_text(buf, stride, self.tabs[i], r.x + 10, ty + 10, col);
            }
        } else {
            let item_w = if self.tab_count > 0 { self.bounds.width / self.tab_count as u32 } else { 80 };
            for i in 0..self.tab_count {
                let tx = self.bounds.x + (i as u32 * item_w) as i32;
                let r = Rect::new(tx + 2, self.bounds.y + 2, item_w - 4, self.bounds.height - 4);
                let is_sel = i == self.selected;

                if is_sel {
                    fill_rect(buf, stride, r, Color::NOVA_CARD);
                    fill_rect(buf, stride, Rect::new(r.x + 4, r.bottom() - 3, r.width - 8, 2), Color::NOVA_ACCENT);
                }
                let col = if is_sel { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY };
                let t_len = self.tabs[i].len() as u32 * FONT_WIDTH;
                let text_x = r.x + (r.width.saturating_sub(t_len) / 2) as i32;
                let text_y = r.y + (r.height.saturating_sub(FONT_HEIGHT) / 2) as i32;
                draw_text(buf, stride, self.tabs[i], text_x, text_y, col);
            }
        }
    }
    fn handle_event(&mut self, ev: &UserEvent) -> bool {
        if ev.event_type == EVENT_MOUSE_DOWN && self.bounds.contains(ev.arg1, ev.arg2) {
            let clicked_idx = if self.is_vertical {
                let item_h = 36;
                ((ev.arg2 - self.bounds.y) as u32 / item_h) as usize
            } else {
                let item_w = if self.tab_count > 0 { self.bounds.width / self.tab_count as u32 } else { 80 };
                ((ev.arg1 - self.bounds.x) as u32 / item_w) as usize
            };
            if clicked_idx < self.tab_count {
                self.selected = clicked_idx;
                return true;
            }
        }
        false
    }
}

// 12. MetricCard Widget
pub struct MetricCard {
    pub bounds: Rect,
    pub title: &'static str,
    pub value: [u8; 32],
    pub value_len: usize,
    pub subtitle: &'static str,
    pub percent: u32,
    pub accent: Color,
}

impl MetricCard {
    pub fn new(x: i32, y: i32, width: u32, height: u32, title: &'static str, value: &str, subtitle: &'static str, percent: u32, accent: Color) -> Self {
        let mut v_buf = [0u8; 32];
        let v_len = value.len().min(32);
        v_buf[..v_len].copy_from_slice(&value.as_bytes()[..v_len]);
        Self {
            bounds: Rect::new(x, y, width, height),
            title,
            value: v_buf,
            value_len: v_len,
            subtitle,
            percent: percent.min(100),
            accent,
        }
    }

    pub fn set_value(&mut self, value: &str, percent: u32) {
        self.value_len = value.len().min(32);
        self.value[..self.value_len].copy_from_slice(&value.as_bytes()[..self.value_len]);
        self.percent = percent.min(100);
    }
}

impl Widget for MetricCard {
    fn bounds(&self) -> Rect {
        self.bounds
    }
    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
    fn render(&self, buf: &mut [u32], stride: u32) {
        fill_rect(buf, stride, self.bounds, Color::NOVA_CARD);
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 1);

        // Title
        draw_text(buf, stride, self.title, self.bounds.x + 12, self.bounds.y + 10, Color::TEXT_SECONDARY);

        // Big Metric Value
        if let Ok(val_str) = core::str::from_utf8(&self.value[..self.value_len]) {
            draw_text(buf, stride, val_str, self.bounds.x + 12, self.bounds.y + 30, Color::TEXT_PRIMARY);
        }

        // Mini Progress Bar
        let bar_y = self.bounds.y + 54;
        let bar_w = self.bounds.width.saturating_sub(24);
        let track_rect = Rect::new(self.bounds.x + 12, bar_y, bar_w, 6);
        fill_rect(buf, stride, track_rect, Color::NOVA_SURFACE);
        if self.percent > 0 {
            let fill_w = ((bar_w as u64 * self.percent as u64) / 100) as u32;
            fill_rect(buf, stride, Rect::new(track_rect.x, track_rect.y, fill_w, 6), self.accent);
        }

        // Subtitle
        draw_text(buf, stride, self.subtitle, self.bounds.x + 12, self.bounds.y + 66, Color::TEXT_SECONDARY);
    }
    fn handle_event(&mut self, _ev: &UserEvent) -> bool {
        false
    }
}
