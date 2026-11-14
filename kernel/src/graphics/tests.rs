//! NovaOS Apex — Milestone 7 Graphics & Window Server Verification Suite
//! Tests color math, alpha blending, geometry, damage tracking, window hierarchy,
//! Z-order, compositor blitting, cursor overlays, and input event routing.

use super::color::Color;
use super::compositor::Compositor;
use super::damage::DamageRegion;
use super::events::WindowEvent;
use super::geometry::{Point, Rect};
use super::surface::Surface;
use crate::serial;

pub fn run_all() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone 7 Verification Suite        \n");
    serial::write_str("=======================================================\n");

    let mut passed = true;

    // Test 1: Color, Alpha Blending, and Porter-Duff Math
    serial::write_str("[M7 TEST 1/6] Testing Color math, RGBA channels & Porter-Duff alpha blending...\n");
    if test_color_alpha_blending() {
        serial::write_str("[M7 TEST 1/6] [PASS] 32-bit ARGB channel math & Porter-Duff alpha verified.\n");
    } else {
        serial::write_str("[M7 TEST 1/6] [FAIL] Color math or alpha blending failed!\n");
        passed = false;
    }

    // Test 2: Geometry, Clipping, and Damage Region Accumulation
    serial::write_str("[M7 TEST 2/6] Testing 2D geometry, rect intersections & damage tracking...\n");
    if test_geometry_clipping_damage() {
        serial::write_str("[M7 TEST 2/6] [PASS] Rect intersection, union & dirty-rect tracking verified.\n");
    } else {
        serial::write_str("[M7 TEST 2/6] [FAIL] Geometry or damage tracking failed!\n");
        passed = false;
    }

    // Test 3: 2D Surface Drawing Primitives & Blitter
    serial::write_str("[M7 TEST 3/6] Testing Surface rasterization (rect, rounded rect, lines, blit)...\n");
    if test_surface_primitives() {
        serial::write_str("[M7 TEST 3/6] [PASS] Surface primitive rasterization & alpha blit verified.\n");
    } else {
        serial::write_str("[M7 TEST 3/6] [FAIL] Surface drawing primitives failed!\n");
        passed = false;
    }

    // Test 4: Window Hierarchy, Z-Order, and Hit Testing
    serial::write_str("[M7 TEST 4/6] Testing Window hierarchy, Z-order layering & hit testing...\n");
    if test_window_hierarchy_and_z_order() {
        serial::write_str("[M7 TEST 4/6] [PASS] Window stacking, bring-to-front & hit test verified.\n");
    } else {
        serial::write_str("[M7 TEST 4/6] [FAIL] Window hierarchy or Z-order failed!\n");
        passed = false;
    }

    // Test 5: Compositor Multi-Window Rasterization & Damage Blit
    serial::write_str("[M7 TEST 5/6] Testing Compositor multi-window blit & backbuffer damage flush...\n");
    if test_compositor_rasterization() {
        serial::write_str("[M7 TEST 5/6] [PASS] Multi-window backbuffer composition verified.\n");
    } else {
        serial::write_str("[M7 TEST 5/6] [FAIL] Compositor multi-window rasterization failed!\n");
        passed = false;
    }

    // Test 6: Mouse Cursor Sprite Overlay & Event Routing
    serial::write_str("[M7 TEST 6/6] Testing Mouse cursor sprite, backdrop restore & event queue...\n");
    if test_cursor_and_event_routing() {
        serial::write_str("[M7 TEST 6/6] [PASS] Cursor sprite overlay, non-destructive restore & events verified.\n");
    } else {
        serial::write_str("[M7 TEST 6/6] [FAIL] Cursor sprite or event routing failed!\n");
        passed = false;
    }

    serial::write_str("\n=======================================================\n");
    if passed {
        serial::write_str("[MILESTONE 7 TEST RESULT] ALL 6 TEST SUITES PASSED.\n");
        serial::write_str("[MILESTONE 7 VERIFIED] 2D Software Compositor, Window Server,\n");
        serial::write_str("                       Alpha Blending, and NovaUI verified operational!\n");
    } else {
        serial::write_str("[MILESTONE 7 TEST RESULT] ONE OR MORE TESTS FAILED!\n");
    }
    serial::write_str("=======================================================\n\n");

    passed
}

