//! NovaOS Apex — Virtual Workspaces Manager (Workspaces 1..4)
//! Multi-desktop management with window isolation and instant switching.

use super::damage::DamageRegion;
use super::window::{Window, WindowState};

pub const NUM_WORKSPACES: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub id: u8,
    pub name: &'static str,
}

pub struct WorkspaceManager {
    pub active_workspace: u8,
    pub workspaces: [Workspace; NUM_WORKSPACES as usize],
}

impl WorkspaceManager {
    pub const fn new() -> Self {
        Self {
            active_workspace: 1,
            workspaces: [
                Workspace { id: 1, name: "Workspace 1" },
                Workspace { id: 2, name: "Workspace 2" },
                Workspace { id: 3, name: "Workspace 3" },
                Workspace { id: 4, name: "Workspace 4" },
            ],
        }
    }

    pub fn set_active_workspace(&mut self, new_id: u8, windows: &mut [Window], damage: &mut DamageRegion) {
        if new_id < 1 || new_id > NUM_WORKSPACES || new_id == self.active_workspace {
            return;
        }

        self.active_workspace = new_id;

        for win in windows.iter_mut() {
            let should_be_visible = (win.workspace == 0 || win.workspace == new_id) && win.state != WindowState::Minimized;
            if win.visible != should_be_visible {
                win.visible = should_be_visible;
                damage.add_rect(win.rect);
            }
        }
    }

    pub fn assign_window(&self, win: &mut Window, workspace_id: u8) {
        let ws = workspace_id.clamp(0, NUM_WORKSPACES);
        win.workspace = ws;
        win.visible = (ws == 0 || ws == self.active_workspace) && win.state != WindowState::Minimized;
    }

    pub fn is_window_visible(&self, win: &Window) -> bool {
        (win.workspace == 0 || win.workspace == self.active_workspace) && win.state != WindowState::Minimized
    }
}
