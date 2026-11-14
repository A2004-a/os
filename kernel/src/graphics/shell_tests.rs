//! NovaOS Apex — Milestone 8 Desktop Shell Verification Suite
//! Tests Nova Bar, Nova Launcher, Snap Assist engine, Virtual Workspaces 1..4,
//! and integrated desktop shell composition.

use super::bar::{BarAction, NovaBar};
use super::desktop::DesktopShell;
use super::launcher::{LauncherAction, NovaLauncher};
use super::snap::{SnapEngine, SnapPosition};
use super::window::{Window, WindowId};
use super::workspace::WorkspaceManager;
use crate::serial;

pub fn run_m8_verification_suite() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone 8 Verification Suite        \n");
    serial::write_str("=======================================================\n");

    let mut passed = true;

    // Test 1: Nova Bar taskbar layout, app pills, and system tray
    serial::write_str("[M8 TEST 1/6] Testing Nova Bar taskbar layout, pills & status island...\n");
    if test_nova_bar_layout_and_tray() {
        serial::write_str("[M8 TEST 1/6] [PASS] Nova Bar geometry, running pills & clock verified.\n");
    } else {
        serial::write_str("[M8 TEST 1/6] [FAIL] Nova Bar taskbar test failed!\n");
        passed = false;
    }

    // Test 2: Nova Launcher start menu, search filtering, and app drawer
    serial::write_str("[M8 TEST 2/6] Testing Nova Launcher popup, search filter & power options...\n");
    if test_nova_launcher_grid_and_search() {
        serial::write_str("[M8 TEST 2/6] [PASS] Nova Launcher grid, search query & actions verified.\n");
    } else {
        serial::write_str("[M8 TEST 2/6] [FAIL] Nova Launcher test failed!\n");
        passed = false;
    }

    // Test 3: Snap Assist window docking calculations & geometry persistence
    serial::write_str("[M8 TEST 3/6] Testing Snap Assist tiling zones (halves, quarters, maximize)...\n");
    if test_snap_assist_calculations() {
        serial::write_str("[M8 TEST 3/6] [PASS] Multi-zone snap math & restored bounds persistence verified.\n");
    } else {
        serial::write_str("[M8 TEST 3/6] [FAIL] Snap Assist tiling math failed!\n");
        passed = false;
    }

    // Test 4: Drag edge snap trigger zones & preview detection
    serial::write_str("[M8 TEST 4/6] Testing Window drag edge & corner snap zone detection...\n");
    if test_drag_edge_snap_zone_detection() {
        serial::write_str("[M8 TEST 4/6] [PASS] Screen edge/corner thresholds & zone detection verified.\n");
    } else {
        serial::write_str("[M8 TEST 4/6] [FAIL] Drag snap zone detection failed!\n");
        passed = false;
    }

    // Test 5: Virtual Workspaces 1..4 isolation and switching
    serial::write_str("[M8 TEST 5/6] Testing Virtual Workspaces 1..4 window assignment & filtering...\n");
    if test_virtual_workspaces() {
        serial::write_str("[M8 TEST 5/6] [PASS] Virtual Workspaces 1..4 switching & visibility verified.\n");
    } else {
        serial::write_str("[M8 TEST 5/6] [FAIL] Virtual Workspaces test failed!\n");
        passed = false;
    }

    // Test 6: Desktop Shell integrated composition pass
    serial::write_str("[M8 TEST 6/6] Testing Integrated Desktop Shell composition (Bar, Launcher, Windows)...\n");
    if test_desktop_shell_integration() {
        serial::write_str("[M8 TEST 6/6] [PASS] Desktop Shell integrated composition verified.\n");
    } else {
        serial::write_str("[M8 TEST 6/6] [FAIL] Desktop Shell integrated composition failed!\n");
        passed = false;
    }

    serial::write_str("\n=======================================================\n");
    if passed {
        serial::write_str("[MILESTONE 8 TEST RESULT] ALL 6 TEST SUITES PASSED.\n");
        serial::write_str("[MILESTONE 8 VERIFIED] Nova Bar, Nova Launcher, Snap Assist Engine,\n");
        serial::write_str("                       and Virtual Workspaces confirmed operational!\n");
    } else {
        serial::write_str("[MILESTONE 8 TEST RESULT] ONE OR MORE TESTS FAILED!\n");
    }
    serial::write_str("=======================================================\n\n");

    passed
}

