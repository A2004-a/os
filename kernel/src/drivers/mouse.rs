//! NovaOS Apex — PS/2 Mouse Driver (#6)
//! Standard 3-byte PS/2 packet decoding, screen boundary clamping, and button state tracking.

use super::{Driver, DriverError, DriverState};
use core::arch::asm;
use core::ptr::addr_of_mut;
use crate::serial;

pub const MOUSE_DATA_PORT: u16 = 0x60;
pub const MOUSE_STATUS_PORT: u16 = 0x64;
pub const MOUSE_QUEUE_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

pub struct Ps2Mouse {
    pub x: i32,
    pub y: i32,
    max_x: i32,
    max_y: i32,
    packet_idx: usize,
    packet: [u8; 3],
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    queue: [Option<MouseEvent>; MOUSE_QUEUE_SIZE],
    read_idx: usize,
    write_idx: usize,
    state: DriverState,
}

static mut MOUSE_DRIVER: Ps2Mouse = Ps2Mouse {
    x: 0,
    y: 0,
    max_x: 1024,
    max_y: 768,
    packet_idx: 0,
    packet: [0u8; 3],
    left_button: false,
    right_button: false,
    middle_button: false,
    queue: [None; MOUSE_QUEUE_SIZE],
    read_idx: 0,
    write_idx: 0,
    state: DriverState::Uninitialized,
};

#[inline(always)]
unsafe fn mouse_wait(is_write: bool) {
    for _ in 0..1000 {
        let mut status: u8;
        asm!("in al, dx", in("dx") MOUSE_STATUS_PORT, out("al") status, options(nomem, nostack, preserves_flags));
        if is_write {
            if (status & 0x02) == 0 { return; } // Ready for write
        } else {
            if (status & 0x01) != 0 { return; } // Ready for read
        }
        asm!("pause", options(nomem, nostack));
    }
}

