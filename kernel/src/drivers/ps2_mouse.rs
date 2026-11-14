//! NovaOS Zero — Freestanding PS/2 Mouse Driver (Auxiliary Port 0x60/0x64, IRQ12)

use crate::arch::x86_64::io::{inb, io_wait, outb};
use crate::sync::Spinlock;

const PS2_DATA_PORT: u16    = 0x60;
const PS2_STATUS_PORT: u16  = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

pub struct Ps2Mouse {
    cycle: u8,
    bytes: [u8; 3],
    pub state: MouseState,
}

impl Ps2Mouse {
    pub const fn new() -> Self {
        Self {
            cycle: 0,
            bytes: [0; 3],
            state: MouseState {
                x: 0,
                y: 0,
                left_button: false,
                right_button: false,
                middle_button: false,
            },
        }
    }

    pub fn process_byte(&mut self, byte: u8) -> Option<MouseState> {
        match self.cycle {
            0 => {
                // Bit 3 must always be 1 in a valid 3-byte PS/2 packet header
                if (byte & 0x08) != 0 {
                    self.bytes[0] = byte;
                    self.cycle = 1;
                }
            }
            1 => {
                self.bytes[1] = byte;
                self.cycle = 2;
            }
            2 => {
                self.bytes[2] = byte;
                self.cycle = 0;

                let flags = self.bytes[0];
                let mut dx = self.bytes[1] as i32;
                let mut dy = self.bytes[2] as i32;

                // Sign extension for 9-bit relative motion
                if (flags & 0x10) != 0 {
                    dx |= !0xFF;
                }
                if (flags & 0x20) != 0 {
                    dy |= !0xFF;
                }

                // Invert dy for standard screen coordinates (y increases downward)
                self.state.x = (self.state.x + dx).clamp(0, 1920);
                self.state.y = (self.state.y - dy).clamp(0, 1080);
                self.state.left_button = (flags & 0x01) != 0;
                self.state.right_button = (flags & 0x02) != 0;
                self.state.middle_button = (flags & 0x04) != 0;

                return Some(self.state);
            }
            _ => self.cycle = 0,
        }

        None
    }
}

pub static MOUSE: Spinlock<Ps2Mouse> = Spinlock::new(Ps2Mouse::new());

unsafe fn wait_write() {
    let mut timeout = 100_000;
    while timeout > 0 && (inb(PS2_STATUS_PORT) & 0x02) != 0 {
        core::hint::spin_loop();
        timeout -= 1;
    }
}

unsafe fn wait_read() {
    let mut timeout = 100_000;
    while timeout > 0 && (inb(PS2_STATUS_PORT) & 0x01) == 0 {
        core::hint::spin_loop();
        timeout -= 1;
    }
}

pub fn init() {
    unsafe {
        // 1. Enable auxiliary mouse port
        wait_write();
        outb(PS2_COMMAND_PORT, 0xA8);
        io_wait();

        // 2. Enable IRQ12 interrupt in PS/2 Controller configuration byte
        wait_write();
        outb(PS2_COMMAND_PORT, 0x20); // Read command
        wait_read();
        let status = inb(PS2_DATA_PORT) | 0x02; // Bit 1 = enable IRQ12

        wait_write();
        outb(PS2_COMMAND_PORT, 0x60); // Write command
        wait_write();
        outb(PS2_DATA_PORT, status);

        // 3. Send default settings command (0xF6) to mouse
        wait_write();
        outb(PS2_COMMAND_PORT, 0xD4); // Route next byte to mouse
        wait_write();
        outb(PS2_DATA_PORT, 0xF6);
        wait_read();
        let _ack1 = inb(PS2_DATA_PORT);

        // 4. Enable data streaming reporting (0xF4)
        wait_write();
        outb(PS2_COMMAND_PORT, 0xD4);
        wait_write();
        outb(PS2_DATA_PORT, 0xF4);
        wait_read();
        let _ack2 = inb(PS2_DATA_PORT);
    }
}

pub fn on_mouse_interrupt() {
    let byte = unsafe { inb(PS2_DATA_PORT) };
    let _ = MOUSE.lock().process_byte(byte);

    // EOI to Slave PIC (IRQ12 is on Slave PIC line 4 = interrupt vector 44)
    crate::interrupts::pic::notify_eoi(crate::interrupts::pic::PIC_2_OFFSET + 4);
}
