//! NovaOS Apex — Run Dialog Native Application (Win+R)
//! Windows 11-class Run quick launcher with history and VFS execution.

#![allow(dead_code)]

use novaos_apex_toolkit::*;

pub struct NovaRunDialogApp {
    pub bounds: Rect,
    pub input_buffer: [u8; 64],
    pub input_len: usize,
    pub ok_button: Button,
    pub cancel_button: Button,
    pub browse_button: Button,
}

impl NovaRunDialogApp {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let btn_w = 80;
        let btn_h = 28;
        let by = y + height as i32 - 40;

        let ok_x = x + width as i32 - (btn_w as i32 * 3) - 32;
        let cancel_x = ok_x + btn_w as i32 + 8;
        let browse_x = cancel_x + btn_w as i32 + 8;

        let mut app = Self {
            bounds: Rect::new(x, y, width, height),
            input_buffer: [0u8; 64],
            input_len: 0,
            ok_button: Button::new(ok_x, by, btn_w, btn_h, "OK"),
            cancel_button: Button::new(cancel_x, by, btn_w, btn_h, "Cancel"),
            browse_button: Button::new(browse_x, by, btn_w, btn_h, "Browse..."),
        };
        app.set_command("terminal");
        app
    }

    pub fn set_command(&mut self, cmd: &str) {
        let bytes = cmd.as_bytes();
        let len = bytes.len().min(64);
        self.input_buffer[..len].copy_from_slice(&bytes[..len]);
        self.input_len = len;
    }

    pub fn render(&self, buf: &mut [u32], stride: u32) {
        // 1. Run window background
        fill_rect(buf, stride, self.bounds, Color::NOVA_SURFACE);
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 2);

        // 2. Icon & Prompt description
        let prompt_text1 = "Type the name of a program, folder, document,";
        let prompt_text2 = "or Internet resource, and NovaOS will open it.";
        draw_text(buf, stride, prompt_text1, self.bounds.x + 56, self.bounds.y + 20, Color::TEXT_PRIMARY);
        draw_text(buf, stride, prompt_text2, self.bounds.x + 56, self.bounds.y + 36, Color::TEXT_SECONDARY);

        // Windows Run Icon badge (Blue pill with 'R')
        let icon_rect = Rect::new(self.bounds.x + 16, self.bounds.y + 20, 28, 28);
        fill_rect(buf, stride, icon_rect, Color::from_hex(0x1F6FEB));
        draw_text(buf, stride, "R", icon_rect.x + 10, icon_rect.y + 6, Color::WHITE);

        // 3. Open label and text input field
        draw_text(buf, stride, "Open:", self.bounds.x + 16, self.bounds.y + 74, Color::TEXT_PRIMARY);

        let input_rect = Rect::new(self.bounds.x + 64, self.bounds.y + 68, self.bounds.width - 80, 28);
        fill_rect(buf, stride, input_rect, Color::NOVA_CARD);
        draw_rect_outline(buf, stride, input_rect, Color::NOVA_ACCENT, 1);

        let cmd_str = core::str::from_utf8(&self.input_buffer[..self.input_len]).unwrap_or("");
        draw_text(buf, stride, cmd_str, input_rect.x + 8, input_rect.y + 8, Color::TEXT_PRIMARY);
        // Blinking cursor
        fill_rect(buf, stride, Rect::new(input_rect.x + 8 + (cmd_str.len() as i32 * 8), input_rect.y + 6, 2, 16), Color::NOVA_ACCENT);

        // 4. Buttons (OK, Cancel, Browse)
        self.ok_button.render(buf, stride);
        self.cancel_button.render(buf, stride);
        self.browse_button.render(buf, stride);
    }
}
