//! NovaOS Apex — Notepad Native Application
//! Windows 11-class text editor with line numbers, status bar, and dirty buffer tracking.

#![allow(dead_code)]

use novaos_apex_toolkit::*;

pub struct NovaNotepadApp {
    pub bounds: Rect,
    pub filename: [u8; 64],
    pub filename_len: usize,
    pub buffer: [u8; 2048],
    pub buffer_len: usize,
    pub cursor_pos: usize,
    pub is_dirty: bool,
    pub line_count: u32,
}

impl NovaNotepadApp {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let mut app = Self {
            bounds: Rect::new(x, y, width, height),
            filename: [0u8; 64],
            filename_len: 0,
            buffer: [0u8; 2048],
            buffer_len: 0,
            cursor_pos: 0,
            is_dirty: false,
            line_count: 1,
        };
        app.set_filename("Untitled.txt");
        let initial_text = b"Welcome to NovaOS Notepad!\n\nWindows 11-class native text editor.\n- UTF-8 text buffer\n- Line numbers & status bar\n- VFS disk integration\n";
        app.set_text(initial_text);
        app
    }

    pub fn set_filename(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(64);
        self.filename[..len].copy_from_slice(&bytes[..len]);
        self.filename_len = len;
    }

    pub fn set_text(&mut self, bytes: &[u8]) {
        let len = bytes.len().min(2048);
        self.buffer[..len].copy_from_slice(&bytes[..len]);
        self.buffer_len = len;
        self.cursor_pos = len;
        self.recalc_lines();
    }

    pub fn insert_char(&mut self, c: u8) {
        if self.buffer_len < 2048 {
            self.buffer[self.buffer_len] = c;
            self.buffer_len += 1;
            self.cursor_pos = self.buffer_len;
            self.is_dirty = true;
            self.recalc_lines();
        }
    }

    pub fn backspace(&mut self) {
        if self.buffer_len > 0 {
            self.buffer_len -= 1;
            self.cursor_pos = self.buffer_len;
            self.is_dirty = true;
            self.recalc_lines();
        }
    }

    fn recalc_lines(&mut self) {
        let mut count = 1;
        for &b in &self.buffer[..self.buffer_len] {
            if b == b'\n' {
                count += 1;
            }
        }
        self.line_count = count;
    }

    pub fn render(&self, buf: &mut [u32], stride: u32) {
        // 1. Editor background
        fill_rect(buf, stride, self.bounds, Color::NOVA_MIDNIGHT);

        // 2. Menu Bar (File, Edit, View, Help)
        let menu_h = 28;
        let menu_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, menu_h);
        fill_rect(buf, stride, menu_rect, Color::NOVA_SURFACE);
        draw_rect_outline(buf, stride, menu_rect, Color::NOVA_BORDER, 1);

        let menus = ["File", "Edit", "Format", "View", "Help"];
        let mut mx = self.bounds.x + 12;
        for m in menus {
            draw_text(buf, stride, m, mx, self.bounds.y + 6, Color::TEXT_PRIMARY);
            mx += (m.len() as i32 * 8) + 16;
        }

        // 3. Line Number Gutter
        let gutter_w = 40;
        let content_y = self.bounds.y + menu_h as i32;
        let status_h = 24;
        let content_h = self.bounds.height.saturating_sub(menu_h + status_h);

        let gutter_rect = Rect::new(self.bounds.x, content_y, gutter_w, content_h);
        fill_rect(buf, stride, gutter_rect, Color::NOVA_CARD);
        draw_rect_outline(buf, stride, gutter_rect, Color::NOVA_BORDER, 1);

        for line_idx in 1..=self.line_count.min(15) {
            let mut l_buf = [b' '; 3];
            if line_idx >= 10 {
                l_buf[0] = b'0' + (line_idx / 10) as u8;
                l_buf[1] = b'0' + (line_idx % 10) as u8;
            } else {
                l_buf[1] = b'0' + line_idx as u8;
            }
            let l_str = core::str::from_utf8(&l_buf).unwrap_or(" 1");
            let ly = content_y + 8 + ((line_idx as i32 - 1) * 16);
            draw_text(buf, stride, l_str, self.bounds.x + 8, ly, Color::TEXT_SECONDARY);
        }

        // 4. Main Text Edit Canvas
        let text_area = Rect::new(self.bounds.x + gutter_w as i32, content_y, self.bounds.width - gutter_w, content_h);
        fill_rect(buf, stride, text_area, Color::from_hex(0x13171F));

        // Render lines of text
        let mut tx = text_area.x + 8;
        let mut ty = content_y + 8;
        let mut cur_line_len = 0;
        for &b in &self.buffer[..self.buffer_len] {
            if b == b'\n' {
                ty += 16;
                tx = text_area.x + 8;
                cur_line_len = 0;
                if ty > content_y + content_h as i32 - 16 {
                    break;
                }
            } else if b >= 32 && b < 127 && cur_line_len < 50 {
                let mut c_buf = [0u8; 1];
                c_buf[0] = b;
                if let Ok(c_str) = core::str::from_utf8(&c_buf) {
                    draw_text(buf, stride, c_str, tx, ty, Color::TEXT_PRIMARY);
                }
                tx += 8;
                cur_line_len += 1;
            }
        }

        // Draw blinking cursor pill
        fill_rect(buf, stride, Rect::new(tx, ty, 2, 14), Color::NOVA_ACCENT);

        // 5. Status Bar (Ln 1, Col 1 | 100% | Windows (CRLF) | UTF-8)
        let status_y = self.bounds.bottom() - status_h as i32;
        let status_rect = Rect::new(self.bounds.x, status_y, self.bounds.width, status_h);
        fill_rect(buf, stride, status_rect, Color::NOVA_SURFACE);
        draw_rect_outline(buf, stride, status_rect, Color::NOVA_BORDER, 1);

        let dirty_str = if self.is_dirty { "*" } else { "" };
        let fname = core::str::from_utf8(&self.filename[..self.filename_len]).unwrap_or("Untitled.txt");
        draw_text(buf, stride, fname, self.bounds.x + 12, status_y + 6, Color::TEXT_SECONDARY);
        draw_text(buf, stride, dirty_str, self.bounds.x + 12 + (fname.len() as i32 * 8), status_y + 6, Color::NOVA_ACCENT);

        draw_text(buf, stride, "Ln 1, Col 1", self.bounds.right() - 220, status_y + 6, Color::TEXT_SECONDARY);
        draw_text(buf, stride, "100%", self.bounds.right() - 130, status_y + 6, Color::TEXT_SECONDARY);
        draw_text(buf, stride, "UTF-8", self.bounds.right() - 60, status_y + 6, Color::TEXT_SECONDARY);
    }
}
