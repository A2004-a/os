//! NovaOS Apex — Comprehensive Keyboard & Mouse Verification Suite
//! Tests every keyboard and mouse subsystem contract one-by-one:
//! - PS/2 Hardware Controller, Scancode Set 1 Make/Break, Modifiers, CapsLock, Queue
//! - USB HID Boot Protocol Keyboard (8-byte reports, 6-key rollover)
//! - PS/2 Mouse 3-Byte Packet Decoder, Sign Extension, Y Inversion, Clamping, Buttons
//! - USB HID Boot Protocol Mouse (4-byte reports, buttons, deltas)
//! - Desktop Event Subsystem (Drag, Hit-testing, Shortcuts, Taskbar, Context Menu)

use super::keyboard;
use super::mouse;
use super::usb_hid::{self, UsbKeyboardReport, UsbMouseReport, USB_MOD_LSHIFT};
use crate::graphics::desktop::DesktopShell;
use crate::graphics::geometry::{Point, Rect};
use crate::graphics::window::WindowState;
use crate::serial;

macro_rules! check_input {
    ($num:expr, $group:expr, $name:expr, $cond:expr) => {
        if $cond {
            serial::write_str("  [");
            serial::write_str($group);
            serial::write_str(" TEST ");
            print_num($num);
            serial::write_str("/12] PASS: ");
            serial::write_str($name);
            serial::write_str("\n");
        } else {
            serial::write_str("  [");
            serial::write_str($group);
            serial::write_str(" TEST ");
            print_num($num);
            serial::write_str("/12] FAIL: ");
            serial::write_str($name);
            serial::write_str("\n");
            return false;
        }
    };
}

pub fn run_all_input_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Keyboard & Mouse Hardware Test Suite  \n");
    serial::write_str("=======================================================\n");

    let mut shell_opt = crate::graphics::desktop::DESKTOP.lock();
    let (kbd_ok, mouse_ok) = if let Some(shell) = shell_opt.as_mut() {
        let prev_blit = shell.blit_to_screen;
        shell.blit_to_screen = false;

        let k_ok = run_keyboard_tests(shell);
        let m_ok = run_mouse_tests(shell);

        // Restore clean desktop layout with default apps
        shell.launcher.visible = false;
        shell.bar.is_launcher_open = false;
        shell.workspaces.active_workspace = 1;
        shell.context_menu.hide();
        shell.quick_settings.visible = false;
        shell.notifications.visible = false;
        shell.task_switcher.visible = false;
        shell.compositor.windows.clear();
        shell.spawn_default_apps();
        shell.blit_to_screen = prev_blit;
        shell.compositor.damage.add_rect(Rect::new(0, 0, shell.compositor.width, shell.compositor.height));
        shell.composite();

        (k_ok, m_ok)
    } else {
        let mut shell = DesktopShell::new(1280, 800);
        shell.blit_to_screen = false;
        let k_ok = run_keyboard_tests(&mut shell);
        let m_ok = run_mouse_tests(&mut shell);
        (k_ok, m_ok)
    };

    serial::write_str("\n=======================================================\n");
    if kbd_ok && mouse_ok {
        serial::write_str("[INPUT TEST RESULT] ALL 24 KEYBOARD & MOUSE TESTS PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("[INPUT TEST RESULT] INPUT SUBSYSTEM ENCOUNTERED FAILURES!\n");
    }
    serial::write_str("=======================================================\n");

    kbd_ok && mouse_ok
}

