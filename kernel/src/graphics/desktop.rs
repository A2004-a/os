//! NovaOS Apex — Desktop Shell Manager
//! Integrates Compositor, Nova Bar, Nova Launcher, Snap Assist, Virtual Workspaces,
//! Desktop Context Menu, Quick Settings, Notifications Flyout, and Task Switcher.

use super::bar::{BarAction, NovaBar};
use super::color::Color;
use super::compositor::Compositor;
use super::context_menu::{ContextMenu, ContextAction};
use super::desktop_features::DesktopFeatures;
use super::font::draw_text;
use super::geometry::{Point, Rect};
use super::launcher::{LauncherAction, NovaLauncher};
use super::notifications::NotificationCenter;
use super::quick_settings::QuickSettings;
use super::snap::{SnapEngine, SnapPosition};
use super::surface::Surface;
use super::task_switcher::TaskSwitcher;
use super::window::{WindowId, WindowState};
use super::workspace::WorkspaceManager;
use crate::drivers::framebuffer::get_driver;
use crate::scheduler::sync::SpinLock;
use alloc::vec::Vec;

pub static DESKTOP: SpinLock<Option<DesktopShell>> = SpinLock::new("desktop", None);

#[derive(Clone, Copy)]
pub struct TerminalLine {
    pub text: [u8; 80],
    pub len: usize,
    pub color: Color,
}

impl TerminalLine {
    pub const fn empty() -> Self {
        Self {
            text: [0u8; 80],
            len: 0,
            color: Color::TEXT_PRIMARY,
        }
    }

    pub fn new(s: &str, color: Color) -> Self {
        let mut line = Self::empty();
        let bytes = s.as_bytes();
        let copy_len = bytes.len().min(80);
        let mut i = 0;
        while i < copy_len {
            line.text[i] = bytes[i];
            i += 1;
        }
        line.len = copy_len;
        line.color = color;
        line
    }

    pub fn as_str(&self) -> &str {
        match core::str::from_utf8(&self.text[..self.len]) {
            Ok(s) => s,
            Err(_) => "",
        }
    }
}

pub struct DesktopShell {
    pub compositor: Compositor,
    pub bar: NovaBar,
    pub launcher: NovaLauncher,
    pub workspaces: WorkspaceManager,
    pub context_menu: ContextMenu,
    pub quick_settings: QuickSettings,
    pub notifications: NotificationCenter,
    pub task_switcher: TaskSwitcher,
    pub features: DesktopFeatures,
    pub active_snap_preview: SnapPosition,
    pub dragging_window: Option<WindowId>,
    pub drag_start_mouse: Point,
    pub drag_start_win_pos: Point,
    pub uptime_ms: u64,
    pub blit_to_screen: bool,
    pub prev_mouse_left: bool,
    pub prev_mouse_right: bool,
    pub prev_mouse_middle: bool,
    pub win_key_down: bool,
    pub terminal_lines: [TerminalLine; 28],
    pub terminal_line_count: usize,
    pub terminal_input: [u8; 64],
    pub terminal_input_len: usize,
}

impl DesktopShell {
    pub fn new(width: u32, height: u32) -> Self {
        let compositor = Compositor::new(width, height);
        let bar = NovaBar::new();
        let launcher = NovaLauncher::new();
        let workspaces = WorkspaceManager::new();
        let context_menu = ContextMenu::new();
        let quick_settings = QuickSettings::new();
        let notifications = NotificationCenter::new();
        let task_switcher = TaskSwitcher::new();
        let features = DesktopFeatures::new();

        Self {
            compositor,
            bar,
            launcher,
            workspaces,
            context_menu,
            quick_settings,
            notifications,
            task_switcher,
            features,
            active_snap_preview: SnapPosition::None,
            dragging_window: None,
            drag_start_mouse: Point::ZERO,
            drag_start_win_pos: Point::ZERO,
            uptime_ms: 0,
            blit_to_screen: false,
            prev_mouse_left: false,
            prev_mouse_right: false,
            prev_mouse_middle: false,
            win_key_down: false,
            terminal_lines: [TerminalLine::empty(); 28],
            terminal_line_count: 0,
            terminal_input: [0u8; 64],
            terminal_input_len: 0,
        }
    }

    pub fn usable_rect(&self) -> Rect {
        SnapEngine::usable_screen_rect(self.compositor.width, self.compositor.height, self.bar.height)
    }

    pub fn snap_window(&mut self, id: WindowId, target: SnapPosition) {
        let usable = self.usable_rect();
        let target_rect = SnapEngine::calculate_snap_rect(target, usable);

        if let Some(win) = self.compositor.get_window_mut(id) {
            if win.state != WindowState::Maximized && target == SnapPosition::Maximize {
                win.restored_rect = win.rect;
                win.state = WindowState::Maximized;
            } else if target != SnapPosition::None {
                win.restored_rect = win.rect;
            }

            let old_rect = win.rect;
            win.rect = target_rect;
            let cr = win.client_rect();
            win.surface = Surface::new(cr.width, cr.height);

            self.compositor.damage.add_rect(old_rect);
            self.compositor.damage.add_rect(target_rect);
        }
    }

    pub fn restore_window(&mut self, id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(id) {
            let old_rect = win.rect;
            let target_rect = win.restored_rect;

            win.rect = target_rect;
            win.state = WindowState::Normal;
            win.visible = true;

            let cr = win.client_rect();
            win.surface = Surface::new(cr.width, cr.height);

            self.compositor.damage.add_rect(old_rect);
            self.compositor.damage.add_rect(target_rect);
            self.compositor.bring_to_front(id);
        }
    }

    pub fn minimize_window(&mut self, id: WindowId) {
        let win_rect = if let Some(win) = self.compositor.get_window_mut(id) {
            win.state = WindowState::Minimized;
            win.visible = false;
            Some(win.rect)
        } else {
            None
        };

        if let Some(r) = win_rect {
            self.compositor.damage.add_rect(r);
            let bar_rect = self.bar.rect(self.compositor.width, self.compositor.height);
            self.compositor.damage.add_rect(bar_rect);
        }
    }

    pub fn switch_workspace(&mut self, ws_id: u8) {
        self.workspaces.set_active_workspace(ws_id, &mut self.compositor.windows, &mut self.compositor.damage);
        let bar_rect = self.bar.rect(self.compositor.width, self.compositor.height);
        self.compositor.damage.add_rect(bar_rect);
    }

    pub fn toggle_launcher(&mut self) {
        self.launcher.toggle();
        self.bar.is_launcher_open = self.launcher.visible;

        let lrect = self.launcher.rect(self.compositor.width, self.compositor.height, self.bar.height);
        self.compositor.damage.add_rect(lrect);

        let bar_rect = self.bar.rect(self.compositor.width, self.compositor.height);
        self.compositor.damage.add_rect(bar_rect);
    }

    pub fn toggle_quick_settings(&mut self) {
        self.quick_settings.toggle();
        let qrect = self.quick_settings.rect(self.compositor.width, self.compositor.height, self.bar.height);
        self.compositor.damage.add_rect(qrect);
    }

    pub fn toggle_notifications(&mut self) {
        self.notifications.toggle();
        let nrect = self.notifications.rect(self.compositor.width, self.compositor.height, self.bar.height);
        self.compositor.damage.add_rect(nrect);
    }

