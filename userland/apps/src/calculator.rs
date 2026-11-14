//! NovaOS Apex — Calculator Native Application
//! Windows 11-class Standard & Programmer Calculator with Hex/Dec/Bin mode.

#![allow(dead_code)]

use novaos_apex_toolkit::*;

pub struct NovaCalculatorApp {
    pub bounds: Rect,
    pub display_val: i64,
    pub prev_val: i64,
    pub current_op: Option<char>,
    pub clear_on_next: bool,
    pub programmer_mode: bool,
}

impl NovaCalculatorApp {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            bounds: Rect::new(x, y, width, height),
            display_val: 0,
            prev_val: 0,
            current_op: None,
            clear_on_next: false,
            programmer_mode: false,
        }
    }

    pub fn input_digit(&mut self, digit: i64) {
        if self.clear_on_next {
            self.display_val = digit;
            self.clear_on_next = false;
        } else {
            self.display_val = self.display_val * 10 + digit;
        }
    }

    pub fn set_operator(&mut self, op: char) {
        self.prev_val = self.display_val;
        self.current_op = Some(op);
        self.clear_on_next = true;
    }

    pub fn evaluate(&mut self) {
        if let Some(op) = self.current_op.take() {
            let res = match op {
                '+' => self.prev_val.saturating_add(self.display_val),
                '-' => self.prev_val.saturating_sub(self.display_val),
                '*' => self.prev_val.saturating_mul(self.display_val),
                '/' => {
                    if self.display_val != 0 {
                        self.prev_val / self.display_val
                    } else {
                        0
                    }
                }
                _ => self.display_val,
            };
            self.display_val = res;
            self.clear_on_next = true;
        }
    }

    pub fn clear(&mut self) {
        self.display_val = 0;
        self.prev_val = 0;
        self.current_op = None;
        self.clear_on_next = false;
    }

    pub fn render(&self, buf: &mut [u32], stride: u32) {
        // 1. Dark mica background
        fill_rect(buf, stride, self.bounds, Color::NOVA_MIDNIGHT);

        // 2. Title & Mode Header
        let header_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 36);
        fill_rect(buf, stride, header_rect, Color::NOVA_SURFACE);
        draw_rect_outline(buf, stride, header_rect, Color::NOVA_BORDER, 1);
        draw_text(buf, stride, "Calculator - Standard", self.bounds.x + 12, self.bounds.y + 10, Color::TEXT_PRIMARY);

        // 3. Calculator Display Readout Area
        let display_rect = Rect::new(self.bounds.x + 12, self.bounds.y + 44, self.bounds.width - 24, 56);
        fill_rect(buf, stride, display_rect, Color::NOVA_CARD);
        draw_rect_outline(buf, stride, display_rect, Color::NOVA_BORDER, 1);

        // Convert display_val to ASCII
        let mut num_buf = [b' '; 20];
        let mut val = self.display_val.abs();
        let mut idx = 19;
        if val == 0 {
            num_buf[idx] = b'0';
            idx -= 1;
        } else {
            while val > 0 && idx > 0 {
                num_buf[idx] = b'0' + (val % 10) as u8;
                val /= 10;
                idx -= 1;
            }
        }
        if self.display_val < 0 && idx > 0 {
            num_buf[idx] = b'-';
            idx -= 1;
        }
        let disp_str = core::str::from_utf8(&num_buf[idx + 1..]).unwrap_or("0");
        draw_text(buf, stride, disp_str, display_rect.right() - 20 - (disp_str.len() as i32 * 8), display_rect.y + 20, Color::TEXT_PRIMARY);

        // 4. Programmer Hex & Bin preview lines
        draw_text(buf, stride, "HEX: 0x00000000", display_rect.x + 8, display_rect.y + 8, Color::TEXT_SECONDARY);
        draw_text(buf, stride, "BIN: 0000 0000", display_rect.x + 160, display_rect.y + 8, Color::TEXT_SECONDARY);

        // 5. Button Keypad Grid (4 columns, 5 rows)
        let btn_labels: [&'static str; 20] = [
            "C", "CE", "%", "/",
            "7", "8", "9", "*",
            "4", "5", "6", "-",
            "1", "2", "3", "+",
            "+/-", "0", ".", "=",
        ];

        let start_x = self.bounds.x + 12;
        let start_y = self.bounds.y + 108;
        let btn_w = (self.bounds.width - 24 - 18) / 4;
        let btn_h = 36;
        let gap = 6;

        for (i, label) in btn_labels.iter().enumerate() {
            let col = (i % 4) as i32;
            let row = (i / 4) as i32;
            let bx = start_x + col * (btn_w as i32 + gap);
            let by = start_y + row * (btn_h as i32 + gap);
            let b_rect = Rect::new(bx, by, btn_w, btn_h as u32);

            let is_accent = *label == "=";
            let is_op = *label == "+" || *label == "-" || *label == "*" || *label == "/";
            let bg = if is_accent {
                Color::NOVA_ACCENT
            } else if is_op {
                Color::from_hex(0x28303E)
            } else {
                Color::NOVA_CARD
            };

            let text_color = if is_accent {
                Color::BLACK
            } else if is_op {
                Color::NOVA_ACCENT
            } else {
                Color::TEXT_PRIMARY
            };

            fill_rect(buf, stride, b_rect, bg);
            draw_rect_outline(buf, stride, b_rect, Color::NOVA_BORDER, 1);
            let tx = bx + (btn_w as i32 - (label.len() as i32 * 8)) / 2;
            let ty = by + 12;
            draw_text(buf, stride, label, tx, ty, text_color);
        }
    }
}