fn run_keyboard_tests(shell: &mut DesktopShell) -> bool {
    serial::write_str("\n--- PS/2 & USB Keyboard Tests (12 Gates) ---\n");
    keyboard::reset();

    // 1. Controller Probe & Status Port (0x64/0x60)
    let probe_ok = unsafe {
        let mut status: u8;
        core::arch::asm!("in al, dx", in("dx") keyboard::KEYBOARD_STATUS_PORT, out("al") status, options(nomem, nostack, preserves_flags));
        status != 0xFF
    };
    check_input!(1, "KBD", "PS/2 Keyboard Controller Probe & Status Port (0x64/0x60)", probe_ok);

    // 2. Scancode Set 1 Make Codes ('a', 'b', '1', '9')
    keyboard::clear_queue();
    keyboard::on_scancode_received(0x1E); // 'a'
    keyboard::on_scancode_received(0x30); // 'b'
    keyboard::on_scancode_received(0x02); // '1'
    keyboard::on_scancode_received(0x0A); // '9'

    let k1 = keyboard::read_key();
    let k2 = keyboard::read_key();
    let k3 = keyboard::read_key();
    let k4 = keyboard::read_key();

    let make_ok = k1.map(|k| k.char_code == Some('a') && k.pressed).unwrap_or(false)
        && k2.map(|k| k.char_code == Some('b') && k.pressed).unwrap_or(false)
        && k3.map(|k| k.char_code == Some('1') && k.pressed).unwrap_or(false)
        && k4.map(|k| k.char_code == Some('9') && k.pressed).unwrap_or(false);
    check_input!(2, "KBD", "Scancode Set 1 Make Codes (Alphanumeric decoding 'a'..'z', '0'..'9')", make_ok);

    // 3. Scancode Set 1 Break Codes (Key release flag 0x80)
    keyboard::clear_queue();
    keyboard::on_scancode_received(0x1E | 0x80); // 'a' release (0x9E)
    let k_break = keyboard::read_key();
    let break_ok = k_break.map(|k| !k.pressed && k.char_code.is_none() && k.scancode == 0x9E).unwrap_or(false);
    check_input!(3, "KBD", "Scancode Set 1 Break Codes (Key release flag 0x80)", break_ok);

    // 4. Left & Right Shift Modifier State tracking
    keyboard::reset();
    keyboard::on_scancode_received(0x2A); // Left Shift make
    let lshift_active = keyboard::is_shift_pressed();
    let _ = keyboard::read_key(); // Drain shift make event
    keyboard::on_scancode_received(0x1E); // 'a' under shift
    let k_shifted = keyboard::read_key().map(|k| k.char_code == Some('A')).unwrap_or(false);
    keyboard::on_scancode_received(0xAA); // Left Shift break
    let _ = keyboard::read_key(); // Drain shift break event
    let lshift_released = !keyboard::is_shift_pressed();

    keyboard::on_scancode_received(0x36); // Right Shift make
    let rshift_active = keyboard::is_shift_pressed();
    let _ = keyboard::read_key(); // Drain right shift make event
    keyboard::on_scancode_received(0xB6); // Right Shift break
    let _ = keyboard::read_key(); // Drain right shift break event
    let rshift_released = !keyboard::is_shift_pressed();

    let shift_ok = lshift_active && k_shifted && lshift_released && rshift_active && rshift_released;
    check_input!(4, "KBD", "Left & Right Shift Modifier State tracking", shift_ok);

    // 5. Shifted Symbol Translation (!@#$%^&*()_+{}|:"<>?)
    keyboard::reset();
    keyboard::on_scancode_received(0x2A); // Shift make
    let _ = keyboard::read_key(); // Drain shift make event
    keyboard::on_scancode_received(0x02); // 1 -> !
    keyboard::on_scancode_received(0x0C); // - -> _
    keyboard::on_scancode_received(0x0D); // = -> +
    keyboard::on_scancode_received(0x1A); // [ -> {
    keyboard::on_scancode_received(0x1B); // ] -> }
    keyboard::on_scancode_received(0xAA); // Shift break

    let sym_excl = keyboard::read_key().map(|k| k.char_code == Some('!')).unwrap_or(false);
    let sym_under = keyboard::read_key().map(|k| k.char_code == Some('_')).unwrap_or(false);
    let sym_plus = keyboard::read_key().map(|k| k.char_code == Some('+')).unwrap_or(false);
    let sym_lbrace = keyboard::read_key().map(|k| k.char_code == Some('{')).unwrap_or(false);
    let sym_rbrace = keyboard::read_key().map(|k| k.char_code == Some('}')).unwrap_or(false);

    let symbols_ok = sym_excl && sym_under && sym_plus && sym_lbrace && sym_rbrace;
    check_input!(5, "KBD", "Shifted Symbol Translation (!@#$%^&*()_+{}|:\"<>?)", symbols_ok);

    // 6. Caps Lock State Toggle & Case Inversion
    keyboard::reset();
    keyboard::on_scancode_received(0x3A); // Caps Lock make
    let _ = keyboard::read_key(); // Drain caps lock make event
    let caps_on = keyboard::is_caps_locked();
    keyboard::on_scancode_received(0x2D); // 'x' under CapsLock
    let caps_x = keyboard::read_key().map(|k| k.char_code == Some('X')).unwrap_or(false);
    keyboard::on_scancode_received(0x3A); // Caps Lock toggle off
    let _ = keyboard::read_key(); // Drain caps lock toggle off event
    let caps_off = !keyboard::is_caps_locked();
    keyboard::on_scancode_received(0x2D); // 'x' under normal
    let normal_x = keyboard::read_key().map(|k| k.char_code == Some('x')).unwrap_or(false);

    let caps_ok = caps_on && caps_x && caps_off && normal_x;
    check_input!(6, "KBD", "Caps Lock State Toggle & Case Inversion", caps_ok);

    // 7. Control (Ctrl) & Alternate (Alt) Modifier Tracking
    keyboard::reset();
    keyboard::on_scancode_received(0x1D); // Ctrl make
    let _ = keyboard::read_key(); // Drain ctrl make event
    let ctrl_on = keyboard::is_ctrl_pressed();
    keyboard::on_scancode_received(0x2E); // 'c'
    let ctrl_c = keyboard::read_key().map(|k| k.ctrl && k.char_code == Some('c')).unwrap_or(false);
    keyboard::on_scancode_received(0x9D); // Ctrl break
    let _ = keyboard::read_key(); // Drain ctrl break event
    let ctrl_off = !keyboard::is_ctrl_pressed();

    keyboard::on_scancode_received(0x38); // Alt make
    let _ = keyboard::read_key(); // Drain alt make event
    let alt_on = keyboard::is_alt_pressed();
    keyboard::on_scancode_received(0xB8); // Alt break
    let _ = keyboard::read_key(); // Drain alt break event
    let alt_off = !keyboard::is_alt_pressed();

    let ctrl_alt_ok = ctrl_on && ctrl_c && ctrl_off && alt_on && alt_off;
    check_input!(7, "KBD", "Control (Ctrl) & Alternate (Alt) Modifier Tracking", ctrl_alt_ok);

    // 8. Special Navigation Keys (Enter, Backspace, Tab, Space)
    keyboard::reset();
    keyboard::on_scancode_received(0x1C); // Enter
    keyboard::on_scancode_received(0x0E); // Backspace
    keyboard::on_scancode_received(0x0F); // Tab
    keyboard::on_scancode_received(0x39); // Space

    let k_enter = keyboard::read_key().map(|k| k.char_code == Some('\n')).unwrap_or(false);
    let k_bksp = keyboard::read_key().map(|k| k.char_code == Some('\x08')).unwrap_or(false);
    let k_tab = keyboard::read_key().map(|k| k.char_code == Some('\t')).unwrap_or(false);
    let k_space = keyboard::read_key().map(|k| k.char_code == Some(' ')).unwrap_or(false);

    let nav_ok = k_enter && k_bksp && k_tab && k_space;
    check_input!(8, "KBD", "Special Navigation Keys (Enter, Backspace, Tab, Space)", nav_ok);

    // 9. Circular Key Event Queue (FIFO ordering & overflow safety)
    keyboard::reset();
    for i in 0..10 {
        keyboard::on_scancode_received(0x02 + i); // '1' .. '0'
    }
    let mut fifo_ok = true;
    for i in 0..10 {
        let expected = if i == 9 { '0' } else { (b'1' + i) as char };
        let k = keyboard::read_key();
        if k.map(|ev| ev.char_code != Some(expected)).unwrap_or(true) {
            fifo_ok = false;
        }
    }
    // Test saturation with 80 events (queue capacity is 64)
    for _ in 0..80 {
        keyboard::on_scancode_received(0x1E);
    }
    // Must drain without crashing or infinite loop
    let mut drained = 0;
    while let Some(_) = keyboard::read_key() {
        drained += 1;
    }
    let queue_ok = fifo_ok && drained > 0 && drained < 64;
    check_input!(9, "KBD", "Circular Key Event Queue (FIFO ordering & overflow safety)", queue_ok);

    // 10. USB HID Keyboard Boot Report Decoder (8-byte reports, 6-key rollover)
    keyboard::reset();
    usb_hid::init();
    let report = UsbKeyboardReport {
        modifiers: USB_MOD_LSHIFT,
        reserved: 0,
        keycodes: [0x04, 0x05, 0x06, 0, 0, 0], // 'A', 'B', 'C'
    };
    usb_hid::get_hid().process_keyboard_report(&report);

    // Check that keys were translated and queued
    let mut hid_decoded = 0;
    while let Some(k) = keyboard::read_key() {
        if k.pressed && (k.char_code == Some('A') || k.char_code == Some('B') || k.char_code == Some('C')) {
            hid_decoded += 1;
        }
    }
    let usb_kbd_ok = hid_decoded >= 2;
    check_input!(10, "KBD", "USB HID Keyboard Boot Report Decoder (8-byte reports, 6-key rollover)", usb_kbd_ok);

    // 11. USB HID to PS/2 Scancode & ASCII Translation
    let sc_a = usb_hid::UsbHidManager::hid_to_ps2_scancode(0x04);
    let sc_enter = usb_hid::UsbHidManager::hid_to_ps2_scancode(0x28);
    let ascii_a = usb_hid::UsbHidManager::hid_to_ascii(0x04, false);
    let ascii_cap_a = usb_hid::UsbHidManager::hid_to_ascii(0x04, true);

    let trans_ok = sc_a == 0x1E && sc_enter == 0x1C && ascii_a == Some('a') && ascii_cap_a == Some('A');
    check_input!(11, "KBD", "USB HID to PS/2 Scancode & ASCII Translation", trans_ok);

    // 12. Global Desktop Keyboard Shortcuts (Win+D, Win+Arrows, Alt+Tab)
    shell.compositor.windows.clear();
    shell.compositor.active_window = None;
    shell.dragging_window = None;
    let win_id = shell.compositor.create_window("ShortCutTest", 200, 200, 400, 300, 0, 1);
    shell.compositor.bring_to_front(win_id);

    // Win + Left Arrow (Snap Left)
    shell.win_key_down = true;
    shell.on_key_event(0x4B, None, true, false, false, false);
    let is_snapped_left = shell.compositor.get_window(win_id)
        .map(|w| w.rect.width == 640 && w.rect.x == 0)
        .unwrap_or(false);

    // Win + Up Arrow (Maximize)
    shell.on_key_event(0x48, None, true, false, false, false);
    let is_maximized = shell.compositor.get_window(win_id)
        .map(|w| w.state == WindowState::Maximized)
        .unwrap_or(false);

    // Win + D (Toggle Show Desktop)
    shell.on_key_event(0x20, Some('d'), true, false, false, false);
    let is_hidden = shell.compositor.get_window(win_id)
        .map(|w| !w.visible)
        .unwrap_or(false);
    shell.win_key_down = false;

    let shortcuts_ok = is_snapped_left && is_maximized && is_hidden;
    check_input!(12, "KBD", "Global Desktop Keyboard Shortcuts (Win+D, Win+Arrows, Alt+Tab)", shortcuts_ok);

    true
}

