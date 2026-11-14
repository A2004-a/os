//! NovaOS Apex — Nova Terminal Graphical Console Application (Milestone 9)
//! Monospace bitmap terminal window running directly within the 2D window server.

#![allow(dead_code)]

use novaos_apex_runtime::*;
use novaos_apex_toolkit::*;

pub struct NovaTerminalApp {
    pub bounds: Rect,
    pub history: [[u8; 80]; 24],
    pub history_len: [usize; 24],
    pub num_lines: usize,
    pub input_buf: [u8; 80],
    pub input_len: usize,
    pub prompt: &'static str,
}

impl NovaTerminalApp {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let mut app = Self {
            bounds: Rect::new(x, y, width, height),
            history: [[0u8; 80]; 24],
            history_len: [0usize; 24],
            num_lines: 0,
            input_buf: [0u8; 80],
            input_len: 0,
            prompt: "apex@novaos:~$ ",
        };

        app.println("NovaOS Apex Terminal v0.9.0-alpha [x86_64-baremetal]");
        app.println("Copyright (c) 2026 NovaOS Apex Platform Engineering.");
        app.println("Type 'help' for built-in commands or 'uname -a' for system info.\n");
        app
    }

    pub fn println(&mut self, text: &str) {
        for line in text.lines() {
            if self.num_lines >= 24 {
                // Scroll up by 1 line
                for i in 1..24 {
                    self.history[i - 1] = self.history[i];
                    self.history_len[i - 1] = self.history_len[i];
                }
                self.num_lines = 23;
            }

            let len = line.len().min(80);
            self.history[self.num_lines][..len].copy_from_slice(&line.as_bytes()[..len]);
            self.history_len[self.num_lines] = len;
            self.num_lines += 1;
        }
    }

    pub fn clear(&mut self) {
        self.num_lines = 0;
    }

    pub fn type_char(&mut self, c: char) {
        if self.input_len < 70 && (c as u32) >= 32 && (c as u32) <= 126 {
            self.input_buf[self.input_len] = c as u8;
            self.input_len += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.input_len > 0 {
            self.input_len -= 1;
            self.input_buf[self.input_len] = 0;
        }
    }

    pub fn execute(&mut self) {
        let mut cmd_buf = [0u8; 80];
        let len = self.input_len.min(80);
        cmd_buf[..len].copy_from_slice(&self.input_buf[..len]);
        self.input_len = 0;

        let cmd_str = match core::str::from_utf8(&cmd_buf[..len]) {
            Ok(s) => s.trim(),
            Err(_) => "",
        };

        // Echo prompt + command to history
        let mut echo_buf = [0u8; 80];
        let mut ep = 0;
        let p_bytes = self.prompt.as_bytes();
        echo_buf[..p_bytes.len()].copy_from_slice(p_bytes);
        ep += p_bytes.len();
        let c_bytes = cmd_str.as_bytes();
        let c_len = c_bytes.len().min(80 - ep);
        echo_buf[ep..ep+c_len].copy_from_slice(&c_bytes[..c_len]);
        ep += c_len;
        if let Ok(echo_str) = core::str::from_utf8(&echo_buf[..ep]) {
            self.println(echo_str);
        }

        if cmd_str.is_empty() {
            return;
        }

        match cmd_str {
            "help" => {
                self.println("Built-in Commands: help, uname, whoami, pwd, ls, uptime, clear, exit");
            }
            "uname" | "uname -a" => {
                self.println("NovaOS-Apex 0.9.0-alpha #1 SMP x86_64 Bare-Metal NovaOS");
            }
            "whoami" => {
                self.println("apex (UID 1000, GID 1000, Admin)");
            }
            "pwd" => {
                self.println("/users/apex");
            }
            "ls" => {
                self.println("Documents/  Downloads/  Pictures/  notes.md  profile.json  welcome.txt");
            }
            "uptime" => {
                self.println("Uptime: 14205 ms (Preemptive Scheduler Running)");
            }
            "clear" => {
                self.clear();
            }
            "exit" => {
                self.println("Terminal session closed.");
            }
            _ => {
                self.println("command not found");
            }
        }
    }

    pub fn render(&self, buf: &mut [u32], stride: u32) {
        // Dark terminal console background
        fill_rect(buf, stride, self.bounds, Color::from_hex(0x0A0D12));
        draw_rect_outline(buf, stride, self.bounds, Color::NOVA_BORDER, 1);

        let mut y = self.bounds.y + 8;
        let x = self.bounds.x + 8;

        // Render history lines
        for i in 0..self.num_lines {
            if y + 16 > self.bounds.bottom() - 24 {
                break;
            }
            if let Ok(line_str) = core::str::from_utf8(&self.history[i][..self.history_len[i]]) {
                let col = if line_str.starts_with("apex@novaos") {
                    Color::NOVA_ACCENT
                } else {
                    Color::TEXT_PRIMARY
                };
                draw_text(buf, stride, line_str, x, y, col);
            }
            y += 18;
        }

        // Render current prompt line
        let prompt_y = self.bounds.bottom() - 22;
        draw_text(buf, stride, self.prompt, x, prompt_y, Color::NOVA_ACCENT);

        let input_x = x + (self.prompt.len() as u32 * FONT_WIDTH) as i32;
        if self.input_len > 0 {
            if let Ok(in_str) = core::str::from_utf8(&self.input_buf[..self.input_len]) {
                draw_text(buf, stride, in_str, input_x, prompt_y, Color::TEXT_PRIMARY);
            }
        }

        // Render cursor block
        let cursor_x = input_x + (self.input_len as u32 * FONT_WIDTH) as i32;
        fill_rect(buf, stride, Rect::new(cursor_x, prompt_y + 2, 8, 14), Color::NOVA_ACCENT);
    }

    pub fn handle_event(&mut self, ev: &UserEvent) -> bool {
        match ev.event_type {
            EVENT_CHAR => {
                let ch = ev.arg1 as u8 as char;
                if ch == '\n' || ch == '\r' {
                    self.execute();
                } else if ch == '\x08' {
                    self.backspace();
                } else if (ch as u32) >= 32 {
                    self.type_char(ch);
                }
                true
            }
            EVENT_KEY_DOWN => {
                if ev.arg1 == 0x1C {
                    // Enter scancode
                    self.execute();
                    true
                } else if ev.arg1 == 0x0E {
                    // Backspace scancode
                    self.backspace();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