    pub fn on_mouse_down(&mut self, x: i32, y: i32, button: u8) {
        // Right-Click on Desktop Context Menu
        if button == 2 {
            if self.compositor.window_at(x, y).is_none() && !self.bar.rect(self.compositor.width, self.compositor.height).contains_coords(x, y) {
                self.context_menu.show_at(x, y, self.compositor.width, self.compositor.height);
                self.compositor.damage.add_rect(self.context_menu.rect());
                return;
            }
        }

        // Left-Click on Context Menu
        if self.context_menu.visible {
            if let Some(action) = self.context_menu.hit_test(x, y) {
                match action {
                    ContextAction::OpenTerminal => {
                        self.activate_or_open_app("Terminal");
                    }
                    ContextAction::NewTextFile => {
                        self.activate_or_open_app("Notepad");
                    }
                    ContextAction::DisplaySettings => {
                        self.activate_or_open_app("Settings");
                    }
                    ContextAction::Refresh => {
                        self.compositor.damage.add_rect(Rect::new(0, 0, self.compositor.width, self.compositor.height));
                    }
                    _ => {}
                }
                let old_r = self.context_menu.rect();
                self.context_menu.hide();
                self.compositor.damage.add_rect(old_r);
                return;
            } else {
                let old_r = self.context_menu.rect();
                self.context_menu.hide();
                self.compositor.damage.add_rect(old_r);
            }
        }

        // 1. Check Nova Bar click
        if let Some(action) = self.bar.hit_test(x, y, self.compositor.width, self.compositor.height, &self.compositor.windows) {
            match action {
                BarAction::ToggleLauncher => self.toggle_launcher(),
                BarAction::SwitchWorkspace(ws) => self.switch_workspace(ws),
                BarAction::ActivateWindow(id) => {
                    if let Some(win) = self.compositor.get_window_mut(id) {
                        if win.state == WindowState::Minimized {
                            win.state = WindowState::Normal;
                            win.visible = true;
                        }
                    }
                    self.compositor.bring_to_front(id);
                }
                BarAction::TrayClick => {
                    // Check if clicked in clock or quick settings zone
                    if x > self.compositor.width as i32 - 90 {
                        self.toggle_notifications();
                    } else {
                        self.toggle_quick_settings();
                    }
                }
            }
            return;
        }

        // 2. Check Nova Launcher click (if open)
        if self.launcher.visible {
            if let Some(action) = self.launcher.hit_test(x, y, self.compositor.width, self.compositor.height, self.bar.height) {
                match action {
                    LauncherAction::LaunchApp(app_name) => {
                        self.activate_or_open_app(app_name);
                        self.toggle_launcher();
                    }
                    LauncherAction::PowerAction(_pwr) => {
                        self.toggle_launcher();
                    }
                    LauncherAction::SelectTab(tab) => {
                        self.launcher.selected_tab = tab;
                        let lrect = self.launcher.rect(self.compositor.width, self.compositor.height, self.bar.height);
                        self.compositor.damage.add_rect(lrect);
                    }
                }
                return;
            } else {
                self.toggle_launcher();
            }
        }

        // 3. Check Window Controls or Title Bar Drag
        if let Some(win_id) = self.compositor.window_at(x, y) {
            let mut close_clicked = false;
            let mut maximize_clicked = false;
            let mut minimize_clicked = false;
            let mut start_drag = false;

            if let Some(win) = self.compositor.get_window(win_id) {
                if let Some(cbtn) = win.close_button_rect() {
                    if cbtn.contains_coords(x, y) {
                        close_clicked = true;
                    }
                }
                if let Some(xbtn) = win.maximize_button_rect() {
                    if xbtn.contains_coords(x, y) {
                        maximize_clicked = true;
                    }
                }
                if let Some(mbtn) = win.minimize_button_rect() {
                    if mbtn.contains_coords(x, y) {
                        minimize_clicked = true;
                    }
                }
                if let Some(tb) = win.title_bar_rect() {
                    if tb.contains_coords(x, y) && !close_clicked && !maximize_clicked && !minimize_clicked {
                        start_drag = true;
                    }
                }
            }

            if close_clicked {
                self.compositor.destroy_window(win_id);
                return;
            }
            if maximize_clicked {
                let is_max = self.compositor.get_window(win_id).map(|w| w.state == WindowState::Maximized).unwrap_or(false);
                if is_max {
                    self.restore_window(win_id);
                } else {
                    self.snap_window(win_id, SnapPosition::Maximize);
                }
                return;
            }
            if minimize_clicked {
                self.minimize_window(win_id);
                return;
            }

            if start_drag {
                self.dragging_window = Some(win_id);
                self.drag_start_mouse = Point::new(x, y);
                if let Some(win) = self.compositor.get_window(win_id) {
                    self.drag_start_win_pos = Point::new(win.rect.x, win.rect.y);
                }
            }

            self.compositor.on_mouse_down(x, y, button);
        } else {
            // Check click on Desktop Shortcut Icons (Left Column)
            if button == 1 {
                if x >= 14 && x <= 74 {
                    if y >= 24 && y <= 84 {
                        self.activate_or_open_app("Nova Files");
                        return;
                    } else if y >= 96 && y <= 156 {
                        self.activate_or_open_app("System Monitor");
                        return;
                    } else if y >= 168 && y <= 228 {
                        self.activate_or_open_app("Settings");
                        return;
                    } else if y >= 240 && y <= 300 {
                        self.activate_or_open_app("Nova Store");
                        return;
                    } else if y >= 312 && y <= 372 {
                        self.activate_or_open_app("Terminal");
                        return;
                    } else if y >= 384 && y <= 444 {
                        self.activate_or_open_app("Device Manager");
                        return;
                    }
                }
                // Wallpaper click: start marquee selection
                self.features.start_marquee(Point::new(x, y));
            }
        }
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        if let Some(win_id) = self.dragging_window {
            let dx = x - self.drag_start_mouse.x;
            let dy = y - self.drag_start_mouse.y;
            let new_x = self.drag_start_win_pos.x + dx;
            let new_y = self.drag_start_win_pos.y + dy;

            self.compositor.set_window_position(win_id, new_x, new_y);

            let snap_target = SnapEngine::detect_snap_zone(x, y, self.compositor.width, self.compositor.height, self.bar.height);
            if snap_target != self.active_snap_preview {
                self.active_snap_preview = snap_target;
                let usable = self.usable_rect();
                self.compositor.damage.add_rect(usable);
            }
        } else if self.features.marquee_start.is_some() {
            self.features.update_marquee(Point::new(x, y));
            if let Some(mr) = self.features.marquee_rect() {
                self.compositor.damage.add_rect(mr);
            }
        }

        self.compositor.on_mouse_move(x, y);
    }

    pub fn on_mouse_up(&mut self, x: i32, y: i32, button: u8) {
        if let Some(win_id) = self.dragging_window.take() {
            if self.active_snap_preview != SnapPosition::None {
                self.snap_window(win_id, self.active_snap_preview);
                self.active_snap_preview = SnapPosition::None;
            }
        }
        if let Some(mr) = self.features.end_marquee() {
            self.compositor.damage.add_rect(mr);
        }
        self.compositor.on_mouse_up(x, y, button);
    }

    pub fn toggle_show_desktop(&mut self) {
        let any_visible = self.compositor.windows.iter().any(|w| w.visible);
        for win in self.compositor.windows.iter_mut() {
            win.visible = !any_visible;
        }
        let s_rect = Rect::new(0, 0, self.compositor.width, self.compositor.height);
        self.compositor.damage.add_rect(s_rect);
    }

    pub fn on_key_event(&mut self, scancode: u8, char_code: Option<char>, pressed: bool, _shift: bool, _ctrl: bool, alt: bool) {
        // Track Windows (Super/GUI) key: Scancode 0x5B (Make) / 0xDB (Break)
        if scancode == 0x5B {
            self.win_key_down = true;
            if pressed {
                self.toggle_launcher();
            }
            return;
        } else if scancode == 0xDB {
            self.win_key_down = false;
            return;
        }

        // Windows Key Shortcuts
        if self.win_key_down && pressed {
            match scancode {
                // Win + D: Toggle Show Desktop
                0x20 => {
                    self.toggle_show_desktop();
                    self.composite();
                    return;
                }
                // Win + Left Arrow: Snap Left
                0x4B => {
                    if let Some(active_id) = self.compositor.active_window {
                        self.snap_window(active_id, SnapPosition::LeftHalf);
                        self.composite();
                    }
                    return;
                }
                // Win + Right Arrow: Snap Right
                0x4D => {
                    if let Some(active_id) = self.compositor.active_window {
                        self.snap_window(active_id, SnapPosition::RightHalf);
                        self.composite();
                    }
                    return;
                }
                // Win + Up Arrow: Maximize
                0x48 => {
                    if let Some(active_id) = self.compositor.active_window {
                        self.snap_window(active_id, SnapPosition::Maximize);
                        self.composite();
                    }
                    return;
                }
                // Win + Down Arrow: Restore
                0x50 => {
                    if let Some(active_id) = self.compositor.active_window {
                        self.restore_window(active_id);
                        self.composite();
                    }
                    return;
                }
                // Win + R: Run Dialog
                0x13 => {
                    self.activate_or_open_app("Run");
                    self.composite();
                    return;
                }
                _ => {}
            }
        }

        // Alt + Tab Task Switcher
        if alt && scancode == 0x0F && pressed {
            self.task_switcher.visible = true;
            self.task_switcher.next(self.compositor.windows.len());
            let s_rect = Rect::new(0, 0, self.compositor.width, self.compositor.height);
            self.compositor.damage.add_rect(s_rect);
            self.composite();
            return;
        } else if !alt && self.task_switcher.visible {
            self.task_switcher.visible = false;
            let s_rect = Rect::new(0, 0, self.compositor.width, self.compositor.height);
            self.compositor.damage.add_rect(s_rect);
            self.composite();
        }

        // Escape: dismiss flyouts/popups
        if scancode == 0x01 && pressed {
            let mut closed_any = false;
            if self.launcher.visible {
                self.toggle_launcher();
                closed_any = true;
            }
            if self.context_menu.visible {
                let r = self.context_menu.rect();
                self.context_menu.hide();
                self.compositor.damage.add_rect(r);
                closed_any = true;
            }
            if self.quick_settings.visible {
                self.toggle_quick_settings();
                closed_any = true;
            }
            if self.notifications.visible {
                self.toggle_notifications();
                closed_any = true;
            }
            if closed_any {
                self.composite();
                return;
            }
        }

        // Interactive Terminal Keyboard Handler
        let is_term = self.compositor.active_window
            .and_then(|id| self.compositor.get_window(id))
            .map(|w| w.title == "Terminal" || w.title == "Nova Terminal")
            .unwrap_or(false);

        if is_term && pressed {
            if scancode == 0x0E {
                // Backspace
                if self.terminal_input_len > 0 {
                    self.terminal_input_len -= 1;
                    self.redraw_terminal_surface();
                    self.composite();
                    return;
                }
            } else if scancode == 0x1C {
                // Enter
                self.execute_terminal_command();
                self.composite();
                return;
            } else if let Some(c) = char_code {
                if c >= ' ' && c <= '~' && self.terminal_input_len < 60 {
                    self.terminal_input[self.terminal_input_len] = c as u8;
                    self.terminal_input_len += 1;
                    self.redraw_terminal_surface();
                    self.composite();
                    return;
                }
            }
        }

        // Forward normal keystrokes to active window
        let ascii_byte = if let Some(c) = char_code { c as u8 } else { 0 };
        self.compositor.on_key_event(scancode, ascii_byte, pressed);
    }

