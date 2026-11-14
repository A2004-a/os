//! NovaOS Zero — Intel High Definition Audio (HDA / Azalia) & AC97 Driver
//! Class 04h (Multimedia), Subclass 03h (Audio Controller)

pub const PCI_CLASS_MULTIMEDIA: u8 = 0x04;
pub const PCI_SUBCLASS_HDA: u8     = 0x03;

pub const HDA_GCTL_CRST: u32       = 1 << 0; // Controller Reset
pub const HDA_SD_CTL_RUN: u32      = 1 << 1; // Stream Run/Stop
pub const HDA_SD_CTL_IOCE: u32     = 1 << 2; // Interrupt on Completion Enable

pub const HDA_FMT_BASE_48KHZ: u16  = 0 << 14;
pub const HDA_FMT_BASE_44KHZ: u16  = 1 << 14;
pub const HDA_FMT_16BIT: u16       = 1 << 4;
pub const HDA_FMT_24BIT: u16       = 3 << 4;
pub const HDA_FMT_32BIT: u16       = 4 << 4;

/// Intel HDA Buffer Descriptor List (BDL) Entry (16 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferDescriptorEntry {
    pub buffer_address: u64, // 128-byte aligned physical address
    pub length_bytes: u32,
    pub ioc: u32,            // Bit 0: Interrupt On Completion
}

impl BufferDescriptorEntry {
    pub fn new(buffer_phys: u64, length: u32, ioc: bool) -> Self {
        BufferDescriptorEntry {
            buffer_address: buffer_phys,
            length_bytes: length,
            ioc: if ioc { 1 } else { 0 },
        }
    }
}

pub struct HdaStream {
    pub stream_id: u8,
    pub is_running: bool,
    pub format_tag: u16,
    pub sample_rate: u32,
    pub channels: u8,
}

pub struct IntelHdaController {
    pub mmio_base: u64,
    pub is_ready: bool,
    pub streams: [HdaStream; 4],
}

impl IntelHdaController {
    pub fn new(mmio_base: u64) -> Self {
        IntelHdaController {
            mmio_base,
            is_ready: true,
            streams: [
                HdaStream { stream_id: 1, is_running: false, format_tag: 0, sample_rate: 48000, channels: 2 },
                HdaStream { stream_id: 2, is_running: false, format_tag: 0, sample_rate: 48000, channels: 2 },
                HdaStream { stream_id: 3, is_running: false, format_tag: 0, sample_rate: 48000, channels: 2 },
                HdaStream { stream_id: 4, is_running: false, format_tag: 0, sample_rate: 48000, channels: 2 },
            ],
        }
    }

    /// Encode sample rate, channels, and bit depth into HDA 16-bit format tag
    pub fn encode_format(sample_rate: u32, channels: u8, bit_depth: u8) -> Result<u16, &'static str> {
        let base = match sample_rate {
            48000 => HDA_FMT_BASE_48KHZ,
            44100 => HDA_FMT_BASE_44KHZ,
            _ => return Err("Unsupported audio sample rate"),
        };

        let bits = match bit_depth {
            16 => HDA_FMT_16BIT,
            24 => HDA_FMT_24BIT,
            32 => HDA_FMT_32BIT,
            _ => return Err("Unsupported audio bit depth"),
        };

        if channels == 0 || channels > 8 {
            return Err("Invalid channel count");
        }
        let chan_field = (channels - 1) as u16;

        Ok(base | bits | chan_field)
    }

    /// Configure audio output stream
    pub fn configure_stream(&mut self, stream_idx: usize, sample_rate: u32, channels: u8, bit_depth: u8) -> Result<u16, &'static str> {
        if stream_idx >= self.streams.len() {
            return Err("Invalid stream index");
        }

        let fmt = Self::encode_format(sample_rate, channels, bit_depth)?;
        self.streams[stream_idx].format_tag = fmt;
        self.streams[stream_idx].sample_rate = sample_rate;
        self.streams[stream_idx].channels = channels;
        Ok(fmt)
    }

    pub fn set_stream_running(&mut self, stream_idx: usize, running: bool) {
        if stream_idx < self.streams.len() {
            self.streams[stream_idx].is_running = running;
        }
    }
}
