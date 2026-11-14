//! NovaOS Apex — USB Human Interface Device (HID) Driver
//! Decodes USB Boot Protocol keyboard reports (8 bytes) and mouse reports (4 bytes),
//! translates USB keycodes to ASCII/KeyCodes, and feeds the desktop event subsystem.

use crate::drivers::keyboard::on_scancode_received;
use crate::drivers::mouse::on_mouse_packet;
use core::ptr::addr_of_mut;

// USB HID Modifiers (Byte 0 of Boot Keyboard Report)
pub const USB_MOD_LCTRL: u8 = 1 << 0;
pub const USB_MOD_LSHIFT: u8 = 1 << 1;
pub const USB_MOD_LALT: u8 = 1 << 2;
pub const USB_MOD_LGUI: u8 = 1 << 3;
pub const USB_MOD_RCTRL: u8 = 1 << 4;
pub const USB_MOD_RSHIFT: u8 = 1 << 5;
pub const USB_MOD_RALT: u8 = 1 << 6;
pub const USB_MOD_RGUI: u8 = 1 << 7;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct UsbKeyboardReport {
    pub modifiers: u8,
    pub reserved: u8,
    pub keycodes: [u8; 6],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct UsbMouseReport {
    pub buttons: u8, // bit 0: Left, bit 1: Right, bit 2: Middle
    pub dx: i8,
    pub dy: i8,
    pub wheel: i8,
}

pub struct UsbHidManager {
    pub last_modifiers: u8,
    pub last_keycodes: [u8; 6],
    pub mouse_buttons: u8,
    pub total_keyboard_events: u64,
    pub total_mouse_events: u64,
}

static mut USB_HID_INSTANCE: UsbHidManager = UsbHidManager {
    last_modifiers: 0,
    last_keycodes: [0; 6],
    mouse_buttons: 0,
    total_keyboard_events: 0,
    total_mouse_events: 0,
};

impl UsbHidManager {
    pub fn new() -> Self {
        Self {
            last_modifiers: 0,
            last_keycodes: [0; 6],
            mouse_buttons: 0,
            total_keyboard_events: 0,
            total_mouse_events: 0,
        }
    }

    /// Translates a standard USB HID Usage ID (Usage Page 0x07) to an ASCII character.
    pub fn hid_to_ascii(usage_id: u8, shift: bool) -> Option<char> {
        match usage_id {
            0x04..=0x1D => {
                // a-z
                let base = if shift { b'A' } else { b'a' };
                Some((base + (usage_id - 0x04)) as char)
            }
            0x1E..=0x27 => {
                // 1-9, 0
                let unshifted = [b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0'];
                let shifted = [b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')'];
                let idx = (usage_id - 0x1E) as usize;
                let c = if shift { shifted[idx] } else { unshifted[idx] };
                Some(c as char)
            }
            0x28 => Some('\n'),      // Enter
            0x2A => Some('\x08'),    // Backspace
            0x2B => Some('\t'),      // Tab
            0x2C => Some(' '),       // Space
            0x2D => Some(if shift { '_' } else { '-' }),
            0x2E => Some(if shift { '+' } else { '=' }),
            0x2F => Some(if shift { '{' } else { '[' }),
            0x30 => Some(if shift { '}' } else { ']' }),
            0x31 => Some(if shift { '|' } else { '\\' }),
            0x33 => Some(if shift { ':' } else { ';' }),
            0x34 => Some(if shift { '"' } else { '\'' }),
            0x35 => Some(if shift { '~' } else { '`' }),
            0x36 => Some(if shift { '<' } else { ',' }),
            0x37 => Some(if shift { '>' } else { '.' }),
            0x38 => Some(if shift { '?' } else { '/' }),
            _ => None,
        }
    }

    /// Translates USB HID Usage ID to PS/2 Set 1 Make scancode for unified input routing.
    pub fn hid_to_ps2_scancode(usage_id: u8) -> u8 {
        match usage_id {
            0x04 => 0x1E, // A
            0x05 => 0x30, // B
            0x06 => 0x2E, // C
            0x07 => 0x20, // D
            0x08 => 0x12, // E
            0x09 => 0x21, // F
            0x0A => 0x22, // G
            0x0B => 0x23, // H
            0x0C => 0x17, // I
            0x0D => 0x24, // J
            0x0E => 0x25, // K
            0x0F => 0x26, // L
            0x10 => 0x32, // M
            0x11 => 0x31, // N
            0x12 => 0x18, // O
            0x13 => 0x19, // P
            0x14 => 0x10, // Q
            0x15 => 0x13, // R
            0x16 => 0x1F, // S
            0x17 => 0x14, // T
            0x18 => 0x16, // U
            0x19 => 0x2F, // V
            0x1A => 0x11, // W
            0x1B => 0x2D, // X
            0x1C => 0x15, // Y
            0x1D => 0x2C, // Z
            0x1E => 0x02, // 1
            0x1F => 0x03, // 2
            0x20 => 0x04, // 3
            0x21 => 0x05, // 4
            0x22 => 0x06, // 5
            0x23 => 0x07, // 6
            0x24 => 0x08, // 7
            0x25 => 0x09, // 8
            0x26 => 0x0A, // 9
            0x27 => 0x0B, // 0
            0x28 => 0x1C, // Enter
            0x29 => 0x01, // Escape
            0x2A => 0x0E, // Backspace
            0x2B => 0x0F, // Tab
            0x2C => 0x39, // Space
            _ => 0,
        }
    }

    /// Processes an incoming 8-byte USB Keyboard report.
    pub fn process_keyboard_report(&mut self, report: &UsbKeyboardReport) {
        self.total_keyboard_events += 1;
        let _shift = (report.modifiers & (USB_MOD_LSHIFT | USB_MOD_RSHIFT)) != 0;

        // Check for new key presses
        for &k in &report.keycodes {
            if k != 0 && !self.last_keycodes.contains(&k) {
                // Key pressed! Dispatch into input pipeline
                let scancode = Self::hid_to_ps2_scancode(k);
                if scancode != 0 {
                    on_scancode_received(scancode);
                }
            }
        }

        // Check for released keys
        for &k in &self.last_keycodes {
            if k != 0 && !report.keycodes.contains(&k) {
                // Key released! (Scancode | 0x80)
                let scancode = Self::hid_to_ps2_scancode(k);
                if scancode != 0 {
                    on_scancode_received(scancode | 0x80);
                }
            }
        }

        self.last_modifiers = report.modifiers;
        self.last_keycodes = report.keycodes;
    }

    /// Processes an incoming 4-byte USB Mouse report and forwards to system mouse driver.
    pub fn process_mouse_report(&mut self, report: &UsbMouseReport) {
        self.total_mouse_events += 1;
        self.mouse_buttons = report.buttons;

        // Construct standard 3-byte PS/2 packet:
        // Byte 0: [Yovfl, Xovfl, Ysign, Xsign, 1, Mbtn, Rbtn, Lbtn]
        let mut b0 = 0x08u8;
        if (report.buttons & 0x01) != 0 { b0 |= 0x01; } // Left
        if (report.buttons & 0x02) != 0 { b0 |= 0x02; } // Right
        if (report.buttons & 0x04) != 0 { b0 |= 0x04; } // Middle
        if report.dx < 0 { b0 |= 0x10; }
        if report.dy < 0 { b0 |= 0x20; }

        let b1 = report.dx as u8;
        let b2 = report.dy as u8;

        on_mouse_packet([b0, b1, b2]);
    }
}

pub fn init() {
    unsafe {
        USB_HID_INSTANCE = UsbHidManager::new();
    }
}

pub fn get_hid() -> &'static mut UsbHidManager {
    unsafe { &mut *addr_of_mut!(USB_HID_INSTANCE) }
}
