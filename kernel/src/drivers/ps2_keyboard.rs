//! NovaOS Zero — Freestanding PS/2 Keyboard Driver (I/O Port 0x60)

use crate::arch::x86_64::io::inb;

const SCANCODE_MAP: [char; 58] = [
    '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0',
    '*', '\0', ' ',
];

pub fn init() {
    // PS/2 Keyboard is auto-detected on Port 0x60
}

pub fn on_keyboard_interrupt() {
    let scancode = unsafe { inb(0x60) };

    // Ignore key release events (bit 7 set)
    if (scancode & 0x80) == 0 {
        let code = scancode as usize;
        if code < SCANCODE_MAP.len() {
            let ch = SCANCODE_MAP[code];
            if ch != '\0' {
                crate::print!("{}", ch);
                crate::serial_print!("{}", ch);
            }
        }
    }

    crate::interrupts::pic::notify_eoi(crate::interrupts::pic::PIC_1_OFFSET + 1);
}
