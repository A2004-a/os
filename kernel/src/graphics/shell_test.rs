//! NovaOS Zero — Milestone M8 Automated Verification Suite
//! Validates Nova Bar, Nova Launcher, Snap Assist Geometry, Virtual Workspaces,
//! Notification Center, and Desktop Compositing Pipeline.

use crate::graphics::compositor::{Color, Rect, Surface};
use crate::graphics::desktop_shell::{
    DesktopShell, NotificationLevel, SnapState, MAX_NOTIFICATIONS, MAX_WORKSPACES, TASKBAR_HEIGHT,
};
use crate::graphics::window_server::{Window, WindowServer, TITLEBAR_HEIGHT};
use crate::serial_println;

pub fn run_shell_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M8 Automated Verification");
    serial_println!("=======================================================");

    test_shell_initialization();
    test_snap_assist_geometry();
    test_virtual_workspaces_isolation();
    test_notification_center();
    test_desktop_shell_rendering();

    serial_println!("[M8 SUCCESS] All Milestone M8 Verification Tests Passed Cleanly!\n");
}

fn test_shell_initialization() {
    serial_println!("[TEST 1/5] Desktop Shell Initialization & Default Geometry...");

    let shell = DesktopShell::new(1024, 768);
    assert_eq!(shell.screen_width, 1024);
    assert_eq!(shell.screen_height, 768);
    assert_eq!(shell.active_workspace, 0);
    assert!(!shell.launcher_open);
    assert_eq!(shell.window_workspaces, [0; 8]);
    assert_eq!(TASKBAR_HEIGHT, 40);
    assert_eq!(MAX_WORKSPACES, 4);
    assert_eq!(MAX_NOTIFICATIONS, 4);

    serial_println!("  -> Desktop shell initialization and default parameters verified.");
}

fn test_snap_assist_geometry() {
    serial_println!("[TEST 2/5] Snap Assist Layout Calculations & Window Resizing...");

    let shell = DesktopShell::new(1024, 768);
    let work_h = 768 - TASKBAR_HEIGHT; // 728
    let half_w = 512;
    let half_h = work_h / 2; // 364

    // Left Half
    let left = shell.calculate_snap_rect(SnapState::LeftHalf);
    assert_eq!(left, Rect::new(0, 0, half_w, work_h));

    // Right Half
    let right = shell.calculate_snap_rect(SnapState::RightHalf);
    assert_eq!(right, Rect::new(half_w as i32, 0, half_w, work_h));

    // Maximized
    let max = shell.calculate_snap_rect(SnapState::Maximized);
    assert_eq!(max, Rect::new(0, 0, 1024, work_h));

    // Quarters
    let tl = shell.calculate_snap_rect(SnapState::TopLeftQuarter);
    assert_eq!(tl, Rect::new(0, 0, half_w, half_h));

    let tr = shell.calculate_snap_rect(SnapState::TopRightQuarter);
    assert_eq!(tr, Rect::new(half_w as i32, 0, half_w, half_h));

    let bl = shell.calculate_snap_rect(SnapState::BottomLeftQuarter);
    assert_eq!(bl, Rect::new(0, half_h as i32, half_w, half_h));

    let br = shell.calculate_snap_rect(SnapState::BottomRightQuarter);
    assert_eq!(br, Rect::new(half_w as i32, half_h as i32, half_w, half_h));

    // Floating / None
    let none = shell.calculate_snap_rect(SnapState::None);
    assert_eq!(none, Rect::new(50, 50, 400, 300));

    // Snap Window Application
    let mut win = Window::new(1, "Terminal", 20, 20, 200, 150);
    shell.snap_window(&mut win, SnapState::LeftHalf);
    assert_eq!(win.rect, Rect::new(0, 0, half_w, work_h));
    assert_eq!(win.surface.width, half_w);
    assert_eq!(win.surface.height, work_h - TITLEBAR_HEIGHT);

    serial_println!("  -> Snap Assist layout calculations and window resizing verified.");
}

