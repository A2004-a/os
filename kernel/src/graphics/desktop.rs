//! NovaOS Apex — Desktop Shell Manager
//! Integrates Compositor, Nova Bar, Nova Launcher, Snap Assist, and Virtual Workspaces.

use super::bar::{BarAction, NovaBar};
use super::compositor::Compositor;
use super::geometry::{Point, Rect};
use super::launcher::{LauncherAction, NovaLauncher};
use super::snap::{SnapEngine, SnapPosition};
use super::surface::Surface;
use super::window::{WindowId, WindowState};
use super::workspace::WorkspaceManager;
use crate::drivers::framebuffer::get_driver;
use crate::scheduler::sync::SpinLock;
use alloc::vec::Vec;

pub static DESKTOP: SpinLock<Option<DesktopShell>> = SpinLock::new("desktop", None);

pub struct DesktopShell {
    pub compositor: Compositor,
    pub bar: NovaBar,
    pub launcher: NovaLauncher,
    pub workspaces: WorkspaceManager,
    pub active_snap_preview: SnapPosition,
    pub dragging_window: Option<WindowId>,
    pub drag_start_mouse: Point,
    pub drag_start_win_pos: Point,
    pub uptime_ms: u64,
}

impl DesktopShell {
    pub fn new(width: u32, height: u32) -> Self {
        let compositor = Compositor::new(width, height);
        let bar = NovaBar::new();
        let launcher = NovaLauncher::new();
        let workspaces = WorkspaceManager::new();

        Self {
            compositor,
            bar,
            launcher,
            workspaces,
            active_snap_preview: SnapPosition::None,
            dragging_window: None,
            drag_start_mouse: Point::ZERO,
            drag_start_win_pos: Point::ZERO,
            uptime_ms: 0,
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
            // Resize client surface to match new client rect
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

            // Re-render taskbar
            let bar_rect = self.bar.rect(self.compositor.width, self.compositor.height);
            self.compositor.damage.add_rect(bar_rect);
        }
    }

    pub fn switch_workspace(&mut self, ws_id: u8) {
        self.workspaces.set_active_workspace(ws_id, &mut self.compositor.windows, &mut self.compositor.damage);
        // Damage taskbar to update active workspace pill
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

    pub fn on_mouse_down(&mut self, x: i32, y: i32, button: u8) {
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
                BarAction::TrayClick => {}
            }
            return;
        }

        // 2. Check Nova Launcher click (if open)
        if self.launcher.visible {
            if let Some(action) = self.launcher.hit_test(x, y, self.compositor.width, self.compositor.height, self.bar.height) {
                match action {
                    LauncherAction::LaunchApp(app_name) => {
                        self.compositor.create_window(app_name, 120, 100, 480, 320, 0, 1);
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
                // Click outside launcher closes it
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
        }
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        if let Some(win_id) = self.dragging_window {
            let dx = x - self.drag_start_mouse.x;
            let dy = y - self.drag_start_mouse.y;
            let new_x = self.drag_start_win_pos.x + dx;
            let new_y = self.drag_start_win_pos.y + dy;

            self.compositor.set_window_position(win_id, new_x, new_y);

            // Detect Snap Zones during drag
            let snap_target = SnapEngine::detect_snap_zone(x, y, self.compositor.width, self.compositor.height, self.bar.height);
            if snap_target != self.active_snap_preview {
                self.active_snap_preview = snap_target;
                // Damage entire usable area to render/clear ghost preview
                let usable = self.usable_rect();
                self.compositor.damage.add_rect(usable);
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
        self.compositor.on_mouse_up(x, y, button);
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

        // 2. Render all visible windows in ascending Z-order (filtered by active workspace)
        for win in &self.compositor.windows {
            if !self.workspaces.is_window_visible(win) {
                continue;
            }
            if win.rect.intersects(&bbox) {
                win.render(&mut self.compositor.backbuffer);
            }
        }

        // 3. Render Snap Ghost Preview (if dragging window into a snap zone)
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

        // 6. Render Mouse Cursor Overlay
        self.compositor.cursor.render_to(&mut self.compositor.backbuffer);

        // 7. Flush changed backbuffer regions to hardware GOP framebuffer
        let fb = get_driver();
        if !fb.base_ptr.is_null() {
            for r in self.compositor.damage.rects() {
                self.compositor.backbuffer.blit_clipped_to_raw(fb.base_ptr, fb.stride, fb.width, fb.height, *r);
            }
        }

        // 8. Clear damage
        self.compositor.damage.clear();
    }
}

pub fn init() {
    let fb = get_driver();
    let width = if fb.width > 0 { fb.width } else { 1280 };
    let height = if fb.height > 0 { fb.height } else { 800 };

    let mut shell = DesktopShell::new(width, height);
    shell.composite();

    *DESKTOP.lock() = Some(shell);
}
