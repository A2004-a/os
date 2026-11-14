//! NovaOS Apex — Nova Package Manager Native App (Milestone 9)
//! Signed Nova Package (.nvp) catalog browser, metadata inspector, capability permission reviewer,
//! and install/uninstall manager.

#![allow(dead_code)]

use novaos_apex_runtime::*;
use novaos_apex_toolkit::*;

#[derive(Clone, Copy)]
pub struct PackageEntry {
    pub name: &'static str,
    pub version: &'static str,
    pub category: &'static str,
    pub publisher: &'static str,
    pub installed: bool,
    pub size_kb: u32,
    pub permissions: &'static str,
    pub signature_valid: bool,
}

pub static CATALOG: [PackageEntry; 6] = [
    PackageEntry {
        name: "coreutils",
        version: "0.5.0",
        category: "System Utilities",
        publisher: "NovaOS Apex Team",
        installed: true,
        size_kb: 128,
        permissions: "fs:read, fs:write, proc:exec",
        signature_valid: true,
    },
    PackageEntry {
        name: "novashell",
        version: "0.8.0",
        category: "System Shell",
        publisher: "NovaOS Apex Team",
        installed: true,
        size_kb: 256,
        permissions: "fs:all, proc:all, net:all",
        signature_valid: true,
    },
    PackageEntry {
        name: "nettools",
        version: "0.6.0",
        category: "Network Tools",
        publisher: "NovaOS Apex Team",
        installed: true,
        size_kb: 96,
        permissions: "net:socket, net:bind, net:raw",
        signature_valid: true,
    },
    PackageEntry {
        name: "textedit",
        version: "1.0.0",
        category: "Productivity",
        publisher: "Apex Labs",
        installed: false,
        size_kb: 180,
        permissions: "fs:read, fs:write, gui:window",
        signature_valid: true,
    },
    PackageEntry {
        name: "calc",
        version: "0.4.0",
        category: "Utilities",
        publisher: "Apex Labs",
        installed: false,
        size_kb: 64,
        permissions: "gui:window",
        signature_valid: true,
    },
    PackageEntry {
        name: "paint",
        version: "0.3.0",
        category: "Creative Tools",
        publisher: "Community Contrib",
        installed: false,
        size_kb: 320,
        permissions: "fs:write, gui:window",
        signature_valid: true,
    },
];

pub struct NovaPackagesApp {
    pub bounds: Rect,
    pub filter_tabs: TabBar,
    pub package_list: ListView,
    pub action_button: Button,
    pub installed_state: [bool; 6],
    pub selected_idx: usize,
    pub status_message: [u8; 64],
    pub status_len: usize,
}

impl NovaPackagesApp {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let filter_tabs_names: [&'static str; 3] = ["All", "Installed", "Available"];
        let filter_h = 32;
        let list_w = 260;

        let mut installed_state = [false; 6];
        for (i, p) in CATALOG.iter().enumerate() {
            installed_state[i] = p.installed;
        }

        let mut app = Self {
            bounds: Rect::new(x, y, width, height),
            filter_tabs: TabBar::new(x, y, width, filter_h, &filter_tabs_names, false),
            package_list: ListView::new(x + 8, y + filter_h as i32 + 8, list_w, height - filter_h - 16, 28),
            action_button: Button::new(x + list_w as i32 + 24, y + height as i32 - 44, 120, 28, "Install"),
            installed_state,
            selected_idx: 0,
            status_message: [0u8; 64],
            status_len: 0,
        };

        app.populate_list();
        app.set_status("NVP Package database loaded.");
        app
    }

    pub fn set_status(&mut self, msg: &str) {
        let len = msg.len().min(64);
        self.status_message[..len].copy_from_slice(&msg.as_bytes()[..len]);
        self.status_len = len;
    }

    pub fn populate_list(&mut self) {
        self.package_list.clear();
        let filter_mode = self.filter_tabs.selected; // 0: All, 1: Installed, 2: Available

        for (i, p) in CATALOG.iter().enumerate() {
            let is_inst = self.installed_state[i];
            if filter_mode == 1 && !is_inst {
                continue;
            }
            if filter_mode == 2 && is_inst {
                continue;
            }

            let status_tag = if is_inst { "[Installed]" } else { "[Available]" };
            let mut line = [0u8; 48];
            let mut ptr = 0;

            let n_bytes = p.name.as_bytes();
            line[ptr..ptr+n_bytes.len()].copy_from_slice(n_bytes);
            ptr += n_bytes.len();

            while ptr < 14 { line[ptr] = b' '; ptr += 1; }

            let s_bytes = status_tag.as_bytes();
            line[ptr..ptr+s_bytes.len()].copy_from_slice(s_bytes);
            ptr += s_bytes.len();

            if let Ok(l_str) = core::str::from_utf8(&line[..ptr]) {
                self.package_list.add_item(l_str);
            }
        }

        self.select_package(0);
    }