fn test_virtual_workspaces_isolation() {
    serial_println!("[TEST 3/5] Virtual Workspaces Switching & Window Allocation...");

    let mut shell = DesktopShell::new(1024, 768);

    // Switch workspaces 0 -> 1 -> 3
    shell.switch_workspace(1);
    assert_eq!(shell.active_workspace, 1);

    shell.switch_workspace(3);
    assert_eq!(shell.active_workspace, 3);

    // Out of bounds switch (must be ignored)
    shell.switch_workspace(4);
    assert_eq!(shell.active_workspace, 3);

    shell.switch_workspace(0);
    assert_eq!(shell.active_workspace, 0);

    // Window workspace allocation
    shell.assign_window_to_workspace(1, 2);
    assert_eq!(shell.window_workspaces[0], 2);

    shell.assign_window_to_workspace(2, 3);
    assert_eq!(shell.window_workspaces[1], 3);

    // Out of bounds window assignment (must safely no-op)
    shell.assign_window_to_workspace(99, 1);
    shell.assign_window_to_workspace(1, 99);
    assert_eq!(shell.window_workspaces[0], 2);

    serial_println!("  -> Virtual Workspaces switching and window isolation verified.");
}

fn test_notification_center() {
    serial_println!("[TEST 4/5] Notification Center Queueing & Eviction...");

    let mut shell = DesktopShell::new(1024, 768);

    // Post 4 notifications
    shell.post_notification("Kernel", "Subsystem online", NotificationLevel::Info, 100);
    shell.post_notification("Net", "DHCP bound to 192.168.1.50", NotificationLevel::Info, 200);
    shell.post_notification("Security", "Elevated token requested", NotificationLevel::Warning, 300);
    shell.post_notification("Storage", "Drive /dev/nvme0n1 mounted", NotificationLevel::Info, 400);

    assert!(shell.notifications[0].is_active);
    assert_eq!(shell.notifications[0].title, "Kernel");
    assert_eq!(shell.notifications[0].level, NotificationLevel::Info);

    assert!(shell.notifications[2].is_active);
    assert_eq!(shell.notifications[2].title, "Security");
    assert_eq!(shell.notifications[2].level, NotificationLevel::Warning);

    // Post 5th notification (evicts oldest slot 0)
    shell.post_notification("Alert", "Memory pressure detected", NotificationLevel::Error, 500);
    assert_eq!(shell.notifications[0].title, "Alert");
    assert_eq!(shell.notifications[0].level, NotificationLevel::Error);

    serial_println!("  -> Notification Center queueing, priority levels, and eviction verified.");
}

fn test_desktop_shell_rendering() {
    serial_println!("[TEST 5/5] Desktop Shell Compositing (Wallpaper, Taskbar, Launcher, Workspaces)...");

    let mut shell = DesktopShell::new(256, 256);
    let mut ws = WindowServer::new();
    let win_id = ws.create_window("Console", 10, 10, 60, 40).expect("Create window failed");
    shell.assign_window_to_workspace(win_id, 0);

    ws.mouse_x = 50;
    ws.mouse_y = 50;

    let mut screen = Surface::new(256, 256);

    // Pass 1: Render on Workspace 0 with launcher closed
    shell.render_shell(&ws, &mut screen);

    // Desktop background at (0, 0)
    assert_eq!(screen.get_pixel(0, 0), Color::DARK_SLATE);

    // Mouse cursor tip at (50, 50) is white
    assert_eq!(screen.get_pixel(50, 50), Color::WHITE);

    // Taskbar area at bottom (y = 236)
    assert_eq!(screen.get_pixel(2, 236), Color::rgb(0x0F, 0x17, 0x2A));

    // Console window titlebar pixel at (11, 11) is active
    assert_eq!(screen.get_pixel(11, 11), Color::TITLEBAR_BG);

    // Pass 2: Open Launcher
    shell.launcher_open = true;
    shell.render_shell(&ws, &mut screen);
    // Start button turns blue when launcher is open
    assert_eq!(screen.get_pixel(10, 222), Color::rgb(0x25, 0x63, 0xEB));

    // Pass 3: Switch to Workspace 1 (Console window from WS0 must be hidden)
    shell.switch_workspace(1);
    shell.render_shell(&ws, &mut screen);
    // Position (11, 11) should now revert to desktop wallpaper!
    assert_eq!(screen.get_pixel(11, 11), Color::DARK_SLATE);

    serial_println!("  -> Complete Desktop Shell multi-layer rendering pipeline verified.");
}
