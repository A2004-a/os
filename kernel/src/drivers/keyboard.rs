//! NovaOS Apex — PS/2 Keyboard Driver (#5)
//! Scancode Set 1/2 decoder, modifier tracking, and circular key event queue.

use super::{Driver, DriverError, DriverState};
use core::arch::asm;
use core::ptr::addr_of_mut;
use crate::serial;

pub const KEYBOARD_DATA_PORT: u16 = 0x60;
pub const KEYBOARD_STATUS_PORT: u16 = 0x64;

pub const EVENT_QUEUE_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub scancode: u8,
    pub char_code: Option<char>,
    pub pressed: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

pub struct Ps2Keyboard {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
    queue: [Option<KeyEvent>; EVENT_QUEUE_SIZE],
    read_idx: usize,
    write_idx: usize,
    state: DriverState,
}

static mut KEYBOARD_DRIVER: Ps2Keyboard = Ps2Keyboard {
    shift_pressed: false,
    ctrl_pressed: false,
    alt_pressed: false,
    caps_lock: false,
    queue: [None; EVENT_QUEUE_SIZE],
    read_idx: 0,
    write_idx: 0,
    state: DriverState::Uninitialized,
};

impl Driver for Ps2Keyboard {
    fn name(&self) -> &'static str {
        "PS/2 Keyboard"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        unsafe {
            // Read status register
            let mut status: u8;
            asm!("in al, dx", in("dx") KEYBOARD_STATUS_PORT, out("al") status, options(nomem, nostack, preserves_flags));
            // 0xFF means floating bus / no device
            Ok(status != 0xFF)
        }
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if !self.probe()? {
            self.state = DriverState::Error;
            return Err(DriverError::DeviceNotFound);
        }

        unsafe {
            // Flush output buffer
            let mut val: u8;
            asm!("in al, dx", in("dx") KEYBOARD_DATA_PORT, out("al") val, options(nomem, nostack, preserves_flags));
            let _ = val;
            // Enable keyboard interface port
            asm!("out dx, al", in("dx") KEYBOARD_STATUS_PORT, in("al") 0xAEu8, options(nomem, nostack, preserves_flags));
        }

        self.state = DriverState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Stopped;
        Ok(())
    }

    fn state(&self) -> DriverState {
        self.state
    }
}

pub fn init() {
    let kbd = unsafe { &mut *addr_of_mut!(KEYBOARD_DRIVER) };
    if let Ok(_) = kbd.init() {
        let _ = kbd.start();
        serial::write_str("[DRIVER 5/10] PS/2 Keyboard Driver active (Scancode Set 1 & modifiers ready).\n");
    }
}

pub fn on_scancode_received(scancode: u8) {
    let kbd = unsafe { &mut *addr_of_mut!(KEYBOARD_DRIVER) };

    let pressed = (scancode & 0x80) == 0;
    let code = scancode & 0x7F;

    match code {
        0x2A | 0x36 => kbd.shift_pressed = pressed, // Left / Right Shift
        0x1D => kbd.ctrl_pressed = pressed,          // Left Ctrl
        0x38 => kbd.alt_pressed = pressed,           // Left Alt
        0x3A if pressed => kbd.caps_lock = !kbd.caps_lock, // Caps Lock toggle
        _ => {}
    }

    let is_shifted = kbd.shift_pressed ^ kbd.caps_lock;
    let char_code = if pressed { decode_ascii(code, is_shifted) } else { None };

    let event = KeyEvent {
        scancode,
        char_code,
        pressed,
        shift: kbd.shift_pressed,
        ctrl: kbd.ctrl_pressed,
        alt: kbd.alt_pressed,
    };

    let next_write = (kbd.write_idx + 1) % EVENT_QUEUE_SIZE;
    if next_write != kbd.read_idx {
        kbd.queue[kbd.write_idx] = Some(event);
        kbd.write_idx = next_write;
    }
}

pub fn on_keyboard_irq() {
    let mut scancode: u8;
    unsafe {
        asm!("in al, dx", in("dx") KEYBOARD_DATA_PORT, out("al") scancode, options(nomem, nostack, preserves_flags));
    }

    on_scancode_received(scancode);
    super::pic::send_eoi(1);
}

