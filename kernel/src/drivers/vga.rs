//! NovaOS Zero — Freestanding VGA 80x25 Text Mode Console Driver (0xb8000)

use core::fmt;
use crate::sync::Spinlock;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_MEMORY: usize = 0xb8000;

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer_base: usize,
}

impl Writer {
    const fn new() -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::Green, Color::Black),
            buffer_base: VGA_MEMORY,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                unsafe {
                    let ptr = (self.buffer_base as *mut ScreenChar).add(row * BUFFER_WIDTH + col);
                    core::ptr::write_volatile(ptr, ScreenChar {
                        ascii_character: byte,
                        color_code,
                    });
                }
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        unsafe {
            let base = self.buffer_base as *mut ScreenChar;
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let src = base.add(row * BUFFER_WIDTH + col);
                    let dst = base.add((row - 1) * BUFFER_WIDTH + col);
                    let ch = core::ptr::read_volatile(src);
                    core::ptr::write_volatile(dst, ch);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        unsafe {
            let base = self.buffer_base as *mut ScreenChar;
            for col in 0..BUFFER_WIDTH {
                core::ptr::write_volatile(base.add(row * BUFFER_WIDTH + col), blank);
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub static WRITER: Spinlock<Writer> = Spinlock::new(Writer::new());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    let _ = writer.write_fmt(args);
}