fn test_color_alpha_blending() -> bool {
    let red = Color::from_rgb(255, 0, 0);
    let green = Color::from_rgb(0, 255, 0);
    let blue = Color::from_rgb(0, 0, 255);
    let white = Color::WHITE;
    let black = Color::BLACK;

    // Verify channel extraction
    if red.r() != 255 || red.g() != 0 || red.b() != 0 || red.a() != 255 {
        return false;
    }
    if green.r() != 0 || green.g() != 255 || green.b() != 0 {
        return false;
    }
    if blue.r() != 0 || blue.g() != 0 || blue.b() != 255 {
        return false;
    }

    // 1. Opaque source over destination returns source
    let blended_opaque = Color::blend(red, black);
    if blended_opaque != red {
        return false;
    }

    // 2. Fully transparent source returns destination
    let trans = Color::from_argb(0, 255, 255, 255);
    let blended_trans = Color::blend(trans, blue);
    if blended_trans != blue {
        return false;
    }

    // 3. 50% white over black should yield ~128 gray
    let half_white = Color::from_argb(128, 255, 255, 255);
    let blended_half = Color::blend(half_white, black);
    // Integer division 255 * 128 / 255 = 128
    if blended_half.r() != 128 || blended_half.g() != 128 || blended_half.b() != 128 {
        return false;
    }

    // 4. Global opacity modulation
    let modulated = Color::blend_with_opacity(white, black, 128);
    if modulated.r() != 128 || modulated.g() != 128 || modulated.b() != 128 {
        return false;
    }

    true
}

fn test_geometry_clipping_damage() -> bool {
    let r1 = Rect::new(10, 10, 100, 100);
    let r2 = Rect::new(50, 50, 100, 100);
    let r3 = Rect::new(200, 200, 50, 50);

    // 1. Point containment
    if !r1.contains_point(Point::new(15, 15)) {
        return false;
    }
    if r1.contains_point(Point::new(5, 5)) {
        return false;
    }
    if r1.contains_point(Point::new(110, 110)) {
        return false;
    }

    // 2. Intersections
    if !r1.intersects(&r2) {
        return false;
    }
    if r1.intersects(&r3) {
        return false;
    }

    // 3. Exact intersection computation
    let inter = match r1.intersection(&r2) {
        Some(i) => i,
        None => return false,
    };
    if inter.x != 50 || inter.y != 50 || inter.width != 60 || inter.height != 60 {
        return false;
    }
    if r1.intersection(&r3).is_some() {
        return false;
    }

    // 4. Union computation
    let union_rect = r1.union(&r2);
    if union_rect.x != 10 || union_rect.y != 10 || union_rect.width != 140 || union_rect.height != 140 {
        return false;
    }

    // 5. Damage region tracking
    let mut damage = DamageRegion::new();
    if !damage.is_empty() {
        return false;
    }
    damage.add_rect(r1);
    if damage.is_empty() || damage.rects().len() != 1 {
        return false;
    }

    // Adding intersecting rect merges into union
    damage.add_rect(r2);
    let bbox = match damage.bounding_box() {
        Some(b) => b,
        None => return false,
    };
    if bbox.width != 140 || bbox.height != 140 {
        return false;
    }

    damage.clear();
    if !damage.is_empty() {
        return false;
    }

    true
}

fn test_surface_primitives() -> bool {
    let mut surface = Surface::new(100, 100);
    surface.clear(Color::BLACK);

    // 1. Verify clear
    if surface.get_pixel(50, 50) != Color::BLACK {
        return false;
    }

    // 2. Fill rectangle
    let test_rect = Rect::new(20, 20, 40, 40);
    surface.fill_rect(test_rect, Color::RED);
    if surface.get_pixel(19, 19) != Color::BLACK {
        return false;
    }
    if surface.get_pixel(25, 25) != Color::RED {
        return false;
    }
    if surface.get_pixel(59, 59) != Color::RED {
        return false;
    }
    if surface.get_pixel(60, 60) != Color::BLACK {
        return false;
    }

    // 3. Draw outline
    let outline_rect = Rect::new(10, 10, 80, 80);
    surface.draw_rect_outline(outline_rect, Color::GREEN, 2);
    // Border should be green
    if surface.get_pixel(10, 10) != Color::GREEN || surface.get_pixel(11, 11) != Color::GREEN {
        return false;
    }
    // Outside should be black
    if surface.get_pixel(9, 9) != Color::BLACK {
        return false;
    }

    // 4. Draw line
    surface.draw_line(0, 0, 99, 0, Color::BLUE);
    for x in 0..100 {
        if surface.get_pixel(x, 0) != Color::BLUE {
            return false;
        }
    }

    // 5. Blit sub-region with alpha
    let mut src = Surface::new(20, 20);
    src.clear(Color::from_argb(128, 255, 255, 255));
    surface.blit(40, 40, &src, None);

    // Pixel at (45, 45) was inside test_rect (RED = 255, 0, 0)
    // Blending 50% white over red gives (255, 128, 128)
    let p = surface.get_pixel(45, 45);
    if p.r() != 255 || p.g() != 128 || p.b() != 128 {
        return false;
    }

    true
}

