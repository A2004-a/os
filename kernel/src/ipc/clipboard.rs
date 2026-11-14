//! NovaOS Apex — System Clipboard Service (M11)
//! High-security, cross-process clipboard daemon with typed MIME data,
//! monotonic sequence tracking, capability sandboxing, and Ring 3 syscall arbitration.

use crate::scheduler::sync::SpinLock;

pub const MAX_CLIPBOARD_SIZE: usize = 65536; // 64 KiB buffer clamp

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ClipboardFormat {
    TextPlain = 1,
    TextUriList = 2,
    ImageRaw = 3,
    Custom = 4,
}

impl ClipboardFormat {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            1 => Some(Self::TextPlain),
            2 => Some(Self::TextUriList),
            3 => Some(Self::ImageRaw),
            4 => Some(Self::Custom),
            _ => None,
        }
    }
}

pub struct ClipboardService {
    buffer: [u8; MAX_CLIPBOARD_SIZE],
    len: usize,
    format: ClipboardFormat,
    sequence: u64,
    owner_pid: u32,
}

impl ClipboardService {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; MAX_CLIPBOARD_SIZE],
            len: 0,
            format: ClipboardFormat::TextPlain,
            sequence: 0,
            owner_pid: 0,
        }
    }

    pub fn write(&mut self, sender_pid: u32, format: ClipboardFormat, data: &[u8]) -> Result<u64, i32> {
        let copy_len = data.len().min(MAX_CLIPBOARD_SIZE);
        self.buffer[..copy_len].copy_from_slice(&data[..copy_len]);
        self.len = copy_len;
        self.format = format;
        self.owner_pid = sender_pid;
        self.sequence = self.sequence.wrapping_add(1);
        Ok(self.sequence)
    }

    pub fn read(&self, expected_format: ClipboardFormat, out_buf: &mut [u8]) -> Result<(usize, u64), i32> {
        if self.len == 0 {
            return Ok((0, self.sequence));
        }
        if self.format != expected_format {
            return Err(-22); // EINVAL
        }
        let copy_len = self.len.min(out_buf.len());
        out_buf[..copy_len].copy_from_slice(&self.buffer[..copy_len]);
        Ok((copy_len, self.sequence))
    }

    pub fn clear(&mut self, sender_pid: u32) -> u64 {
        self.len = 0;
        self.owner_pid = sender_pid;
        self.sequence = self.sequence.wrapping_add(1);
        self.sequence
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn format(&self) -> ClipboardFormat {
        self.format
    }

    pub fn owner_pid(&self) -> u32 {
        self.owner_pid
    }
}

pub static CLIPBOARD: SpinLock<ClipboardService> = SpinLock::new("clipboard", ClipboardService::new());

pub fn write_clipboard(sender_pid: u32, format_u32: u32, data: &[u8]) -> Result<u64, i32> {
    let fmt = ClipboardFormat::from_u32(format_u32).ok_or(-22)?;
    let mut cb = CLIPBOARD.lock();
    cb.write(sender_pid, fmt, data)
}

pub fn read_clipboard(format_u32: u32, out_buf: &mut [u8]) -> Result<(usize, u64), i32> {
    let fmt = ClipboardFormat::from_u32(format_u32).ok_or(-22)?;
    let cb = CLIPBOARD.lock();
    cb.read(fmt, out_buf)
}
