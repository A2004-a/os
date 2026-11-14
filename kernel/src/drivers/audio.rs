//! NovaOS Apex — Basic Audio Abstraction Driver (#10)
//! PC Speaker square-wave tone generation, sound synthesis, and audio device probing.

use super::{Driver, DriverError, DriverState};
use crate::serial;
use core::arch::asm;
use core::ptr::addr_of_mut;

pub const SPEAKER_PORT: u16 = 0x61;
pub const PIT_CHANNEL_2: u16 = 0x42;
pub const PIT_COMMAND_PORT: u16 = 0x43;
pub const PIT_BASE_FREQ: u32 = 1_193_182;

pub struct AudioDriver {
    pub is_playing: bool,
    pub current_freq: u32,
    state: DriverState,
}

static mut AUDIO_DRIVER: AudioDriver = AudioDriver {
    is_playing: false,
    current_freq: 0,
    state: DriverState::Uninitialized,
};

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let mut val: u8;
    asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    val
}

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

impl Driver for AudioDriver {
    fn name(&self) -> &'static str {
        "PC Speaker / Hardware Audio"
    }

    fn probe(&mut self) -> Result<bool, DriverError> {
        // PC speaker control register at port 0x61 is always present on x86 platforms
        unsafe {
            let val = inb(SPEAKER_PORT);
            Ok(val != 0xFF)
        }
    }

    fn init(&mut self) -> Result<(), DriverError> {
        self.probe()?;
        self.stop_tone();
        self.state = DriverState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DriverError> {
        self.state = DriverState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), DriverError> {
        self.stop_tone();
        self.state = DriverState::Stopped;
        Ok(())
    }

    fn state(&self) -> DriverState {
        self.state
    }
}

impl AudioDriver {
    pub fn play_tone(&mut self, freq_hz: u32) {
        if freq_hz == 0 {
            self.stop_tone();
            return;
        }

        let divisor = (PIT_BASE_FREQ / freq_hz) as u16;

        unsafe {
            // Program Channel 2: Mode 3 (Square Wave), 16-bit binary
            outb(PIT_COMMAND_PORT, 0xB6);
            outb(PIT_CHANNEL_2, (divisor & 0xFF) as u8);
            outb(PIT_CHANNEL_2, ((divisor >> 8) & 0xFF) as u8);

            // Enable PC Speaker output (bits 0 and 1 of port 0x61)
            let prev = inb(SPEAKER_PORT);
            if (prev & 0x03) != 0x03 {
                outb(SPEAKER_PORT, prev | 0x03);
            }
        }

        self.is_playing = true;
        self.current_freq = freq_hz;
    }

    pub fn stop_tone(&mut self) {
        unsafe {
            let prev = inb(SPEAKER_PORT);
            outb(SPEAKER_PORT, prev & !0x03);
        }
        self.is_playing = false;
        self.current_freq = 0;
    }
}

pub fn init() {
    let audio = unsafe { &mut *addr_of_mut!(AUDIO_DRIVER) };
    if let Ok(_) = audio.init() {
        let _ = audio.start();
        serial::write_str("[DRIVER 10/10] Basic Audio Driver active (PC Speaker synthesizer ready).\n");
    }
}

pub fn beep(freq_hz: u32, duration_ms: u64) {
    let audio = unsafe { &mut *addr_of_mut!(AUDIO_DRIVER) };
    audio.play_tone(freq_hz);
    super::timer::sleep_ms(duration_ms);
    audio.stop_tone();
}

pub fn play_beep(freq_hz: u32, duration_ms: u32) {
    beep(freq_hz, duration_ms as u64);
}

pub fn play_tone(freq_hz: u32) {
    let audio = unsafe { &mut *addr_of_mut!(AUDIO_DRIVER) };
    audio.play_tone(freq_hz);
}

pub fn stop_tone() {
    let audio = unsafe { &mut *addr_of_mut!(AUDIO_DRIVER) };
    audio.stop_tone();
}