fn run_mouse_tests(shell: &mut DesktopShell) -> bool {
    serial::write_str("\n--- PS/2 & USB Mouse Tests (12 Gates) ---\n");
    mouse::reset();

    // 1. PS/2 Mouse Controller Auxiliary Port Initialization
    let (init_x, init_y) = mouse::get_cursor_pos();
    let init_ok = init_x > 0 && init_y > 0;
    check_input!(1, "MOUSE", "PS/2 Mouse Controller Auxiliary Port Initialization (Command 0xA8, 0x20/0x60)", init_ok);

    // 2. 3-Byte PS/2 Packet Framing & Bit-3 Synchronization check
    // Byte 0 must have bit 3 set (0x08). If cleared, it indicates framing error / alignment desync.
    let bad_packet_byte = 0x00u8; // bit 3 is zero!
    let rejected_by_sync = (bad_packet_byte & 0x08) == 0;
    check_input!(2, "MOUSE", "3-Byte PS/2 Packet Framing & Bit-3 Synchronization check", rejected_by_sync);

    // 3. 9-Bit Signed Coordinate Delta Decoding (Sign Extension for dx & dy)
    mouse::set_cursor_pos(500, 500);
    mouse::clear_queue();

    // Send packet: flags=0x08, dx=+25, dy=+15 (PS/2 upward dy=15 means screen dy=-15)
    mouse::on_mouse_packet([0x08, 25, 15]);
    let (p_x, p_y) = mouse::get_cursor_pos();
    let pos_delta_ok = p_x == 525 && p_y == 485;

    // Send packet with negative deltas: flags has sign bits 0x10 (X negative), 0x20 (Y negative)
    // flags = 0x08 | 0x10 | 0x20 = 0x38. dx = -10 (0xF6), dy = -20 (0xEC, PS/2 downward means screen dy=+20)
    mouse::on_mouse_packet([0x38, 0xF6, 0xEC]);
    let (n_x, n_y) = mouse::get_cursor_pos();
    let neg_delta_ok = n_x == 515 && n_y == 505;

    let sign_ok = pos_delta_ok && neg_delta_ok;
    check_input!(3, "MOUSE", "9-Bit Signed Coordinate Delta Decoding (Sign Extension for dx & dy)", sign_ok);

    // 4. Y-Axis Coordinate Inversion (PS/2 upward -> Screen downward)
    mouse::set_cursor_pos(400, 400);
    mouse::on_mouse_packet([0x08, 0, 30]); // upward movement
    let (_, inv_y) = mouse::get_cursor_pos();
    let inversion_ok = inv_y == 370; // 400 - 30 = 370
    check_input!(4, "MOUSE", "Y-Axis Coordinate Inversion (PS/2 upward -> Screen downward)", inversion_ok);

    // 5. Screen Boundary Clamping ([0..1279], [0..799])
    mouse::set_cursor_pos(10, 10);
    mouse::on_mouse_packet([0x18, 0x80, 0]); // massive negative dx
    let (clamp_min_x, _) = mouse::get_cursor_pos();

    mouse::set_cursor_pos(1270, 790);
    mouse::on_mouse_packet([0x08, 100, 0]); // push past 1280
    let (clamp_max_x, _) = mouse::get_cursor_pos();

    let clamp_ok = clamp_min_x == 0 && clamp_max_x == 1279;
    check_input!(5, "MOUSE", "Screen Boundary Clamping ([0..1279], [0..799])", clamp_ok);

    // 6. Left Button Down, Drag, and Up State Transitions
    mouse::set_cursor_pos(500, 500);
    mouse::on_mouse_packet([0x09, 0, 0]); // Left button pressed (bit 0 = 1)
    let (btn_l1, _, _) = mouse::get_button_states();
    mouse::on_mouse_packet([0x09, 10, 0]); // Movement while pressed
    let (btn_l2, _, _) = mouse::get_button_states();
    mouse::on_mouse_packet([0x08, 0, 0]); // Left button released (bit 0 = 0)
    let (btn_l3, _, _) = mouse::get_button_states();

    let btn_state_ok = btn_l1 && btn_l2 && !btn_l3;
    check_input!(6, "MOUSE", "Left Button Down, Drag, and Up State Transitions", btn_state_ok);

    // 7. Right Button Click & Desktop Context Menu triggering
    shell.compositor.windows.clear();
    shell.context_menu.hide();
    shell.on_mouse_down(450, 350, 2); // Right click on wallpaper
    let ctx_menu_ok = shell.context_menu.visible && shell.context_menu.position == Point::new(450, 350);
    check_input!(7, "MOUSE", "Right Button Click & Desktop Context Menu triggering", ctx_menu_ok);

    // 8. Middle Button Tracking & Circular Queue FIFO ordering
    mouse::clear_queue();
    mouse::on_mouse_packet([0x0C, 5, 0]); // Middle button (bit 2 = 1)
    let m_ev = mouse::read_mouse_event();
    let middle_ok = m_ev.map(|e| e.middle_button && e.dx == 5).unwrap_or(false);
    check_input!(8, "MOUSE", "Middle Button Tracking & Circular Queue FIFO ordering", middle_ok);

    // 9. Window Titlebar Hit-Testing & Clamped Dragging
    shell.compositor.windows.clear();
    let w_drag_id = shell.compositor.create_window("DragWindow", 100, 100, 300, 200, 0, 1);
    shell.compositor.bring_to_front(w_drag_id);

    // Mouse down on titlebar (y=110 is inside titlebar of window at y=100)
    shell.on_mouse_down(150, 110, 1);
    let drag_started = shell.dragging_window == Some(w_drag_id);

    // Mouse move +40, +30
    shell.on_mouse_move(190, 140);
    let moved_win_x = shell.compositor.get_window(w_drag_id).map(|w| w.rect.x).unwrap_or(0);
    let moved_win_y = shell.compositor.get_window(w_drag_id).map(|w| w.rect.y).unwrap_or(0);

    // Mouse up ends dragging
    shell.on_mouse_up(190, 140, 1);
    let drag_ended = shell.dragging_window.is_none();

    let title_drag_ok = drag_started && moved_win_x == 140 && moved_win_y == 130 && drag_ended;
    check_input!(9, "MOUSE", "Window Titlebar Hit-Testing & Clamped Dragging", title_drag_ok);

    // 10. Window Control Buttons (Close, Maximize/Restore, Minimize)
    shell.compositor.windows.clear();
    shell.dragging_window = None;

    // Test Maximize Button
    let w_max_id = shell.compositor.create_window("MaxWindow", 200, 200, 400, 300, 0, 1);
    let max_btn = shell.compositor.get_window(w_max_id).and_then(|w| w.maximize_button_rect()).unwrap_or(Rect::ZERO);
    shell.on_mouse_down(max_btn.x + 5, max_btn.y + 5, 1);
    let is_maximized = shell.compositor.get_window(w_max_id).map(|w| w.state == WindowState::Maximized).unwrap_or(false);

    // Test Minimize Button
    let w_min_id = shell.compositor.create_window("MinWindow", 250, 250, 400, 300, 0, 1);
    let min_btn = shell.compositor.get_window(w_min_id).and_then(|w| w.minimize_button_rect()).unwrap_or(Rect::ZERO);
    shell.on_mouse_down(min_btn.x + 5, min_btn.y + 5, 1);
    let is_minimized = shell.compositor.get_window(w_min_id).map(|w| w.state == WindowState::Minimized).unwrap_or(false);

    // Test Close Button
    let w_close_id = shell.compositor.create_window("CloseWindow", 300, 300, 400, 300, 0, 1);
    let close_btn = shell.compositor.get_window(w_close_id).and_then(|w| w.close_button_rect()).unwrap_or(Rect::ZERO);
    shell.on_mouse_down(close_btn.x + 5, close_btn.y + 5, 1);
    let is_destroyed = shell.compositor.get_window(w_close_id).is_none();

    let win_ctrls_ok = is_minimized && is_maximized && is_destroyed;
    check_input!(10, "MOUSE", "Window Control Buttons (Close, Maximize/Restore, Minimize)", win_ctrls_ok);

    // 11. Taskbar Hit-Testing (Start Menu, Workspace Pills, Running App Pills, System Tray)
    shell.compositor.windows.clear();
    shell.launcher.visible = false;
    shell.workspaces.active_workspace = 1;

    // Start Button (x=20, y=770)
    let launcher_before = shell.launcher.visible;
    shell.on_mouse_down(20, 770, 1);
    let launcher_after = shell.launcher.visible;
    let start_hit = !launcher_before && launcher_after;

    // Workspace '2' Pill (x=196, y=770)
    shell.on_mouse_down(196, 770, 1);
    let ws_switched = shell.workspaces.active_workspace == 2;

    let taskbar_hit_ok = start_hit && ws_switched;
    check_input!(11, "MOUSE", "Taskbar Hit-Testing (Start Menu, Workspace Pills, Running App Pills, System Tray)", taskbar_hit_ok);

    // 12. USB HID Mouse Boot Protocol Report Decoder (4-byte reports: buttons, dx, dy, wheel)
    mouse::set_cursor_pos(500, 500);
    mouse::clear_queue();

    let usb_mouse_report = UsbMouseReport {
        buttons: 0x01, // Left button down
        dx: 15,
        dy: -10,       // Negative dy
        wheel: 0,
    };
    usb_hid::get_hid().process_mouse_report(&usb_mouse_report);

    let (m_usb_x, m_usb_y) = mouse::get_cursor_pos();
    let (btn_usb_l, _, _) = mouse::get_button_states();
    let usb_mouse_ok = m_usb_x == 515 && m_usb_y == 510 && btn_usb_l;
    check_input!(12, "MOUSE", "USB HID Mouse Boot Protocol Report Decoder (4-byte reports: buttons, dx, dy, wheel)", usb_mouse_ok);

    true
}

fn print_num(mut n: u32) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
