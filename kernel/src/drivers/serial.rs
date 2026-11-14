//! NovaOS Zero — Freestanding UART 16550 Serial Driver (COM1 at Port 0x3F8)

use core::fmt;
use crate::arch::x86_64::io::{inb, outb};
use crate::sync::Spinlock;

pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub const unsafe fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            outb(self.port + 1, 0x00);
            // Enable DLAB (set baud rate divisor)
            outb(self.port + 3, 0x80);
            // Set divisor to 3 (lo byte) 38400 baud
            outb(self.port + 0, 0x03);
            outb(self.port + 1, 0x00); // (hi byte)
            // 8 bits, no parity, one stop bit
            outb(self.port + 3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 2, 0xC7);
            // IRQs enabled, RTS/DSR set
            outb(self.port + 4, 0x0B);
        }
    }

    fn line_status(&self) -> u8 {
        unsafe { inb(self.port + 5) }
    }

    pub fn send(&mut self, data: u8) {
        // Wait until transmit buffer is empty (bit 5)
        while (self.line_status() & 0x20) == 0 {
            core::hint::spin_loop();
        }
        unsafe {
            outb(self.port, data);
        }
    }

    pub fn receive(&mut self) -> Option<u8> {
        // Data ready (bit 0)
        if (self.line_status() & 1) != 0 {
            Some(unsafe { inb(self.port) })
        } else {
            None
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send(b'\r');
            }
            self.send(byte);
        }
        Ok(())
    }
}

pub static SERIAL1: Spinlock<SerialPort> = Spinlock::new(unsafe { SerialPort::new(0x3F8) });

pub fn init() {
    SERIAL1.lock().init();
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut serial = SERIAL1.lock();
    let _ = serial.write_fmt(args);
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