#[inline(always)]
unsafe fn mouse_write(val: u8) {
    mouse_wait(true);
    asm!("out dx, al", in("dx") MOUSE_STATUS_PORT, in("al") 0xD4u8, options(nomem, nostack, preserves_flags));
    mouse_wait(true);
    asm!("out dx, al", in("dx") MOUSE_DATA_PORT, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline(always)]
unsafe fn mouse_read() -> u8 {
    mouse_wait(false);
    let mut val: u8;
    asm!("in al, dx", in("dx") MOUSE_DATA_PORT, out("al") val, options(nomem, nostack, preserves_flags));
    val
}

impl Driver for Ps2Mouse {
    fn name(&self) -> &'static str {
        "PS/2 Mouse"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        unsafe {
            let mut status: u8;
            asm!("in al, dx", in("dx") MOUSE_STATUS_PORT, out("al") status, options(nomem, nostack, preserves_flags));
            Ok(status != 0xFF)
        }
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if !self.probe()? {
            self.state = DriverState::Error;
            return Err(DriverError::DeviceNotFound);
        }

        unsafe {
            // Enable auxiliary PS/2 device port (Mouse)
            mouse_wait(true);
            asm!("out dx, al", in("dx") MOUSE_STATUS_PORT, in("al") 0xA8u8, options(nomem, nostack, preserves_flags));

            // Enable interrupts in command byte
            mouse_wait(true);
            asm!("out dx, al", in("dx") MOUSE_STATUS_PORT, in("al") 0x20u8, options(nomem, nostack, preserves_flags));
            mouse_wait(false);
            let mut status: u8;
            asm!("in al, dx", in("dx") MOUSE_DATA_PORT, out("al") status, options(nomem, nostack, preserves_flags));
            status |= 0x02; // Enable auxiliary interrupt (IRQ12)
            mouse_wait(true);
            asm!("out dx, al", in("dx") MOUSE_STATUS_PORT, in("al") 0x60u8, options(nomem, nostack, preserves_flags));
            mouse_wait(true);
            asm!("out dx, al", in("dx") MOUSE_DATA_PORT, in("al") status, options(nomem, nostack, preserves_flags));

            // Set default settings
            mouse_write(0xF6);
            let _ = mouse_read();

            // Enable packet streaming
            mouse_write(0xF4);
            let _ = mouse_read();
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

pub fn init(screen_w: u32, screen_h: u32) {
    let mouse = unsafe { &mut *addr_of_mut!(MOUSE_DRIVER) };
    mouse.max_x = if screen_w > 0 { screen_w as i32 } else { 1024 };
    mouse.max_y = if screen_h > 0 { screen_h as i32 } else { 768 };
    mouse.x = mouse.max_x / 2;
    mouse.y = mouse.max_y / 2;

    if let Ok(_) = mouse.init() {
        let _ = mouse.start();
        serial::write_str("[DRIVER 6/10] PS/2 Mouse Driver active (3-byte streaming packet mode).\n");
    }
}

pub fn on_mouse_packet(packet: [u8; 3]) {
    let mouse = unsafe { &mut *addr_of_mut!(MOUSE_DRIVER) };
    let flags = packet[0];
    let raw_x = packet[1];
    let raw_y = packet[2];

    // Parse buttons
    mouse.left_button = (flags & 0x01) != 0;
    mouse.right_button = (flags & 0x02) != 0;
    mouse.middle_button = (flags & 0x04) != 0;

    // Parse deltas with sign extension
    let mut dx = raw_x as i32;
    if (flags & 0x10) != 0 {
        dx |= !0xFF; // Negative sign extension
    }
    let mut dy = raw_y as i32;
    if (flags & 0x20) != 0 {
        dy |= !0xFF;
    }
    // Invert Y delta for screen coordinates (PS/2 Y points upward)
    dy = -dy;

    // Update cursor coordinates clamped to display bounds
    mouse.x = (mouse.x + dx).clamp(0, mouse.max_x.saturating_sub(1));
    mouse.y = (mouse.y + dy).clamp(0, mouse.max_y.saturating_sub(1));

    let event = MouseEvent {
        x: mouse.x,
        y: mouse.y,
        dx,
        dy,
        left_button: mouse.left_button,
        right_button: mouse.right_button,
        middle_button: mouse.middle_button,
    };

    let next_write = (mouse.write_idx + 1) % MOUSE_QUEUE_SIZE;
    if next_write != mouse.read_idx {
        mouse.queue[mouse.write_idx] = Some(event);
        mouse.write_idx = next_write;
    }
}

pub fn on_mouse_irq() {
    let mut b: u8;
    unsafe {
        asm!("in al, dx", in("dx") MOUSE_DATA_PORT, out("al") b, options(nomem, nostack, preserves_flags));
    }

    let mouse = unsafe { &mut *addr_of_mut!(MOUSE_DRIVER) };

    // Align with packet start (bit 3 of byte 0 must be 1 in standard PS/2 protocol)
    if mouse.packet_idx == 0 && (b & 0x08) == 0 {
        super::pic::send_eoi(12);
        return;
    }

    mouse.packet[mouse.packet_idx] = b;
    mouse.packet_idx += 1;

    if mouse.packet_idx == 3 {
        // Complete 3-byte packet received
        mouse.packet_idx = 0;
        let pkt = mouse.packet;
        on_mouse_packet(pkt);
    }

    super::pic::send_eoi(12);
}

pub fn get_cursor_pos() -> (i32, i32) {
    let mouse = unsafe { &*addr_of_mut!(MOUSE_DRIVER) };
    (mouse.x, mouse.y)
}

pub fn read_mouse_event() -> Option<MouseEvent> {
    let mouse = unsafe { &mut *addr_of_mut!(MOUSE_DRIVER) };
    if mouse.read_idx == mouse.write_idx {
        return None;
    }
    let event = mouse.queue[mouse.read_idx].take();
    mouse.read_idx = (mouse.read_idx + 1) % MOUSE_QUEUE_SIZE;
    event
}
