//! NovaOS Apex — Early Boot UART 16550 Serial Driver (COM1 0x3F8)

use core::arch::asm;

pub const COM1_PORT: u16 = 0x3F8;

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

pub fn init() {
    unsafe {
        outb(COM1_PORT + 1, 0x00); // Disable interrupts
        outb(COM1_PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1_PORT + 0, 0x03); // Set divisor to 3 (38400 baud)
        outb(COM1_PORT + 1, 0x00); // High byte
        outb(COM1_PORT + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1_PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(COM1_PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

pub fn write_byte(b: u8) {
    unsafe {
        while (inb(COM1_PORT + 5) & 0x20) == 0 {
            asm!("pause", options(nomem, nostack));
        }
        outb(COM1_PORT, b);
    }
}

pub fn write_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            write_byte(b'\r');
        }
        write_byte(b);
    }
}