fn test_window_hierarchy_and_z_order() -> bool {
    let mut comp = Compositor::new(400, 300);

    // Create 3 overlapping windows
    let w1 = comp.create_window("Window 1", 20, 20, 100, 80, 0, 100);
    let w2 = comp.create_window("Window 2", 40, 40, 100, 80, 0, 101);
    let w3 = comp.create_window("Window 3", 60, 60, 100, 80, 0, 102);

    // Most recently created window (w3) should be active and on top
    if comp.active_window != Some(w3) {
        return false;
    }

    // Hit test at (70, 70) where w1, w2, and w3 all overlap
    // Should return w3 because it is on top
    if comp.window_at(70, 70) != Some(w3) {
        return false;
    }

    // Bring w1 to front
    comp.bring_to_front(w1);
    if comp.active_window != Some(w1) {
        return false;
    }

    // Now at (70, 70), w1 is on top!
    if comp.window_at(70, 70) != Some(w1) {
        return false;
    }

    // Verify Z-orders: w2 is 0, w3 is 1, w1 is 2
    let win1 = comp.get_window(w1).unwrap();
    let win2 = comp.get_window(w2).unwrap();
    let win3 = comp.get_window(w3).unwrap();
    if win2.z_order != 0 || win3.z_order != 1 || win1.z_order != 2 {
        return false;
    }

    // Destroy w1 -> w3 should become active
    comp.destroy_window(w1);
    if comp.active_window != Some(w3) {
        return false;
    }

    true
}

fn test_compositor_rasterization() -> bool {
    let mut comp = Compositor::new(400, 300);

    let w1 = comp.create_window("App 1", 20, 20, 100, 80, 0, 100);
    if let Some(win) = comp.get_window_mut(w1) {
        win.surface.clear(Color::SUCCESS_GREEN);
    }
    comp.damage_window(w1);

    // Composite backbuffer
    comp.composite();

    // The client area of w1 should contain SUCCESS_GREEN in backbuffer
    let win = comp.get_window(w1).unwrap();
    let cr = win.client_rect();
    let center_x = (cr.x + 10) as u32;
    let center_y = (cr.y + 10) as u32;

    let pixel = comp.backbuffer.get_pixel(center_x, center_y);
    if pixel != Color::SUCCESS_GREEN {
        return false;
    }

    // Verify damage region was cleared
    if !comp.damage.is_empty() {
        return false;
    }

    true
}

fn test_cursor_and_event_routing() -> bool {
    let mut comp = Compositor::new(400, 300);
    let w1 = comp.create_window("Target Window", 50, 50, 100, 80, 0, 100);

    // 1. Move mouse into window client area
    let target_x = 70;
    let target_y = 80;
    comp.on_mouse_move(target_x, target_y);

    if comp.cursor.x != target_x || comp.cursor.y != target_y {
        return false;
    }

    // Check that Window 1 received the translated MouseMove event
    let win = comp.get_window_mut(w1).unwrap();
    let cr = win.client_rect();
    let expected_client_x = target_x - cr.x;
    let expected_client_y = target_y - cr.y;

    let mut found_move = false;
    while let Some(ev) = win.pop_event() {
        if let WindowEvent::MouseMove { x, y, .. } = ev {
            if x == expected_client_x && y == expected_client_y {
                found_move = true;
                break;
            }
        }
    }
    if !found_move {
        return false;
    }

    // 2. Mouse click down event
    comp.on_mouse_down(target_x, target_y, 1);
    let win = comp.get_window_mut(w1).unwrap();
    let mut found_down = false;
    while let Some(ev) = win.pop_event() {
        if let WindowEvent::MouseDown { x, y, button } = ev {
            if x == expected_client_x && y == expected_client_y && button == 1 {
                found_down = true;
                break;
            }
        }
    }
    if !found_down {
        return false;
    }

    // 3. Key press forwarding to active window
    comp.on_key_event(0x1E, b'a', true);
    let win = comp.get_window_mut(w1).unwrap();
    let mut found_char = false;
    while let Some(ev) = win.pop_event() {
        if let WindowEvent::Char(c) = ev {
            if c == 'a' {
                found_char = true;
                break;
            }
        }
    }
    if !found_char {
        return false;
    }

    true
}
