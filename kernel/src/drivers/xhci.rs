//! NovaOS Zero — USB 3.0 Extensible Host Controller Interface (xHCI) & USB HID Driver
//! Class 0Ch (Serial Bus), Subclass 03h (USB Controller), ProgIF 30h (xHCI)

pub const PCI_CLASS_SERIAL: u8    = 0x0C;
pub const PCI_SUBCLASS_USB: u8    = 0x03;
pub const PCI_PROGIF_XHCI: u8     = 0x30;

pub const TRB_TYPE_NORMAL: u32       = 1;
pub const TRB_TYPE_SETUP_STAGE: u32  = 2;
pub const TRB_TYPE_DATA_STAGE: u32   = 3;
pub const TRB_TYPE_STATUS_STAGE: u32 = 4;
pub const TRB_TYPE_LINK: u32         = 6;

pub const XHCI_RING_CAPACITY: usize = 32;

/// Standard 16-byte xHCI Transfer Request Block (TRB)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferRequestBlock {
    pub param_low: u32,
    pub param_high: u32,
    pub status: u32,
    pub control: u32, // Bit 0: Cycle bit; Bits 15..10: TRB Type
}

impl TransferRequestBlock {
    pub fn empty() -> Self {
        TransferRequestBlock {
            param_low: 0,
            param_high: 0,
            status: 0,
            control: 0,
        }
    }

    pub fn new(param: u64, status: u32, trb_type: u32, cycle: bool) -> Self {
        let control = ((trb_type & 0x3F) << 10) | if cycle { 1 } else { 0 };
        TransferRequestBlock {
            param_low: (param & 0xFFFF_FFFF) as u32,
            param_high: ((param >> 32) & 0xFFFF_FFFF) as u32,
            status,
            control,
        }
    }

    pub fn get_trb_type(&self) -> u32 {
        (self.control >> 10) & 0x3F
    }

    pub fn get_cycle_bit(&self) -> bool {
        (self.control & 1) != 0
    }

    pub fn get_param_low(&self) -> u32 { self.param_low }
    pub fn get_param_high(&self) -> u32 { self.param_high }
}

/// Standard USB HID Boot Keyboard Report (8 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbHidKeyboardReport {
    pub modifiers: u8, // Ctrl, Shift, Alt, GUI
    pub reserved: u8,
    pub keycodes: [u8; 6],
}

impl UsbHidKeyboardReport {
    pub fn new(modifiers: u8, keycodes: [u8; 6]) -> Self {
        UsbHidKeyboardReport {
            modifiers,
            reserved: 0,
            keycodes,
        }
    }

    pub fn is_shift_pressed(&self) -> bool {
        (self.modifiers & 0x22) != 0 // Left Shift (0x02) or Right Shift (0x20)
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        (self.modifiers & 0x11) != 0 // Left Ctrl (0x01) or Right Ctrl (0x10)
    }
}

/// Standard USB HID Boot Mouse Report (3 or 4 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbHidMouseReport {
    pub buttons: u8, // Bit 0: Left, Bit 1: Right, Bit 2: Middle
    pub delta_x: i8,
    pub delta_y: i8,
    pub wheel: i8,
}

impl UsbHidMouseReport {
    pub fn new(buttons: u8, delta_x: i8, delta_y: i8, wheel: i8) -> Self {
        UsbHidMouseReport {
            buttons,
            delta_x,
            delta_y,
            wheel,
        }
    }
}

pub struct XhciController {
    pub mmio_base: u64,
    pub max_slots: u8,
    pub max_ports: u8,
    pub cycle_state: bool,
    pub ring_index: usize,
    pub cmd_ring: [TransferRequestBlock; XHCI_RING_CAPACITY],
}

impl XhciController {
    pub fn new(mmio_base: u64, max_slots: u8, max_ports: u8) -> Self {
        XhciController {
            mmio_base,
            max_slots,
            max_ports,
            cycle_state: true,
            ring_index: 0,
            cmd_ring: [TransferRequestBlock::empty(); XHCI_RING_CAPACITY],
        }
    }

    /// Enqueue a TRB onto the Command Ring
    pub fn enqueue_trb(&mut self, mut trb: TransferRequestBlock) -> usize {
        let idx = self.ring_index;
        // Set current producer cycle bit
        trb.control = (trb.control & !1) | if self.cycle_state { 1 } else { 0 };
        self.cmd_ring[idx] = trb;

        self.ring_index += 1;
        if self.ring_index >= XHCI_RING_CAPACITY {
            self.ring_index = 0;
            self.cycle_state = !self.cycle_state;
        }
        idx
    }

    /// Decode USB HID Keyboard usage ID into ASCII character
    pub fn decode_hid_scancode(keycode: u8, shift: bool) -> Option<char> {
        match keycode {
            0x04..=0x1D => {
                // 'a' (0x04) through 'z' (0x1D)
                let base = if shift { b'A' } else { b'a' };
                Some((base + (keycode - 0x04)) as char)
            }
            0x1E..=0x26 => {
                // '1'..'9'
                if shift {
                    let symbols = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
                    Some(symbols[(keycode - 0x1E + 1) as usize])
                } else {
                    Some((b'1' + (keycode - 0x1E)) as char)
                }
            }
            0x27 => Some(if shift { ')' } else { '0' }),
            0x28 => Some('\n'), // Enter
            0x2A => Some('\x08'), // Backspace
            0x2C => Some(' '),  // Spacebar
            _ => None,
        }
    }
}
