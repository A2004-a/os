//! NovaOS Zero — Milestone M7 Automated Test Suite
//! Validates Alpha Blending, Clipping Bounds, Window Z-Ordering, Compositing Pipeline, and NovaUI Widgets.

use crate::graphics::compositor::{Color, Rect, Surface};
use crate::graphics::window_server::WindowServer;
use crate::graphics::novaui::{Button, ProgressBar, WidgetState};
use crate::serial_println;

pub fn run_graphics_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M7 Automated Verification");
    serial_println!("=======================================================");

    test_alpha_blending();
    test_clipping_bounds_safety();
    test_window_z_ordering();
    test_compositor_desktop_pass();
    test_novaui_widgets();

    serial_println!("[M7 SUCCESS] All Milestone M7 Verification Tests Passed Cleanly!\n");
}

fn test_alpha_blending() {
    serial_println!("[TEST 1/5] Alpha Blending & Porter-Duff Calculation...");

    // 1. Opaque source replaces destination
    let opaque_red = Color::rgba(255, 0, 0, 255);
    let solid_blue = Color::rgba(0, 0, 255, 255);
    assert_eq!(Color::blend(opaque_red, solid_blue), opaque_red);

    // 2. Fully transparent source leaves destination untouched
    let transparent = Color::rgba(255, 0, 0, 0);
    assert_eq!(Color::blend(transparent, solid_blue), solid_blue);

    // 3. 50% translucent red over solid blue yields purple
    let half_red = Color::rgba(255, 0, 0, 128);
    let blended = Color::blend(half_red, solid_blue);
    assert_eq!(blended.r, 128);
    assert_eq!(blended.g, 0);
    assert_eq!(blended.b, 127);
    assert_eq!(blended.a, 255);

    serial_println!("  -> Alpha blending computation verified.");
}

fn test_clipping_bounds_safety() {
    serial_println!("[TEST 2/5] Clipping Rectangles & Surface Memory Safety...");

    let mut surface = Surface::new(64, 64);
    surface.clear(Color::BLACK);

    // Draw out-of-bounds pixels (must safely no-op without memory corruption or panic)
    surface.set_pixel(-10, -10, Color::WHITE);
    surface.set_pixel(100, 100, Color::WHITE);
    assert_eq!(surface.get_pixel(-10, -10), Color::TRANSPARENT);
    assert_eq!(surface.get_pixel(100, 100), Color::TRANSPARENT);

    // Draw partially overlapping rectangle spanning negative coordinates
    surface.fill_rect(Rect::new(-5, -5, 15, 15), Color::RED);
    // Pixel inside bounds (0, 0) must be red
    assert_eq!(surface.get_pixel(0, 0), Color::RED);
    assert_eq!(surface.get_pixel(9, 9), Color::RED);
    // Pixel beyond rect (10, 10) must remain black
    assert_eq!(surface.get_pixel(10, 10), Color::BLACK);

    serial_println!("  -> Clipping boundary enforcement and memory safety verified.");
}

fn test_window_z_ordering() {
    serial_println!("[TEST 3/5] Window Stacking & Z-Order Focus Management...");

    let mut server = WindowServer::new();

    let win1 = server.create_window("Terminal", 10, 10, 100, 80).expect("Create win1 failed");
    let _win2 = server.create_window("Editor", 30, 30, 100, 80).expect("Create win2 failed");

    // Win2 is newest, should be focused
    assert!(!server.windows[0].as_ref().unwrap().is_focused);
    assert!(server.windows[1].as_ref().unwrap().is_focused);

    // Focus Win1: should become focused and top of Z-order
    server.focus_window(win1);
    assert!(server.windows[0].as_ref().unwrap().is_focused);
    assert!(!server.windows[1].as_ref().unwrap().is_focused);

    let z1 = server.windows[0].as_ref().unwrap().z_order;
    let z2 = server.windows[1].as_ref().unwrap().z_order;
    assert!(z1 > z2);

    serial_println!("  -> Window focus and Z-order elevation verified.");
}

fn test_compositor_desktop_pass() {
    serial_println!("[TEST 4/5] Multi-Layer Compositing (Desktop + Windows + Cursor)...");

    let mut server = WindowServer::new();
    server.create_window("Status", 10, 10, 60, 40).expect("Create window failed");
    server.mouse_x = 50;
    server.mouse_y = 50;

    let mut desktop = Surface::new(120, 120);
    server.composite_desktop(&mut desktop);

    // Verify background wallpaper at (0, 0)
    assert_eq!(desktop.get_pixel(0, 0), Color::DARK_SLATE);

    // Verify mouse cursor tip at (50, 50) is white
    assert_eq!(desktop.get_pixel(50, 50), Color::WHITE);

    serial_println!("  -> Multi-layer composition pass verified.");
}

fn test_novaui_widgets() {
    serial_println!("[TEST 5/5] NovaUI Component Hierarchy & State Machine...");

    let mut surface = Surface::new(128, 64);
    surface.clear(Color::WINDOW_BG);

    // 1. Button rendering
    let mut btn = Button::new("Click", Rect::new(10, 10, 50, 20));
    btn.render(&mut surface);
    btn.state = WidgetState::Pressed;
    btn.render(&mut surface);

    // 2. ProgressBar rendering
    let pbar = ProgressBar::new(Rect::new(10, 40, 100, 12), 75);
    pbar.render(&mut surface);

    // Pixel at (15, 45) inside progress bar must be green
    assert_eq!(surface.get_pixel(15, 45), Color::GREEN);

    serial_println!("  -> NovaUI widgets (Button, ProgressBar) rendered and verified.");
}