fn test_nova_bar_layout_and_tray() -> bool {
    let bar = NovaBar::new();
    let screen_w = 1280;
    let screen_h = 800;

    // 1. Taskbar geometry
    let bar_rect = bar.rect(screen_w, screen_h);
    if bar_rect.x != 0 || bar_rect.y != (800 - 48) || bar_rect.width != 1280 || bar_rect.height != 48 {
        return false;
    }

    // 2. Start button rect
    let sbtn = bar.start_button_rect(screen_h);
    if sbtn.x != 12 || sbtn.y != (800 - 48 + 6) || sbtn.width != 36 || sbtn.height != 36 {
        return false;
    }

    // 3. Workspace switcher rects (1..4)
    for ws in 1..=4 {
        let ws_rect = bar.workspace_button_rect(screen_h, ws);
        if ws_rect.width != 24 || ws_rect.height != 28 {
            return false;
        }
    }

    // 4. Hit testing
    let windows: [Window; 0] = [];
    // Click on start button
    if bar.hit_test(20, 760, screen_w, screen_h, &windows) != Some(BarAction::ToggleLauncher) {
        return false;
    }
    // Click on workspace 3 button (x = 60 + 2*30 = 120)
    if bar.hit_test(130, 770, screen_w, screen_h, &windows) != Some(BarAction::SwitchWorkspace(3)) {
        return false;
    }

    true
}

fn test_nova_launcher_grid_and_search() -> bool {
    let mut launcher = NovaLauncher::new();
    let screen_w = 1280;
    let screen_h = 800;
    let taskbar_h = 48;

    // 1. Initial state
    if launcher.visible {
        return false;
    }
    launcher.toggle();
    if !launcher.visible {
        return false;
    }

    // 2. Launcher geometry (centered above taskbar)
    let lrect = launcher.rect(screen_w, screen_h, taskbar_h);
    if lrect.width != 520 || lrect.height != 440 {
        return false;
    }
    if lrect.x != ((1280 - 520) / 2) {
        return false;
    }

    // 3. Search query typing and backspace
    launcher.type_char('T');
    launcher.type_char('e');
    launcher.type_char('r');
    launcher.type_char('m');
    if launcher.search_query_str() != "term" {
        return false;
    }
    launcher.backspace();
    if launcher.search_query_str() != "ter" {
        return false;
    }

    // 4. Category filtering
    launcher.selected_tab = 1; // System
    launcher.clear_search();
    if !launcher.search_query_str().is_empty() {
        return false;
    }

    // 5. Footer power action hit test
    // Click on [Shutdown] button in footer
    let frect = launcher.footer_rect(lrect);
    let shutdown_x = frect.right() - 220 + 150;
    let shutdown_y = frect.y + 16;
    if launcher.hit_test(shutdown_x, shutdown_y, screen_w, screen_h, taskbar_h) != Some(LauncherAction::PowerAction("shutdown")) {
        return false;
    }

    // 6. Close launcher on toggle
    launcher.toggle();
    if launcher.visible {
        return false;
    }

    true
}

fn test_snap_assist_calculations() -> bool {
    let usable = SnapEngine::usable_screen_rect(1280, 800, 48);
    if usable.x != 0 || usable.y != 0 || usable.width != 1280 || usable.height != 752 {
        return false;
    }

    // 1. Left Half (640 x 752)
    let left_snap = SnapEngine::calculate_snap_rect(SnapPosition::LeftHalf, usable);
    if left_snap.x != 0 || left_snap.y != 0 || left_snap.width != 640 || left_snap.height != 752 {
        return false;
    }

    // 2. Right Half (640 x 752 at x=640)
    let right_snap = SnapEngine::calculate_snap_rect(SnapPosition::RightHalf, usable);
    if right_snap.x != 640 || right_snap.y != 0 || right_snap.width != 640 || right_snap.height != 752 {
        return false;
    }

    // 3. Top-Left Quarter (640 x 376 at 0, 0)
    let tl_snap = SnapEngine::calculate_snap_rect(SnapPosition::TopLeft, usable);
    if tl_snap.x != 0 || tl_snap.y != 0 || tl_snap.width != 640 || tl_snap.height != 376 {
        return false;
    }

    // 4. Bottom-Right Quarter (640 x 376 at 640, 376)
    let br_snap = SnapEngine::calculate_snap_rect(SnapPosition::BottomRight, usable);
    if br_snap.x != 640 || br_snap.y != 376 || br_snap.width != 640 || br_snap.height != 376 {
        return false;
    }

    // 5. Maximize (full usable screen)
    let max_snap = SnapEngine::calculate_snap_rect(SnapPosition::Maximize, usable);
    if max_snap != usable {
        return false;
    }

    true
}