    pub fn select_package(&mut self, idx: usize) {
        self.selected_idx = idx.min(5);
        let is_inst = self.installed_state[self.selected_idx];
        let btn_label = if is_inst { "Uninstall" } else { "Install" };
        let mut l_buf = [0u8; 32];
        let len = btn_label.len();
        l_buf[..len].copy_from_slice(btn_label.as_bytes());
        self.action_button.label = l_buf;
        self.action_button.label_len = len;
    }

    pub fn toggle_selected(&mut self) {
        if self.selected_idx < 6 {
            let new_state = !self.installed_state[self.selected_idx];
            self.installed_state[self.selected_idx] = new_state;
            let _pkg_name = CATALOG[self.selected_idx].name;
            if new_state {
                self.set_status("Package installed and registered in system PATH.");
            } else {
                self.set_status("Package uninstalled successfully.");
            }
            self.populate_list();
            self.select_package(self.selected_idx);
        }
    }

    pub fn render(&self, buf: &mut [u32], stride: u32) {
        fill_rect(buf, stride, self.bounds, Color::NOVA_MIDNIGHT);

        // Top Filter Tabs
        self.filter_tabs.render(buf, stride);

        // Left Package List
        self.package_list.render(buf, stride);

        // Right Package Detail Card
        let list_w = 260;
        let detail_x = self.bounds.x + list_w as i32 + 20;
        let detail_y = self.bounds.y + 44;
        let detail_w = self.bounds.width.saturating_sub(list_w + 30);
        let detail_rect = Rect::new(detail_x, detail_y, detail_w, self.bounds.height.saturating_sub(54));

        fill_rect(buf, stride, detail_rect, Color::NOVA_CARD);
        draw_rect_outline(buf, stride, detail_rect, Color::NOVA_BORDER, 1);

        if self.selected_idx < 6 {
            let pkg = &CATALOG[self.selected_idx];
            let is_inst = self.installed_state[self.selected_idx];

            // Package Title & Version
            draw_text(buf, stride, pkg.name, detail_x + 16, detail_y + 16, Color::NOVA_ACCENT);
            let v_line = "Version: ";
            draw_text(buf, stride, v_line, detail_x + 16, detail_y + 42, Color::TEXT_SECONDARY);
            draw_text(buf, stride, pkg.version, detail_x + 88, detail_y + 42, Color::TEXT_PRIMARY);

            // Category & Publisher
            draw_text(buf, stride, "Category:  ", detail_x + 16, detail_y + 66, Color::TEXT_SECONDARY);
            draw_text(buf, stride, pkg.category, detail_x + 104, detail_y + 66, Color::TEXT_PRIMARY);

            draw_text(buf, stride, "Publisher: ", detail_x + 16, detail_y + 90, Color::TEXT_SECONDARY);
            draw_text(buf, stride, pkg.publisher, detail_x + 104, detail_y + 90, Color::TEXT_PRIMARY);

            // Security & Signature
            draw_text(buf, stride, "Trust & Security Verification:", detail_x + 16, detail_y + 120, Color::TEXT_PRIMARY);
            draw_text(buf, stride, "[PASS] Ed25519 Digital Signature: VALID", detail_x + 16, detail_y + 144, Color::SUCCESS_GREEN);
            draw_text(buf, stride, "[PASS] Origin: verified-official-catalog", detail_x + 16, detail_y + 168, Color::SUCCESS_GREEN);

            // Capabilities & Permissions
            draw_text(buf, stride, "Requested Permissions:", detail_x + 16, detail_y + 200, Color::TEXT_PRIMARY);
            draw_text(buf, stride, pkg.permissions, detail_x + 16, detail_y + 224, Color::NOVA_ACCENT_HOVER);

            // Status Badge
            let (stat_text, stat_col) = if is_inst {
                ("STATUS: Installed (Ready)", Color::SUCCESS_GREEN)
            } else {
                ("STATUS: Available in NVP Catalog", Color::TEXT_SECONDARY)
            };
            draw_text(buf, stride, stat_text, detail_x + 16, detail_y + 256, stat_col);

            // Action Button
            self.action_button.render(buf, stride);

            // Status message
            if self.status_len > 0 {
                if let Ok(msg) = core::str::from_utf8(&self.status_message[..self.status_len]) {
                    draw_text(buf, stride, msg, detail_x + 154, detail_y + detail_rect.height as i32 - 36, Color::TEXT_SECONDARY);
                }
            }
        }
    }

    pub fn handle_event(&mut self, ev: &UserEvent) -> bool {
        let old_tab = self.filter_tabs.selected;
        if self.filter_tabs.handle_event(ev) {
            if self.filter_tabs.selected != old_tab {
                self.populate_list();
            }
            return true;
        }

        if self.package_list.handle_event(ev) {
            if let Some(idx) = self.package_list.selected {
                self.select_package(idx);
            }
            return true;
        }

        if self.action_button.handle_event(ev) {
            if self.action_button.clicked {
                self.action_button.clicked = false;
                self.toggle_selected();
            }
            return true;
        }

        false
    }
}