pub fn read_key() -> Option<KeyEvent> {
    let kbd = unsafe { &mut *addr_of_mut!(KEYBOARD_DRIVER) };
    if kbd.read_idx == kbd.write_idx {
        return None;
    }
    let event = kbd.queue[kbd.read_idx].take();
    kbd.read_idx = (kbd.read_idx + 1) % EVENT_QUEUE_SIZE;
    event
}

pub fn read_char() -> Option<char> {
    while let Some(key) = read_key() {
        if key.pressed && key.char_code.is_some() {
            return key.char_code;
        }
    }
    None
}

fn decode_ascii(code: u8, shifted: bool) -> Option<char> {
    match (code, shifted) {
        (0x02, false) => Some('1'), (0x02, true) => Some('!'),
        (0x03, false) => Some('2'), (0x03, true) => Some('@'),
        (0x04, false) => Some('3'), (0x04, true) => Some('#'),
        (0x05, false) => Some('4'), (0x05, true) => Some('$'),
        (0x06, false) => Some('5'), (0x06, true) => Some('%'),
        (0x07, false) => Some('6'), (0x07, true) => Some('^'),
        (0x08, false) => Some('7'), (0x08, true) => Some('&'),
        (0x09, false) => Some('8'), (0x09, true) => Some('*'),
        (0x0A, false) => Some('9'), (0x0A, true) => Some('('),
        (0x0B, false) => Some('0'), (0x0B, true) => Some(')'),
        (0x0C, false) => Some('-'), (0x0C, true) => Some('_'),
        (0x0D, false) => Some('='), (0x0D, true) => Some('+'),
        (0x0E, _) => Some('\x08'), // Backspace
        (0x0F, _) => Some('\t'),   // Tab
        (0x10, false) => Some('q'), (0x10, true) => Some('Q'),
        (0x11, false) => Some('w'), (0x11, true) => Some('W'),
        (0x12, false) => Some('e'), (0x12, true) => Some('E'),
        (0x13, false) => Some('r'), (0x13, true) => Some('R'),
        (0x14, false) => Some('t'), (0x14, true) => Some('T'),
        (0x15, false) => Some('y'), (0x15, true) => Some('Y'),
        (0x16, false) => Some('u'), (0x16, true) => Some('U'),
        (0x17, false) => Some('i'), (0x17, true) => Some('I'),
        (0x18, false) => Some('o'), (0x18, true) => Some('O'),
        (0x19, false) => Some('p'), (0x19, true) => Some('P'),
        (0x1A, false) => Some('['), (0x1A, true) => Some('{'),
        (0x1B, false) => Some(']'), (0x1B, true) => Some('}'),
        (0x1C, _) => Some('\n'),   // Enter
        (0x1E, false) => Some('a'), (0x1E, true) => Some('A'),
        (0x1F, false) => Some('s'), (0x1F, true) => Some('S'),
        (0x20, false) => Some('d'), (0x20, true) => Some('D'),
        (0x21, false) => Some('f'), (0x21, true) => Some('F'),
        (0x22, false) => Some('g'), (0x22, true) => Some('G'),
        (0x23, false) => Some('h'), (0x23, true) => Some('H'),
        (0x24, false) => Some('j'), (0x24, true) => Some('J'),
        (0x25, false) => Some('k'), (0x25, true) => Some('K'),
        (0x26, false) => Some('l'), (0x26, true) => Some('L'),
        (0x27, false) => Some(';'), (0x27, true) => Some(':'),
        (0x28, false) => Some('\''),(0x28, true) => Some('"'),
        (0x29, false) => Some('`'), (0x29, true) => Some('~'),
        (0x2B, false) => Some('\\'),(0x2B, true) => Some('|'),
        (0x2C, false) => Some('z'), (0x2C, true) => Some('Z'),
        (0x2D, false) => Some('x'), (0x2D, true) => Some('X'),
        (0x2E, false) => Some('c'), (0x2E, true) => Some('C'),
        (0x2F, false) => Some('v'), (0x2F, true) => Some('V'),
        (0x30, false) => Some('b'), (0x30, true) => Some('B'),
        (0x31, false) => Some('n'), (0x31, true) => Some('N'),
        (0x32, false) => Some('m'), (0x32, true) => Some('M'),
        (0x33, false) => Some(','), (0x33, true) => Some('<'),
        (0x34, false) => Some('.'), (0x34, true) => Some('>'),
        (0x35, false) => Some('/'), (0x35, true) => Some('?'),
        (0x39, _) => Some(' '),    // Space
        _ => None,
    }
}