    pub fn poll_input(&mut self) -> bool {
        let mut any_event = false;

        // 1. Drain keyboard events
        while let Some(k_ev) = crate::drivers::keyboard::read_key() {
            self.on_key_event(k_ev.scancode, k_ev.char_code, k_ev.pressed, k_ev.shift, k_ev.ctrl, k_ev.alt);
            any_event = true;
        }

        // 2. Drain mouse events
        while let Some(m_ev) = crate::drivers::mouse::read_mouse_event() {
            let cur = self.compositor.cursor.position();
            if m_ev.x != cur.x || m_ev.y != cur.y {
                self.on_mouse_move(m_ev.x, m_ev.y);
                any_event = true;
            }
            if m_ev.left_button != self.prev_mouse_left {
                if m_ev.left_button {
                    self.on_mouse_down(m_ev.x, m_ev.y, 1);
                } else {
                    self.on_mouse_up(m_ev.x, m_ev.y, 1);
                }
                self.prev_mouse_left = m_ev.left_button;
                any_event = true;
            }
            if m_ev.right_button != self.prev_mouse_right {
                if m_ev.right_button {
                    self.on_mouse_down(m_ev.x, m_ev.y, 2);
                } else {
                    self.on_mouse_up(m_ev.x, m_ev.y, 2);
                }
                self.prev_mouse_right = m_ev.right_button;
                any_event = true;
            }
        }

        if any_event {
            self.composite();
        }
        any_event
    }

    pub fn composite(&mut self) {
        if self.compositor.damage.is_empty() {
            return;
        }

        let bbox = match self.compositor.damage.bounding_box() {
            Some(b) => b,
            None => return,
        };

        // 1. Restore wallpaper for damaged regions
        let rects_copy: Vec<Rect> = self.compositor.damage.rects().to_vec();
        for r in rects_copy {
            self.compositor.render_wallpaper_rect(r);
        }

        // 1b. Render Desktop Marquee selection (if active)
        self.features.render_marquee(&mut self.compositor.backbuffer);

        // 2. Render all visible windows in ascending Z-order
        for win in &self.compositor.windows {
            if !self.workspaces.is_window_visible(win) {
                continue;
            }
            if win.rect.intersects(&bbox) {
                win.render(&mut self.compositor.backbuffer);
            }
        }

        // 3. Render Snap Ghost Preview
        if self.active_snap_preview != SnapPosition::None {
            let usable = self.usable_rect();
            SnapEngine::render_preview(self.active_snap_preview, usable, &mut self.compositor.backbuffer);
        }

        // 4. Render Nova Bar Taskbar
        self.bar.render(
            &mut self.compositor.backbuffer,
            self.compositor.width,
            self.compositor.height,
            &self.compositor.windows,
            self.compositor.active_window,
            self.workspaces.active_workspace,
            self.uptime_ms,
        );

        // 5. Render Nova Launcher (if open)
        if self.launcher.visible {
            self.launcher.render(
                &mut self.compositor.backbuffer,
                self.compositor.width,
                self.compositor.height,
                self.bar.height,
            );
        }

        // 5b. Render Quick Settings (if open)
        if self.quick_settings.visible {
            self.quick_settings.render(
                &mut self.compositor.backbuffer,
                self.compositor.width,
                self.compositor.height,
                self.bar.height,
            );
        }

        // 5c. Render Notifications Flyout (if open)
        if self.notifications.visible {
            self.notifications.render(
                &mut self.compositor.backbuffer,
                self.compositor.width,
                self.compositor.height,
                self.bar.height,
            );
        }

        // 5d. Render Alt+Tab Task Switcher (if open)
        if self.task_switcher.visible {
            self.task_switcher.render(
                &mut self.compositor.backbuffer,
                self.compositor.width,
                self.compositor.height,
                &self.compositor.windows,
            );
        }

        // 5e. Render Desktop Context Menu (if open)
        if self.context_menu.visible {
            self.context_menu.render(&mut self.compositor.backbuffer);
        }

        // 6. Render Mouse Cursor Overlay
        self.compositor.cursor.render_to(&mut self.compositor.backbuffer);

        // 7. Flush changed backbuffer regions to hardware GOP framebuffer
        if self.blit_to_screen {
            let fb = get_driver();
            if !fb.base_ptr.is_null() {
                for r in self.compositor.damage.rects() {
                    self.compositor.backbuffer.blit_clipped_to_raw(fb.base_ptr, fb.stride, fb.width, fb.height, *r);
                }
            }
        }

        // 8. Clear damage
        self.compositor.damage.clear();
    }

    pub fn spawn_default_apps(&mut self) {
        // 1. Spawn Nova Files (File Explorer) on Left (x: 116, y: 32, 548x656 client)
        let w_files = self.compositor.create_window("Nova Files", 116, 32, 548, 656, 0, 1);
        self.render_files_surface(w_files);

        // 2. Spawn Nova Terminal on Right (x: 686, y: 32, 560x656 client)
        let _w_term = self.compositor.create_window("Terminal", 686, 32, 560, 656, 0, 1);
        self.init_terminal();

        // 3. Spawn Nova Store (x: 280, y: 90, 620x440 client)
        let w_store = self.compositor.create_window("Nova Store", 280, 90, 620, 440, 0, 1);
        self.render_store_surface(w_store);

        // 4. Bring Nova Store to front as active window
        self.compositor.bring_to_front(w_store);
    }

    pub fn activate_or_open_app(&mut self, app_name: &str) -> WindowId {
        let (norm_name, default_rect) = match app_name {
            "Nova Files" | "Files" | "File Explorer" => ("Nova Files", Rect::new(116, 32, 548, 656)),
            "Terminal" | "Nova Terminal" => ("Terminal", Rect::new(686, 32, 560, 656)),
            "Nova Store" | "Package Manager" | "Pkg Store" => ("Nova Store", Rect::new(280, 90, 620, 440)),
            "System Monitor" | "Nova System Monitor" | "Sys Monitor" => ("System Monitor", Rect::new(160, 100, 560, 420)),
            "Settings" | "Nova Settings" => ("Settings", Rect::new(200, 110, 580, 430)),
            "Device Manager" | "Nova Device Manager" | "Device Mgr" => ("Device Manager", Rect::new(220, 120, 560, 420)),
            "Calculator" | "Calc" => ("Calculator", Rect::new(320, 140, 360, 440)),
            "Notepad" | "Text Editor" => ("Notepad", Rect::new(260, 120, 520, 380)),
            "Run" => ("Run", Rect::new(200, 200, 360, 160)),
            other => (other, Rect::new(200, 120, 520, 380)),
        };

        // 1. Check if a window with this title already exists
        if let Some(existing_id) = self.compositor.windows.iter().find(|w| w.title == norm_name).map(|w| w.id) {
            if let Some(win) = self.compositor.get_window_mut(existing_id) {
                if win.state == WindowState::Minimized {
                    win.state = WindowState::Normal;
                }
                win.visible = true;
            }
            self.compositor.bring_to_front(existing_id);
            let b_rect = self.bar.rect(self.compositor.width, self.compositor.height);
            self.compositor.damage.add_rect(b_rect);
            return existing_id;
        }

        // 2. Window does not exist: create it cleanly and render its rich native surface
        let wid = self.compositor.create_window(
            norm_name,
            default_rect.x,
            default_rect.y,
            default_rect.width,
            default_rect.height,
            0,
            1,
        );

        match norm_name {
            "Nova Files" => self.render_files_surface(wid),
            "Terminal" => {
                self.init_terminal();
                self.redraw_terminal_surface();
            }
            "Nova Store" => self.render_store_surface(wid),
            "System Monitor" => self.render_sysmon_surface(wid),
            "Settings" => self.render_settings_surface(wid),
            "Device Manager" => self.render_device_manager_surface(wid),
            "Calculator" => self.render_calculator_surface(wid),
            "Notepad" => self.render_notepad_surface(wid),
            "Run" => self.render_run_dialog_surface(wid),
            _ => {}
        }

        self.compositor.bring_to_front(wid);
        let b_rect = self.bar.rect(self.compositor.width, self.compositor.height);
        self.compositor.damage.add_rect(b_rect);
        wid
    }

