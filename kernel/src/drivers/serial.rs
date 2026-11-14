//! NovaOS Apex — First-Party 16550 UART Serial Driver (#1)
//! Implements strict lifecycle (probe, init, start, stop) and 8N1 serial I/O.

use super::{Driver, DriverError, DriverState};
use core::arch::asm;
use core::ptr::addr_of_mut;

pub const COM1_BASE: u16 = 0x3F8;

pub struct Uart16550 {
    port: u16,
    state: DriverState,
}

static mut UART_DRIVER: Uart16550 = Uart16550 {
    port: COM1_BASE,
    state: DriverState::Uninitialized,
};

impl Uart16550 {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            state: DriverState::Uninitialized,
        }
    }

    #[inline(always)]
    unsafe fn inb(&self, offset: u16) -> u8 {
        let mut val: u8;
        asm!("in al, dx", in("dx") self.port + offset, out("al") val, options(nomem, nostack, preserves_flags));
        val
    }

    #[inline(always)]
    unsafe fn outb(&self, offset: u16, val: u8) {
        asm!("out dx, al", in("dx") self.port + offset, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

impl Driver for Uart16550 {
    fn name(&self) -> &'static str {
        "16550 UART Serial"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        unsafe {
            // Test scratch register (offset 7)
            self.outb(7, 0xAE);
            if self.inb(7) != 0xAE {
                return Ok(false);
            }
            self.outb(7, 0x55);
            if self.inb(7) != 0x55 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn init(&mut self) -> Result<(), DriverError> {
        if !self.probe()? {
            self.state = DriverState::Error;
            return Err(DriverError::DeviceNotFound);
        }

        unsafe {
            // Disable interrupts
            self.outb(1, 0x00);
            // Enable DLAB (set baud rate divisor)
            self.outb(3, 0x80);
            // Set divisor to 3 (38,400 baud)
            self.outb(0, 0x03);
            self.outb(1, 0x00);
            // 8 bits, no parity, one stop bit (8N1)
            self.outb(3, 0x03);
            // Enable FIFO, clear TX/RX queues, 14-byte threshold
            self.outb(2, 0xC7);
            // Enable IRQs, RTS/DSR set
            self.outb(4, 0x0B);
        }

        self.state = DriverState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        if self.state != DriverState::Initialized && self.state != DriverState::Stopped {
            return Err(DriverError::NotStarted);
        }
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        unsafe {
            // Disable FIFO and interrupts
            self.outb(1, 0x00);
            self.outb(2, 0x00);
        }
        self.state = DriverState::Stopped;
        Ok(())
    }

    fn state(&self) -> DriverState {
        self.state
    }
}

pub fn init() {
    let uart = unsafe { &mut *addr_of_mut!(UART_DRIVER) };
    if let Err(_) = uart.init() {
        crate::serial::write_str("[SERIAL WARN] UART probe failed; using fallback serial.\n");
    } else {
        let _ = uart.start();
        crate::serial::write_str("[DRIVER 1/10] 16550 UART Serial Driver initialized & started.\n");
    }
}

pub fn write_byte(b: u8) {
    let uart = unsafe { &*addr_of_mut!(UART_DRIVER) };
    unsafe {
        loop {
            // Wait for Line Status Register bit 5 (Empty Transmitter Holding Register)
            if (uart.inb(5) & 0x20) != 0 {
                break;
            }
            asm!("pause", options(nomem, nostack));
        }
        uart.outb(0, b);
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

pub fn read_byte() -> Option<u8> {
    let uart = unsafe { &*addr_of_mut!(UART_DRIVER) };
    unsafe {
        // Line Status Register bit 0: Data Ready
        if (uart.inb(5) & 0x01) != 0 {
            Some(uart.inb(0))
        } else {
            None
        }
    }
}
