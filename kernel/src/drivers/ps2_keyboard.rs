//! PS/2 Keyboard Driver (I/O Port 0x60)

use crate::arch::x86_64::io::inb;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;

static KEYBOARD: Mutex<Option<Keyboard<layouts::Us104Key, ScancodeSet1>>> = Mutex::new(None);

pub fn init() {
    let kb = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    *KEYBOARD.lock() = Some(kb);
}

pub fn on_keyboard_interrupt() {
    let scancode = unsafe { inb(0x60) };

    if let Some(ref mut kb) = *KEYBOARD.lock() {
        if let Ok(Some(key_event)) = kb.add_byte(scancode) {
            if let Some(key) = kb.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c) => {
                        crate::print!("{}", c);
                        crate::serial_print!("{}", c);
                    }
                    DecodedKey::RawKey(k) => {
                        crate::serial_println!("[KEY: {:?}]", k);
                    }
                }
            }
        }
    }

    crate::interrupts::pic::notify_eoi(crate::interrupts::pic::PIC_1_OFFSET + 1);
}