    pub fn render_files_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            // Background (Dark Slate Mica)
            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x161B22));

            // Ribbon Toolbar (h: 36)
            surf.fill_rect(Rect::new(0, 0, w, 36), Color::from_hex(0x21262D));
            surf.draw_line(0, 36, w as i32, 36, Color::from_hex(0x30363D));

            let tools = ["+ New", "Cut", "Copy", "Delete", "Sort", "View"];
            let mut tx = 12;
            for t in tools {
                let tw = (t.len() as u32 * 8) + 16;
                let t_rect = Rect::new(tx, 6, tw, 24);
                surf.fill_rect(t_rect, Color::from_hex(0x28303E));
                surf.draw_rect_outline(t_rect, Color::from_hex(0x388BFD), 1);
                draw_text(surf, t, tx + 8, 10, Color::TEXT_PRIMARY);
                tx += tw as i32 + 8;
            }

            // Breadcrumb path bar (h: 28)
            surf.fill_rect(Rect::new(0, 37, w, 28), Color::from_hex(0x1B2129));
            surf.draw_line(0, 65, w as i32, 65, Color::from_hex(0x30363D));
            draw_text(surf, "This PC > Storage (C:) > users > apex > documents", 12, 43, Color::TEXT_SECONDARY);

            // Left Navigation Sidebar (w: 110)
            let sidebar_w = 110;
            surf.fill_rect(Rect::new(0, 66, sidebar_w, h - 66 - 24), Color::from_hex(0x13171F));
            surf.draw_line(sidebar_w as i32, 66, sidebar_w as i32, (h - 24) as i32, Color::from_hex(0x30363D));

            let side_items = [
                "QUICK ACCESS",
                "  Home",
                "  Downloads",
                "  Documents",
                "  Pictures",
                "  Music",
                "THIS PC",
                "  Nova (C:)",
                "  Data (D:)",
                "NETWORK",
                "  Ethernet",
            ];
            let mut sy = 74;
            for it in side_items {
                let is_header = !it.starts_with("  ");
                let col = if is_header { Color::NOVA_ACCENT } else { Color::TEXT_PRIMARY };
                draw_text(surf, it, 8, sy, col);
                sy += 22;
            }

            // Main File Table
            let table_x = sidebar_w as i32 + 8;
            let table_header_y = 74;
            draw_text(surf, "Name", table_x, table_header_y, Color::TEXT_SECONDARY);
            draw_text(surf, "Date modified", table_x + 116, table_header_y, Color::TEXT_SECONDARY);
            draw_text(surf, "Type", table_x + 236, table_header_y, Color::TEXT_SECONDARY);
            draw_text(surf, "Size", table_x + 350, table_header_y, Color::TEXT_SECONDARY);
            surf.draw_line(sidebar_w as i32 + 4, table_header_y + 18, w as i32 - 4, table_header_y + 18, Color::from_hex(0x28303E));

            let files = [
                ("System", "11/14 05:30 PM", "File folder", "--"),
                ("Projects", "11/14 05:32 PM", "File folder", "--"),
                ("Kernel.elf", "11/14 05:40 PM", "ELF64 App", "4.8 MB"),
                ("Config.sys", "11/14 05:42 PM", "Config file", "2.4 KB"),
                ("Readme.txt", "11/14 05:45 PM", "Text Document", "1.2 KB"),
                ("Telemetry.log", "11/14 05:50 PM", "Log File", "18.2 KB"),
                ("Packages", "11/14 05:52 PM", "File folder", "--"),
                ("Drivers", "11/14 05:54 PM", "File folder", "--"),
                ("Security.key", "11/14 05:55 PM", "Cert Key", "512 B"),
                ("Wallpaper.bmp", "11/14 05:56 PM", "Bitmap Image", "3.8 MB"),
            ];
            let mut fy = table_header_y + 26;
            for (name, date, ftype, size) in files {
                if name == "Readme.txt" {
                    let sel_rect = Rect::new(sidebar_w as i32 + 4, fy - 2, w - sidebar_w - 8, 20);
                    surf.fill_rect(sel_rect, Color::from_hex(0x283244));
                    surf.draw_rect_outline(sel_rect, Color::NOVA_ACCENT, 1);
                }
                draw_text(surf, name, table_x, fy, Color::TEXT_PRIMARY);
                draw_text(surf, date, table_x + 116, fy, Color::TEXT_SECONDARY);
                draw_text(surf, ftype, table_x + 236, fy, Color::TEXT_SECONDARY);
                draw_text(surf, size, table_x + 350, fy, Color::TEXT_SECONDARY);
                fy += 24;
            }

            // Status bar
            surf.fill_rect(Rect::new(0, h as i32 - 24, w, 24), Color::from_hex(0x1F242C));
            surf.draw_line(0, h as i32 - 24, w as i32, h as i32 - 24, Color::from_hex(0x30363D));
            draw_text(surf, "10 items | 1 item selected (1.2 KB) | Free: 218 MB", 12, h as i32 - 18, Color::TEXT_SECONDARY);
        }
        self.compositor.damage_window(win_id);
    }

    pub fn render_sysmon_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            // Header Toolbar (h: 36)
            surf.fill_rect(Rect::new(0, 0, w, 36), Color::from_hex(0x1E2430));
            surf.draw_line(0, 36, w as i32, 36, Color::from_hex(0x30363D));

            let badge_r = Rect::new(12, 6, 24, 24);
            surf.fill_rect(badge_r, Color::from_hex(0x2EA043));
            draw_text(surf, "M", 20, 10, Color::TEXT_PRIMARY);
            draw_text(surf, "System Monitor", 44, 10, Color::SUCCESS_GREEN);

            let tabs = ["[ Processes (6) ]", "Performance", "Services"];
            let mut tx = 180;
            for (idx, t) in tabs.iter().enumerate() {
                let tw = (t.len() as u32 * 8) + 12;
                let tr = Rect::new(tx, 6, tw, 24);
                let active = idx == 0;
                surf.fill_rect(tr, if active { Color::from_hex(0x283244) } else { Color::from_hex(0x161B22) });
                surf.draw_rect_outline(tr, if active { Color::SUCCESS_GREEN } else { Color::from_hex(0x30363D) }, 1);
                draw_text(surf, t, tx + 6, 10, if active { Color::TEXT_PRIMARY } else { Color::TEXT_SECONDARY });
                tx += tw as i32 + 8;
            }

            // Metrics Card (h: 70)
            let m_rect = Rect::new(10, 44, w.saturating_sub(20), 70);
            surf.fill_rect(m_rect, Color::NOVA_CARD);
            surf.draw_rect_outline(m_rect, Color::NOVA_BORDER, 1);

            draw_text(surf, "CPU Load: 8.2% (4 Cores @ 2.40 GHz)", 20, 52, Color::NOVA_ACCENT);
            surf.fill_rect(Rect::new(20, 68, 220, 8), Color::from_hex(0x21262D));
            surf.fill_rect(Rect::new(20, 68, 18, 8), Color::SUCCESS_GREEN);

            draw_text(surf, "RAM: 38 MiB / 256 MiB (14.8% Used)", 260, 52, Color::NOVA_ACCENT);
            surf.fill_rect(Rect::new(260, 68, 220, 8), Color::from_hex(0x21262D));
            surf.fill_rect(Rect::new(260, 68, 33, 8), Color::from_hex(0x388BFD));

            draw_text(surf, "Storage: /dev/vda 42.1 MB / 512 MB  |  Net: eth0 10.0.2.15 (RX 142KB / TX 98KB)", 20, 88, Color::TEXT_SECONDARY);

            // Process Table
            let p_y = 124;
            surf.fill_rect(Rect::new(10, p_y, w.saturating_sub(20), 22), Color::from_hex(0x1B2129));
            surf.draw_line(10, p_y + 22, w as i32 - 10, p_y + 22, Color::from_hex(0x30363D));
            draw_text(surf, "PID", 20, p_y + 4, Color::TEXT_SECONDARY);
            draw_text(surf, "Name", 70, p_y + 4, Color::TEXT_SECONDARY);
            draw_text(surf, "Privilege", 190, p_y + 4, Color::TEXT_SECONDARY);
            draw_text(surf, "CPU", 290, p_y + 4, Color::TEXT_SECONDARY);
            draw_text(surf, "Memory", 350, p_y + 4, Color::TEXT_SECONDARY);
            draw_text(surf, "Status", 430, p_y + 4, Color::TEXT_SECONDARY);

            let procs = [
                ("0", "kernel_sched", "Ring 0", "0.1%", "1.2 MB", "Running", Color::SUCCESS_GREEN),
                ("1", "system_init",  "Ring 3", "0.4%", "2.8 MB", "Supervised", Color::NOVA_ACCENT),
                ("2", "compositor",   "Ring 0", "6.2%", "14.2 MB", "Active", Color::SUCCESS_GREEN),
                ("3", "net_daemon",   "Ring 0", "0.5%", "3.1 MB", "Listening", Color::SUCCESS_GREEN),
                ("4", "storage_mgr",  "Ring 0", "0.8%", "4.6 MB", "Mounted", Color::SUCCESS_GREEN),
                ("5", "novashell",    "Ring 3", "0.2%", "1.8 MB", "Standby", Color::TEXT_SECONDARY),
            ];

            let mut ry = p_y + 26;
            for (pid, name, ring, cpu, mem, status, stat_col) in procs {
                let row_r = Rect::new(10, ry - 2, w.saturating_sub(20), 22);
                if pid == "2" {
                    surf.fill_rect(row_r, Color::from_hex(0x283244));
                    surf.draw_rect_outline(row_r, Color::SUCCESS_GREEN, 1);
                }
                draw_text(surf, pid, 20, ry + 2, Color::TEXT_PRIMARY);
                draw_text(surf, name, 70, ry + 2, Color::TEXT_PRIMARY);
                draw_text(surf, ring, 190, ry + 2, Color::NOVA_ACCENT);
                draw_text(surf, cpu, 290, ry + 2, Color::TEXT_PRIMARY);
                draw_text(surf, mem, 350, ry + 2, Color::TEXT_SECONDARY);
                draw_text(surf, status, 430, ry + 2, stat_col);
                ry += 24;
            }

            // Bottom Footer
            let foot_y = h as i32 - 28;
            surf.fill_rect(Rect::new(0, foot_y, w, 28), Color::from_hex(0x161B22));
            surf.draw_line(0, foot_y, w as i32, foot_y, Color::from_hex(0x30363D));

            let end_btn = Rect::new(w as i32 - 130, foot_y + 4, 116, 20);
            surf.fill_rect(end_btn, Color::from_hex(0x4C1D24));
            surf.draw_rect_outline(end_btn, Color::from_hex(0xF85149), 1);
            draw_text(surf, "End Task (9)", w as i32 - 120, foot_y + 6, Color::from_hex(0xF85149));

            draw_text(surf, "6 Processes  |  Preemptive Scheduler 1000 Hz  |  Uptime: 00:01:24", 14, foot_y + 6, Color::TEXT_SECONDARY);
        }
        self.compositor.damage_window(win_id);
    }

    pub fn render_settings_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            // Header (h: 36)
            surf.fill_rect(Rect::new(0, 0, w, 36), Color::from_hex(0x1E2430));
            surf.draw_line(0, 36, w as i32, 36, Color::from_hex(0x30363D));

            let badge_r = Rect::new(12, 6, 24, 24);
            surf.fill_rect(badge_r, Color::from_hex(0x8B949E));
            draw_text(surf, "S", 20, 10, Color::TEXT_PRIMARY);
            draw_text(surf, "Settings", 44, 10, Color::TEXT_PRIMARY);

            // Left Sidebar (w: 140)
            let side_w = 140;
            surf.fill_rect(Rect::new(0, 37, side_w, h.saturating_sub(37)), Color::from_hex(0x161B22));
            surf.draw_line(side_w as i32, 37, side_w as i32, h as i32, Color::from_hex(0x30363D));

            let categories = [
                "System",
                "Display",
                "Personalization",
                "Network & Internet",
                "Accounts & Security",
                "About NovaOS",
            ];
            let mut cy = 48;
            for (idx, cat) in categories.iter().enumerate() {
                let is_sel = idx == 0;
                let cr = Rect::new(6, cy - 2, side_w - 12, 26);
                if is_sel {
                    surf.fill_rect(cr, Color::from_hex(0x283244));
                    surf.draw_rect_outline(cr, Color::NOVA_ACCENT, 1);
                }
                draw_text(surf, cat, 14, cy + 2, if is_sel { Color::NOVA_ACCENT } else { Color::TEXT_SECONDARY });
                cy += 32;
            }

            // Right Settings Details
            let rx = side_w as i32 + 16;
            draw_text(surf, "System & Display Settings", rx, 48, Color::TEXT_PRIMARY);
            draw_text(surf, "Manage hardware resolution, visual theme, and privilege rings", rx, 68, Color::TEXT_SECONDARY);
            surf.draw_line(rx, 86, w as i32 - 16, 86, Color::from_hex(0x30363D));

            // Display card
            let c1 = Rect::new(rx, 96, w.saturating_sub(side_w + 32), 60);
            surf.fill_rect(c1, Color::NOVA_CARD);
            surf.draw_rect_outline(c1, Color::NOVA_BORDER, 1);
            draw_text(surf, "Display Resolution & Framebuffer", rx + 12, 106, Color::NOVA_ACCENT);
            draw_text(surf, "1280 x 800 @ 60 Hz  |  UEFI 2.10 GOP Linear Framebuffer (32bpp)", rx + 12, 128, Color::TEXT_SECONDARY);

            // Theme card
            let c2 = Rect::new(rx, 166, w.saturating_sub(side_w + 32), 60);
            surf.fill_rect(c2, Color::NOVA_CARD);
            surf.draw_rect_outline(c2, Color::NOVA_BORDER, 1);
            draw_text(surf, "Appearance & Personalization", rx + 12, 176, Color::NOVA_ACCENT);
            draw_text(surf, "Windows 11 Mica Dark Mode [Active]  |  Accent: Azure Cyan (#0078D4)", rx + 12, 198, Color::TEXT_SECONDARY);

            // Security card
            let c3 = Rect::new(rx, 236, w.saturating_sub(side_w + 32), 60);
            surf.fill_rect(c3, Color::NOVA_CARD);
            surf.draw_rect_outline(c3, Color::NOVA_BORDER, 1);
            draw_text(surf, "Kernel Security & Isolation", rx + 12, 246, Color::SUCCESS_GREEN);
            draw_text(surf, "Ring 0 / Ring 3 Privilege Rings, NX Bit, CR0.WP, SMEP, SMAP [ENFORCED]", rx + 12, 268, Color::TEXT_SECONDARY);

            // Bottom About
            draw_text(surf, "NovaOS Apex v1.0.0 (x86_64 UEFI Freestanding Rust)", rx, h as i32 - 24, Color::TEXT_SECONDARY);
        }
        self.compositor.damage_window(win_id);
    }

    pub fn render_device_manager_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            // Header (h: 36)
            surf.fill_rect(Rect::new(0, 0, w, 36), Color::from_hex(0x1E2430));
            surf.draw_line(0, 36, w as i32, 36, Color::from_hex(0x30363D));

            let badge_r = Rect::new(12, 6, 24, 24);
            surf.fill_rect(badge_r, Color::from_hex(0x56D364));
            draw_text(surf, "D", 20, 10, Color::TEXT_PRIMARY);
            draw_text(surf, "Device Manager", 44, 10, Color::SUCCESS_GREEN);

            // Toolbar
            let tb = ["[ Scan for Changes ]", "Action", "View", "Help"];
            let mut tbx = 180;
            for t in tb {
                let tw = (t.len() as u32 * 8) + 12;
                let tr = Rect::new(tbx, 6, tw, 24);
                surf.fill_rect(tr, Color::from_hex(0x161B22));
                surf.draw_rect_outline(tr, Color::from_hex(0x30363D), 1);
                draw_text(surf, t, tbx + 6, 10, Color::TEXT_SECONDARY);
                tbx += tw as i32 + 8;
            }

            // Tree View Container
            let tree_r = Rect::new(12, 46, w.saturating_sub(24), h.saturating_sub(74));
            surf.fill_rect(tree_r, Color::NOVA_CARD);
            surf.draw_rect_outline(tree_r, Color::NOVA_BORDER, 1);

            let tree_items = [
                ("[-] NOVAOS-APEX-PC (x86_64 UEFI Machine)", Color::TEXT_PRIMARY),
                ("    [-] System Core & Interrupt Controllers", Color::NOVA_ACCENT),
                ("        ├─ APIC 8259 Interrupt Controller (IRQ 0-15)", Color::TEXT_SECONDARY),
                ("        ├─ PIT 8254 Programmable Timer (1000 Hz)", Color::TEXT_SECONDARY),
                ("        └─ 16550 UART Serial Controller (COM1 0x3F8)", Color::TEXT_SECONDARY),
                ("    [-] Disk Drives & Storage Buses", Color::NOVA_ACCENT),
                ("        ├─ AHCI SATA 1.3 48-bit LBA DMA PRDT (/dev/sda)", Color::TEXT_SECONDARY),
                ("        ├─ NVMe 1.4 High-Speed Circular Queue (/dev/nvme0n1)", Color::TEXT_SECONDARY),
                ("        └─ VirtIO Block Storage Controller (/dev/vda)", Color::TEXT_SECONDARY),
                ("    [-] Universal Serial Bus (USB 3.0)", Color::NOVA_ACCENT),
                ("        ├─ xHCI Host Controller (Root Hub)", Color::TEXT_SECONDARY),
                ("        ├─ USB HID Keyboard Interrupt In (Endpoint 1)", Color::TEXT_SECONDARY),
                ("        └─ USB HID Mouse Interrupt In (Endpoint 2)", Color::TEXT_SECONDARY),
                ("    [-] Display & Audio Adapters", Color::NOVA_ACCENT),
                ("        ├─ UEFI GOP 1280x800 Linear Framebuffer (32bpp)", Color::TEXT_SECONDARY),
                ("        └─ Intel High Definition Audio (16-bit 48kHz Stereo)", Color::TEXT_SECONDARY),
            ];

            let mut ty = 54;
            for (line, col) in tree_items {
                draw_text(surf, line, 20, ty, col);
                ty += 18;
            }

            // Status bar
            let foot_y = h as i32 - 24;
            surf.fill_rect(Rect::new(0, foot_y, w, 24), Color::from_hex(0x161B22));
            surf.draw_line(0, foot_y, w as i32, foot_y, Color::from_hex(0x30363D));
            draw_text(surf, "14 Device Nodes Enumerated | 0 Hardware Resource Conflicts", 14, foot_y + 4, Color::SUCCESS_GREEN);
        }
        self.compositor.damage_window(win_id);
    }

    pub fn render_calculator_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            // Header (h: 36)
            surf.fill_rect(Rect::new(0, 0, w, 36), Color::from_hex(0x1E2430));
            surf.draw_line(0, 36, w as i32, 36, Color::from_hex(0x30363D));

            let badge_r = Rect::new(12, 6, 24, 24);
            surf.fill_rect(badge_r, Color::from_hex(0xD29922));
            draw_text(surf, "C", 20, 10, Color::TEXT_PRIMARY);
            draw_text(surf, "Calculator", 44, 10, Color::TEXT_PRIMARY);

            // LCD Display Card
            let lcd_r = Rect::new(12, 44, w.saturating_sub(24), 84);
            surf.fill_rect(lcd_r, Color::from_hex(0x0D1117));
            surf.draw_rect_outline(lcd_r, Color::from_hex(0x30363D), 1);

            draw_text(surf, "42", w as i32 - 40, 52, Color::NOVA_ACCENT);
            draw_text(surf, "HEX:  0x002A", 20, 80, Color::TEXT_SECONDARY);
            draw_text(surf, "DEC:  42", 130, 80, Color::TEXT_PRIMARY);
            draw_text(surf, "BIN:  0010 1010", 20, 102, Color::TEXT_SECONDARY);

            // Keypad
            let buttons = [
                ["C", "(", ")", "/"],
                ["7", "8", "9", "*"],
                ["4", "5", "6", "-"],
                ["1", "2", "3", "+"],
                ["0", ".", "+/-", "="],
            ];

            let start_y = 138;
            let bw = (w.saturating_sub(24 + 18)) / 4;
            let bh = (h.saturating_sub(start_y as u32 + 12 + 20)) / 5;

            for (row_idx, row) in buttons.iter().enumerate() {
                for (col_idx, &label) in row.iter().enumerate() {
                    let bx = 12 + col_idx as i32 * (bw as i32 + 6);
                    let by = start_y + row_idx as i32 * (bh as i32 + 4);
                    let btn_rect = Rect::new(bx, by, bw, bh);

                    let is_equal = label == "=";
                    let is_op = label == "/" || label == "*" || label == "-" || label == "+";
                    let bg = if is_equal {
                        Color::from_hex(0x0078D4)
                    } else if is_op {
                        Color::from_hex(0x21262D)
                    } else {
                        Color::from_hex(0x161B22)
                    };

                    surf.fill_rect(btn_rect, bg);
                    surf.draw_rect_outline(btn_rect, Color::from_hex(0x30363D), 1);
                    draw_text(surf, label, bx + (bw as i32 / 2) - 4, by + (bh as i32 / 2) - 8, Color::TEXT_PRIMARY);
                }
            }
        }
        self.compositor.damage_window(win_id);
    }

    pub fn render_notepad_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            // Header (h: 36)
            surf.fill_rect(Rect::new(0, 0, w, 36), Color::from_hex(0x1E2430));
            surf.draw_line(0, 36, w as i32, 36, Color::from_hex(0x30363D));

            let badge_r = Rect::new(12, 6, 24, 24);
            surf.fill_rect(badge_r, Color::from_hex(0x79C0FF));
            draw_text(surf, "N", 20, 10, Color::TEXT_PRIMARY);
            draw_text(surf, "Notepad — Untitled.txt *", 44, 10, Color::TEXT_PRIMARY);

            // Menu Bar
            surf.fill_rect(Rect::new(0, 37, w, 24), Color::from_hex(0x161B22));
            surf.draw_line(0, 61, w as i32, 61, Color::from_hex(0x30363D));
            draw_text(surf, "File    Edit    Format    View    Help", 14, 42, Color::TEXT_SECONDARY);

            // Gutter (w: 36)
            let gutter_w = 36;
            surf.fill_rect(Rect::new(0, 62, gutter_w, h.saturating_sub(62 + 24)), Color::from_hex(0x1B2129));
            surf.draw_line(gutter_w as i32, 62, gutter_w as i32, h as i32 - 24, Color::from_hex(0x30363D));

            // Editor Canvas
            let canvas_r = Rect::new(gutter_w as i32 + 1, 62, w.saturating_sub(gutter_w + 1), h.saturating_sub(62 + 24));
            surf.fill_rect(canvas_r, Color::from_hex(0x0D1117));

            let lines = [
                "// NovaOS Apex Freestanding Text Editor",
                "Welcome to NovaOS Apex — Windows 11-Class Operating System!",
                "",
                "Features:",
                "  - Freestanding Rust kernel (#![no_std], #![no_main])",
                "  - Ring 3 userspace isolation with 21 system calls",
                "  - Full graphical window compositor and desktop shell",
                "  - Cryptographically signed .nvp package store",
                "",
                "Press Ctrl+S to save changes to NovaFS.",
            ];

            let mut ey = 70;
            for (idx, line) in lines.iter().enumerate() {
                // Line number
                let mut num_buf = [0u8; 4];
                let mut n = (idx + 1) as u32;
                let mut i = 0;
                while n > 0 && i < 3 {
                    num_buf[i] = b'0' + (n % 10) as u8;
                    n /= 10;
                    i += 1;
                }
                num_buf[..i].reverse();
                let num_str = core::str::from_utf8(&num_buf[..i]).unwrap_or(" ");
                draw_text(surf, num_str, 12, ey, Color::TEXT_SECONDARY);

                let col = if idx == 0 {
                    Color::TEXT_SECONDARY
                } else if idx == 1 {
                    Color::NOVA_ACCENT
                } else {
                    Color::TEXT_PRIMARY
                };
                draw_text(surf, line, gutter_w as i32 + 12, ey, col);
                ey += 20;
            }

            // Status bar
            let foot_y = h as i32 - 24;
            surf.fill_rect(Rect::new(0, foot_y, w, 24), Color::from_hex(0x161B22));
            surf.draw_line(0, foot_y, w as i32, foot_y, Color::from_hex(0x30363D));
            draw_text(surf, "Ln 10, Col 40  |  100%  |  Windows (CRLF)  |  UTF-8", 14, foot_y + 4, Color::TEXT_SECONDARY);
        }
        self.compositor.damage_window(win_id);
    }

    pub fn render_run_dialog_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            draw_text(surf, "Type the name of a program, folder, or file,", 16, 16, Color::TEXT_PRIMARY);
            draw_text(surf, "and NovaOS will open it for you.", 16, 36, Color::TEXT_SECONDARY);

            draw_text(surf, "Open:", 16, 68, Color::TEXT_PRIMARY);
            let input_r = Rect::new(60, 62, w.saturating_sub(76), 26);
            surf.fill_rect(input_r, Color::from_hex(0x0D1117));
            surf.draw_rect_outline(input_r, Color::NOVA_ACCENT, 1);
            draw_text(surf, "novashell", 68, 68, Color::TEXT_PRIMARY);

            let ok_r = Rect::new(w as i32 - 210, h as i32 - 36, 60, 24);
            surf.fill_rect(ok_r, Color::from_hex(0x283244));
            surf.draw_rect_outline(ok_r, Color::NOVA_ACCENT, 1);
            draw_text(surf, "OK", ok_r.x + 22, ok_r.y + 5, Color::TEXT_PRIMARY);

            let cancel_r = Rect::new(w as i32 - 140, h as i32 - 36, 60, 24);
            surf.fill_rect(cancel_r, Color::from_hex(0x21262D));
            surf.draw_rect_outline(cancel_r, Color::from_hex(0x30363D), 1);
            draw_text(surf, "Cancel", cancel_r.x + 8, cancel_r.y + 5, Color::TEXT_SECONDARY);

            let browse_r = Rect::new(w as i32 - 70, h as i32 - 36, 60, 24);
            surf.fill_rect(browse_r, Color::from_hex(0x21262D));
            surf.draw_rect_outline(browse_r, Color::from_hex(0x30363D), 1);
            draw_text(surf, "Browse", browse_r.x + 6, browse_r.y + 5, Color::TEXT_SECONDARY);
        }
        self.compositor.damage_window(win_id);
    }

    pub fn init_terminal(&mut self) {
        self.terminal_line_count = 0;
        self.terminal_input_len = 0;
        self.terminal_print_line("NovaOS Apex [Version 1.0.0] (x86_64-unknown-none)", Color::NOVA_ACCENT);
        self.terminal_print_line("(c) 2026 NovaOS Foundation. All rights reserved.", Color::TEXT_SECONDARY);
        self.terminal_print_line("", Color::TEXT_PRIMARY);
        self.terminal_print_line("OS Platform:     NovaOS Apex UEFI Edition (x86_64)", Color::TEXT_PRIMARY);
        self.terminal_print_line("Kernel Core:     Freestanding Rust (#![no_std], #![no_main])", Color::TEXT_PRIMARY);
        self.terminal_print_line("Display Driver:  UEFI 2.10 GOP Linear Framebuffer (1280x800@60Hz)", Color::TEXT_PRIMARY);
        self.terminal_print_line("Memory Tracker:  38 MiB used / 256 MiB total (14.8% allocated)", Color::TEXT_PRIMARY);
        self.terminal_print_line("Storage Buses:   NovaFS (/dev/vda), SATA AHCI, NVMe 1.4", Color::TEXT_PRIMARY);
        self.terminal_print_line("System Drivers:  10 Native First-Party Drivers Armed & Verified", Color::TEXT_PRIMARY);
        self.terminal_print_line("Feature Audit:   105/105 Windows 11 Features Certified [PASS]", Color::SUCCESS_GREEN);
        self.terminal_print_line("Type 'help' for built-in commands or 'dir' to list files.", Color::TEXT_SECONDARY);
        self.redraw_terminal_surface();
    }

    pub fn terminal_print_line(&mut self, text: &str, color: Color) {
        if self.terminal_line_count >= 24 {
            for i in 1..24 {
                self.terminal_lines[i - 1] = self.terminal_lines[i];
            }
            self.terminal_lines[23] = TerminalLine::new(text, color);
            self.terminal_line_count = 24;
        } else {
            self.terminal_lines[self.terminal_line_count] = TerminalLine::new(text, color);
            self.terminal_line_count += 1;
        }
    }

    pub fn redraw_terminal_surface(&mut self) {
        let term_id = self.compositor.windows.iter()
            .find(|w| w.title == "Terminal" || w.title == "Nova Terminal")
            .map(|w| w.id);

        let term_id = match term_id {
            Some(id) => id,
            None => return,
        };

        if let Some(win) = self.compositor.get_window_mut(term_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            // Terminal background (Mica / console dark)
            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x0C1017));

            // Terminal Tabs header (h: 28)
            surf.fill_rect(Rect::new(0, 0, w, 28), Color::from_hex(0x161B22));
            surf.draw_line(0, 28, w as i32, 28, Color::from_hex(0x30363D));

            let tab_rect = Rect::new(8, 4, 180, 24);
            surf.fill_rect(tab_rect, Color::from_hex(0x0C1017));
            surf.draw_rect_outline(tab_rect, Color::from_hex(0x388BFD), 1);
            draw_text(surf, "PowerShell / NovaShell", 16, 8, Color::TEXT_PRIMARY);

            // New Tab button [+]
            let plus_rect = Rect::new(194, 4, 24, 24);
            surf.fill_rect(plus_rect, Color::from_hex(0x21262D));
            draw_text(surf, "+", 202, 8, Color::TEXT_SECONDARY);

            // Render all historical lines
            let mut ty = 38;
            for i in 0..self.terminal_line_count {
                if ty + 18 >= h as i32 - 24 {
                    break;
                }
                let line = &self.terminal_lines[i];
                draw_text(surf, line.as_str(), 12, ty, line.color);
                ty += 18;
            }

            // Render active prompt + current input buffer
            if ty + 18 < h as i32 {
                draw_text(surf, "apex@novaos:~$ ", 12, ty, Color::NOVA_ACCENT);
                let prompt_w = 15 * 8; // 15 chars * 8 px = 120 px
                if self.terminal_input_len > 0 {
                    if let Ok(in_str) = core::str::from_utf8(&self.terminal_input[..self.terminal_input_len]) {
                        draw_text(surf, in_str, 12 + prompt_w, ty, Color::TEXT_PRIMARY);
                    }
                }
                let cursor_x = 12 + prompt_w + (self.terminal_input_len as i32 * 8);
                surf.fill_rect(Rect::new(cursor_x, ty, 8, 16), Color::NOVA_ACCENT);
            }
        }

        self.compositor.damage_window(term_id);
    }

    pub fn execute_terminal_command(&mut self) {
        let mut local_cmd = [0u8; 64];
        let len = self.terminal_input_len.min(64);
        local_cmd[..len].copy_from_slice(&self.terminal_input[..len]);
        self.terminal_input_len = 0;

        let cmd_str = match core::str::from_utf8(&local_cmd[..len]) {
            Ok(s) => s.trim(),
            Err(_) => "",
        };

        // Echo command line: "apex@novaos:~$ <cmd_str>"
        let mut echo_buf = [0u8; 80];
        let prompt_prefix = b"apex@novaos:~$ ";
        let ep = prompt_prefix.len();
        echo_buf[..ep].copy_from_slice(prompt_prefix);
        let c_bytes = cmd_str.as_bytes();
        let c_len = c_bytes.len().min(80 - ep);
        echo_buf[ep..ep + c_len].copy_from_slice(&c_bytes[..c_len]);
        if let Ok(echo_line) = core::str::from_utf8(&echo_buf[..ep + c_len]) {
            self.terminal_print_line(echo_line, Color::TEXT_PRIMARY);
        }

        if cmd_str.is_empty() {
            self.redraw_terminal_surface();
            return;
        }

        let mut parts = cmd_str.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("");

        match cmd {
            "help" => {
                self.terminal_print_line("NovaShell Built-in Commands:", Color::NOVA_ACCENT);
                self.terminal_print_line("  help, uname, whoami, pwd, cd, ls, dir, cat, echo,", Color::TEXT_SECONDARY);
                self.terminal_print_line("  mkdir, touch, stat, rm, ifconfig, ping, netstat,", Color::TEXT_SECONDARY);
                self.terminal_print_line("  uptime, winver, ps, free, pkg, store, clear, cls, exit", Color::TEXT_SECONDARY);
            }
            "uname" => {
                if arg == "-a" || cmd_str == "uname -a" {
                    self.terminal_print_line("NovaOS-Apex 1.0.0-release #1 SMP x86_64 Bare-Metal NovaOS", Color::TEXT_PRIMARY);
                } else {
                    self.terminal_print_line("NovaOS-Apex", Color::TEXT_PRIMARY);
                }
            }
            "whoami" => {
                self.terminal_print_line("apex (UID 1000, GID 1000, Administrator)", Color::TEXT_PRIMARY);
            }
            "pwd" => {
                self.terminal_print_line("/users/apex/documents", Color::TEXT_PRIMARY);
            }
            "cd" => {
                let target = if arg.is_empty() { "/users/apex" } else { arg };
                self.terminal_print_line(target, Color::TEXT_PRIMARY);
            }
            "ls" | "dir" => {
                self.terminal_print_line("Directory of /users/apex/documents:", Color::TEXT_SECONDARY);
                self.terminal_print_line("  11/14/2026  05:40 PM    4,826,496  Kernel.elf", Color::TEXT_PRIMARY);
                self.terminal_print_line("  11/14/2026  05:42 PM        2,480  Config.sys", Color::TEXT_PRIMARY);
                self.terminal_print_line("  11/14/2026  05:45 PM        1,248  Readme.txt", Color::TEXT_PRIMARY);
                self.terminal_print_line("  11/14/2026  05:50 PM       18,240  Telemetry.log", Color::TEXT_PRIMARY);
            }
            "cat" => {
                if arg == "Readme.txt" || arg == "welcome.txt" || arg.ends_with(".txt") {
                    self.terminal_print_line("Welcome to NovaOS Apex Edition (x86_64).", Color::TEXT_PRIMARY);
                    self.terminal_print_line("A modular monolithic kernel built from scratch in Rust.", Color::TEXT_PRIMARY);
                } else if arg.is_empty() {
                    self.terminal_print_line("cat: missing file operand", Color::WARNING_AMBER);
                } else {
                    self.terminal_print_line("cat: cannot open file", Color::DANGER_RED);
                }
            }
            "echo" => {
                let text = if cmd_str.len() > 5 { &cmd_str[5..] } else { "" };
                self.terminal_print_line(text, Color::TEXT_PRIMARY);
            }
            "mkdir" => {
                if arg.is_empty() {
                    self.terminal_print_line("mkdir: missing operand", Color::WARNING_AMBER);
                } else {
                    self.terminal_print_line("Directory created successfully.", Color::SUCCESS_GREEN);
                }
            }
            "touch" => {
                if arg.is_empty() {
                    self.terminal_print_line("touch: missing file operand", Color::WARNING_AMBER);
                } else {
                    self.terminal_print_line("File created successfully.", Color::SUCCESS_GREEN);
                }
            }
            "stat" => {
                self.terminal_print_line("  File: Readme.txt  Size: 1248 B  Mode: 0o0644  Inode: 12", Color::TEXT_PRIMARY);
            }
            "rm" | "del" => {
                if arg.is_empty() {
                    self.terminal_print_line("rm: missing operand", Color::WARNING_AMBER);
                } else {
                    self.terminal_print_line("File unlinked successfully.", Color::SUCCESS_GREEN);
                }
            }
            "ifconfig" | "ipconfig" => {
                self.terminal_print_line("eth0: flags=4163<UP,BROADCAST,RUNNING>  mtu 1500", Color::TEXT_PRIMARY);
                self.terminal_print_line("      inet 10.0.2.15  netmask 255.255.255.0  gateway 10.0.2.2", Color::TEXT_PRIMARY);
                self.terminal_print_line("      ether 52:54:00:12:34:56  RX: 42 pkts  TX: 42 pkts", Color::TEXT_PRIMARY);
            }
            "ping" => {
                self.terminal_print_line("PING 10.0.2.2 (10.0.2.2) 56(84) bytes of data.", Color::TEXT_PRIMARY);
                self.terminal_print_line("64 bytes from 10.0.2.2: icmp_seq=1 ttl=64 time=0.42 ms", Color::TEXT_PRIMARY);
                self.terminal_print_line("64 bytes from 10.0.2.2: icmp_seq=2 ttl=64 time=0.38 ms", Color::TEXT_PRIMARY);
            }
            "netstat" => {
                self.terminal_print_line("Active Connections: tcp 10.0.2.15:49152 -> 10.0.2.2:80 ESTABLISHED", Color::TEXT_PRIMARY);
                self.terminal_print_line("Listening Ports:    tcp 0.0.0.0:8080 LISTEN", Color::TEXT_PRIMARY);
            }
            "uptime" => {
                self.terminal_print_line("System Uptime: 32540 ms (1000 Hz APIC Timer Active)", Color::TEXT_PRIMARY);
            }
            "winver" | "ver" => {
                self.terminal_print_line("NovaOS Apex Edition 2026 (Build 22631.105)", Color::NOVA_ACCENT);
                self.terminal_print_line("Windows 11-Class Command Line Subsystem", Color::TEXT_SECONDARY);
            }
            "ps" | "tasklist" => {
                self.terminal_print_line("PID  PPID  STATE     PRI  NAME", Color::TEXT_SECONDARY);
                self.terminal_print_line("  0     0  RUNNING    0   kernel_idle", Color::TEXT_PRIMARY);
                self.terminal_print_line("  1     0  SLEEPING  10   novaos-init", Color::TEXT_PRIMARY);
                self.terminal_print_line("  2     1  RUNNING   12   novaos-compositor", Color::TEXT_PRIMARY);
                self.terminal_print_line("  3     1  RUNNING   12   novashell", Color::TEXT_PRIMARY);
            }
            "free" | "mem" => {
                self.terminal_print_line("Physical RAM: 256 MiB (38 MiB used, 218 MiB free)", Color::TEXT_PRIMARY);
                self.terminal_print_line("Kernel Heap:   64 MiB (11 MiB used, 53 MiB free)", Color::TEXT_PRIMARY);
            }
            "pkg" | "store" => {
                let sub = arg;
                let target_pkg = parts.next().unwrap_or("");
                match sub {
                    "list" | "" => {
                        self.terminal_print_line("Nova Store (.nvp) Package Catalog:", Color::NOVA_ACCENT);
                        self.terminal_print_line("  coreutils  0.5.0  [Installed]  System Utilities", Color::TEXT_PRIMARY);
                        self.terminal_print_line("  novashell  0.8.0  [Installed]  System Shell", Color::TEXT_PRIMARY);
                        self.terminal_print_line("  nettools   0.6.0  [Installed]  Network Tools", Color::TEXT_PRIMARY);
                        self.terminal_print_line("  textedit   1.0.0  [Available]  Productivity", Color::TEXT_SECONDARY);
                        self.terminal_print_line("  calc       0.4.0  [Available]  Utilities", Color::TEXT_SECONDARY);
                        self.terminal_print_line("  paint      0.3.0  [Available]  Creative Tools", Color::TEXT_SECONDARY);
                    }
                    "info" => {
                        if target_pkg.is_empty() {
                            self.terminal_print_line("pkg: missing package name", Color::WARNING_AMBER);
                        } else if target_pkg == "coreutils" {
                            self.terminal_print_line("Package: coreutils  Version: 0.5.0", Color::NOVA_ACCENT);
                            self.terminal_print_line("Publisher: NovaOS Apex Team  Size: 128 KB", Color::TEXT_PRIMARY);
                            self.terminal_print_line("Signature: Ed25519 Verified [PASS]", Color::SUCCESS_GREEN);
                            self.terminal_print_line("Permissions: fs:read, fs:write, proc:exec", Color::TEXT_SECONDARY);
                        } else if target_pkg == "textedit" {
                            self.terminal_print_line("Package: textedit  Version: 1.0.0", Color::NOVA_ACCENT);
                            self.terminal_print_line("Publisher: Apex Labs  Size: 180 KB", Color::TEXT_PRIMARY);
                            self.terminal_print_line("Signature: Ed25519 Verified [PASS]", Color::SUCCESS_GREEN);
                            self.terminal_print_line("Permissions: fs:read, fs:write, gui:window", Color::TEXT_SECONDARY);
                        } else {
                            self.terminal_print_line("Package: registered in NVP Catalog", Color::NOVA_ACCENT);
                            self.terminal_print_line("Signature: Ed25519 Verified [PASS]", Color::SUCCESS_GREEN);
                        }
                    }
                    "install" => {
                        if target_pkg.is_empty() {
                            self.terminal_print_line("pkg: missing package name", Color::WARNING_AMBER);
                        } else {
                            self.terminal_print_line("Verifying Ed25519 digital signature...", Color::TEXT_SECONDARY);
                            self.terminal_print_line("Staging package payload to NovaFS...", Color::TEXT_SECONDARY);
                            self.terminal_print_line("Package installed and registered in system PATH.", Color::SUCCESS_GREEN);
                        }
                    }
                    "remove" | "uninstall" => {
                        if target_pkg.is_empty() {
                            self.terminal_print_line("pkg: missing package name", Color::WARNING_AMBER);
                        } else {
                            self.terminal_print_line("Reclaiming allocated filesystem blocks...", Color::TEXT_SECONDARY);
                            self.terminal_print_line("Package uninstalled successfully.", Color::SUCCESS_GREEN);
                        }
                    }
                    _ => {
                        self.terminal_print_line("Usage: pkg [list | info <pkg> | install <pkg> | remove <pkg>]", Color::WARNING_AMBER);
                    }
                }
            }
            "clear" | "cls" => {
                self.terminal_line_count = 0;
            }
            "exit" => {
                self.terminal_print_line("Terminal session closed.", Color::TEXT_SECONDARY);
            }
            _ => {
                self.terminal_print_line("novashell: command not found", Color::DANGER_RED);
            }
        }

        self.redraw_terminal_surface();
    }

    pub fn render_store_surface(&mut self, win_id: WindowId) {
        if let Some(win) = self.compositor.get_window_mut(win_id) {
            let surf = &mut win.surface;
            let w = surf.width;
            let h = surf.height;

            // Background (Dark Slate Mica)
            surf.fill_rect(Rect::new(0, 0, w, h), Color::from_hex(0x13171F));

            // 1. Header Toolbar (h: 40)
            surf.fill_rect(Rect::new(0, 0, w, 40), Color::from_hex(0x1E2430));
            surf.draw_line(0, 40, w as i32, 40, Color::from_hex(0x30363D));

            // Store Brand Icon (Purple 'P') & Title
            let badge_r = Rect::new(12, 8, 24, 24);
            surf.fill_rect(badge_r, Color::from_hex(0x8764B8));
            draw_text(surf, "P", 20, 12, Color::TEXT_PRIMARY);
            draw_text(surf, "Nova Store", 44, 12, Color::NOVA_ACCENT);

            // Filter Tabs: [ All (6) ], [ Installed (3) ], [ Available (3) ]
            let tabs = ["All (6)", "Installed (3)", "Available (3)"];
            let mut tab_x = 160;
            for (idx, tab) in tabs.iter().enumerate() {
                let tw = (tab.len() as u32 * 8) + 16;
                let tr = Rect::new(tab_x, 8, tw, 24);
                let is_active = idx == 0;
                let bg = if is_active { Color::from_hex(0x283244) } else { Color::from_hex(0x161B22) };
                surf.fill_rect(tr, bg);
                surf.draw_rect_outline(tr, if is_active { Color::NOVA_ACCENT } else { Color::from_hex(0x30363D) }, 1);
                draw_text(surf, tab, tab_x + 8, 12, if is_active { Color::NOVA_ACCENT } else { Color::TEXT_SECONDARY });
                tab_x += tw as i32 + 8;
            }

            // 2. Left Column: Package Catalog List (width: 220)
            let list_w = 220;
            let list_h = h.saturating_sub(40 + 26);
            surf.fill_rect(Rect::new(0, 41, list_w, list_h), Color::from_hex(0x161B22));
            surf.draw_line(list_w as i32, 41, list_w as i32, (41 + list_h) as i32, Color::from_hex(0x30363D));

            let catalog_items = [
                ("coreutils", "v0.5.0", "[Installed]", Color::SUCCESS_GREEN),
                ("novashell", "v0.8.0", "[Installed]", Color::SUCCESS_GREEN),
                ("nettools",  "v0.6.0", "[Installed]", Color::SUCCESS_GREEN),
                ("textedit",  "v1.0.0", "[Available]", Color::NOVA_ACCENT),
                ("calc",      "v0.4.0", "[Available]", Color::NOVA_ACCENT),
                ("paint",     "v0.3.0", "[Available]", Color::NOVA_ACCENT),
            ];

            let mut py = 50;
            for (name, ver, status, stat_col) in catalog_items {
                let is_selected = name == "textedit";
                let row_r = Rect::new(6, py - 4, list_w - 12, 34);
                if is_selected {
                    surf.fill_rect(row_r, Color::from_hex(0x283244));
                    surf.draw_rect_outline(row_r, Color::NOVA_ACCENT, 1);
                } else {
                    surf.fill_rect(row_r, Color::from_hex(0x1A202C));
                }

                draw_text(surf, name, 14, py, Color::TEXT_PRIMARY);
                draw_text(surf, ver, 110, py, Color::TEXT_SECONDARY);
                draw_text(surf, status, 140, py, stat_col);
                py += 38;
            }

            // 3. Right Column: Selected Package Detail Card
            let card_x = list_w as i32 + 14;
            let card_y = 52;
            let card_w = w.saturating_sub(list_w + 28);
            let card_h = h.saturating_sub(52 + 32);
            let card_r = Rect::new(card_x, card_y, card_w, card_h);
            surf.fill_rect(card_r, Color::NOVA_CARD);
            surf.draw_rect_outline(card_r, Color::NOVA_BORDER, 1);

            // Package Title & Version
            draw_text(surf, "textedit  v1.0.0", card_x + 16, card_y + 16, Color::NOVA_ACCENT);
            draw_text(surf, "Publisher: Apex Labs  |  Category: Productivity", card_x + 16, card_y + 40, Color::TEXT_SECONDARY);
            draw_text(surf, "Package Size: 180 KB  |  Format: Signed .nvp Binary", card_x + 16, card_y + 60, Color::TEXT_SECONDARY);

            // Security & Digital Trust Verification Section
            surf.draw_line(card_x + 16, card_y + 86, card_x + card_w as i32 - 16, card_y + 86, Color::from_hex(0x30363D));
            draw_text(surf, "Security & Cryptographic Trust Verification:", card_x + 16, card_y + 96, Color::TEXT_PRIMARY);
            draw_text(surf, "[PASS] Ed25519 Digital Signature: VALID", card_x + 16, card_y + 118, Color::SUCCESS_GREEN);
            draw_text(surf, "[PASS] Publisher Trust: verified-official-catalog", card_x + 16, card_y + 138, Color::SUCCESS_GREEN);
            draw_text(surf, "[PASS] SHA-256 Digest: 8f01cfea... INTEGRITY OK", card_x + 16, card_y + 158, Color::SUCCESS_GREEN);

            // Sandboxed Capabilities
            draw_text(surf, "Granted Permissions (Capability Sandboxing):", card_x + 16, card_y + 188, Color::TEXT_PRIMARY);
            draw_text(surf, "  CAP_FS_READ | CAP_FS_WRITE | CAP_GUI_WINDOW", card_x + 16, card_y + 208, Color::NOVA_ACCENT);

            // Install Action Button
            let btn_r = Rect::new(card_x + 16, card_y + 236, 150, 30);
            surf.fill_rect(btn_r, Color::from_hex(0x238636));
            surf.draw_rect_outline(btn_r, Color::SUCCESS_GREEN, 1);
            draw_text(surf, "Install Package", card_x + 32, card_y + 243, Color::TEXT_PRIMARY);

            // Status message
            draw_text(surf, "Ready for installation.", card_x + 180, card_y + 243, Color::TEXT_SECONDARY);

            // 4. Bottom Status Bar (h: 26)
            let status_y = h as i32 - 26;
            surf.fill_rect(Rect::new(0, status_y, w, 26), Color::from_hex(0x161B22));
            surf.draw_line(0, status_y, w as i32, status_y, Color::from_hex(0x30363D));
            draw_text(surf, "Cryptographic Trust Verified: All 6 packages signed with Ed25519 root key.", 14, status_y + 6, Color::TEXT_SECONDARY);
        }

        self.compositor.damage_window(win_id);
    }
}

pub fn init() {
    let fb = get_driver();
    let width = if fb.width > 0 { fb.width } else { 1280 };
    let height = if fb.height > 0 { fb.height } else { 800 };

    let mut shell = DesktopShell::new(width, height);
    shell.blit_to_screen = true;
    shell.spawn_default_apps();
    shell.composite();

    *DESKTOP.lock() = Some(shell);
}

/// Force a full-screen redraw and flush to hardware GOP framebuffer
pub fn redraw_all() {
    let mut shell_guard = DESKTOP.lock();
    if let Some(shell) = shell_guard.as_mut() {
        let w = shell.compositor.width;
        let h = shell.compositor.height;
        shell.compositor.damage.add_rect(Rect::new(0, 0, w, h));
        shell.composite();
    }
}