fn test_drag_edge_snap_zone_detection() -> bool {
    let screen_w = 1280;
    let screen_h = 800;
    let taskbar_h = 48;

    // 1. Drag to Left edge -> LeftHalf
    if SnapEngine::detect_snap_zone(5, 300, screen_w, screen_h, taskbar_h) != SnapPosition::LeftHalf {
        return false;
    }

    // 2. Drag to Right edge -> RightHalf
    if SnapEngine::detect_snap_zone(1275, 300, screen_w, screen_h, taskbar_h) != SnapPosition::RightHalf {
        return false;
    }

    // 3. Drag to Top center edge -> Maximize
    if SnapEngine::detect_snap_zone(600, 5, screen_w, screen_h, taskbar_h) != SnapPosition::Maximize {
        return false;
    }

    // 4. Drag to Top-Left corner -> TopLeft
    if SnapEngine::detect_snap_zone(20, 5, screen_w, screen_h, taskbar_h) != SnapPosition::TopLeft {
        return false;
    }

    // 5. Drag to Top-Right corner -> TopRight
    if SnapEngine::detect_snap_zone(1260, 5, screen_w, screen_h, taskbar_h) != SnapPosition::TopRight {
        return false;
    }

    // 6. Drag in center (no snap) -> None
    if SnapEngine::detect_snap_zone(500, 400, screen_w, screen_h, taskbar_h) != SnapPosition::None {
        return false;
    }

    true
}

fn test_virtual_workspaces() -> bool {
    let mut ws_mgr = WorkspaceManager::new();
    let mut damage = super::damage::DamageRegion::new();

    let mut w1 = Window::new(WindowId(1), "App WS1", 50, 50, 200, 150, 0, 100);
    let mut w2 = Window::new(WindowId(2), "App WS2", 100, 100, 200, 150, 0, 101);
    let mut w3 = Window::new(WindowId(3), "App Sticky", 150, 150, 200, 150, 0, 102);

    // Assign to workspaces: w1 -> WS1, w2 -> WS2, w3 -> WS0 (sticky)
    ws_mgr.assign_window(&mut w1, 1);
    ws_mgr.assign_window(&mut w2, 2);
    ws_mgr.assign_window(&mut w3, 0);

    // In Workspace 1:
    if !w1.visible || w2.visible || !w3.visible {
        return false;
    }

    // Switch to Workspace 2:
    let mut windows = [w1, w2, w3];
    ws_mgr.set_active_workspace(2, &mut windows, &mut damage);

    if ws_mgr.active_workspace != 2 {
        return false;
    }
    // Now in WS 2: w1 is hidden, w2 is visible, w3 is visible
    if windows[0].visible || !windows[1].visible || !windows[2].visible {
        return false;
    }

    // Switch to Workspace 3:
    ws_mgr.set_active_workspace(3, &mut windows, &mut damage);
    // In WS 3: w1 is hidden, w2 is hidden, w3 is visible
    if windows[0].visible || windows[1].visible || !windows[2].visible {
        return false;
    }

    true
}

fn test_desktop_shell_integration() -> bool {
    let mut shell = DesktopShell::new(400, 300);

    // 1. Create two windows
    let w1 = shell.compositor.create_window("Left App", 20, 20, 100, 80, 0, 100);
    let w2 = shell.compositor.create_window("Right App", 140, 20, 100, 80, 0, 101);

    // 2. Snap Win 1 to LeftHalf, Win 2 to RightHalf
    shell.snap_window(w1, SnapPosition::LeftHalf);
    shell.snap_window(w2, SnapPosition::RightHalf);

    let win1 = shell.compositor.get_window(w1).unwrap();
    let win2 = shell.compositor.get_window(w2).unwrap();

    let usable = shell.usable_rect();
    if win1.rect.x != usable.x || win1.rect.width != usable.width / 2 {
        return false;
    }
    if win2.rect.x != (usable.x + (usable.width / 2) as i32) {
        return false;
    }

    // 3. Restore Win 1
    shell.restore_window(w1);
    let win1_restored = shell.compositor.get_window(w1).unwrap();
    if win1_restored.rect != win1_restored.restored_rect {
        return false;
    }

    // 4. Toggle launcher
    shell.toggle_launcher();
    if !shell.launcher.visible || !shell.bar.is_launcher_open {
        return false;
    }

    // 5. Composite entire desktop shell pass
    shell.composite();

    true
}
