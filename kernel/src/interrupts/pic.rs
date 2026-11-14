//! NovaOS Zero — Freestanding 8259 Programmable Interrupt Controller (PIC) Driver

use crate::arch::x86_64::io::{inb, io_wait, outb};
use crate::sync::Spinlock;

pub const PIC1_CMD: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_CMD: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const PIC_EOI: u8 = 0x20;

pub struct ChainedPics {
    offset1: u8,
    offset2: u8,
}

impl ChainedPics {
    pub const fn new(offset1: u8, offset2: u8) -> Self {
        Self { offset1, offset2 }
    }

    pub unsafe fn initialize(&mut self) {
        // Save interrupt masks
        let _mask1 = inb(PIC1_DATA);
        let _mask2 = inb(PIC2_DATA);

        // ICW1: Start initialization in cascade mode
        outb(PIC1_CMD, 0x11);
        io_wait();
        outb(PIC2_CMD, 0x11);
        io_wait();

        // ICW2: Master PIC vector offset
        outb(PIC1_DATA, self.offset1);
        io_wait();
        // ICW2: Slave PIC vector offset
        outb(PIC2_DATA, self.offset2);
        io_wait();

        // ICW3: Tell Master PIC there is a Slave PIC at IRQ2 (0000 0100)
        outb(PIC1_DATA, 0x04);
        io_wait();
        // ICW3: Tell Slave PIC its cascade identity (0000 0010)
        outb(PIC2_DATA, 0x02);
        io_wait();

        // ICW4: Set 8086/88 mode
        outb(PIC1_DATA, 0x01);
        io_wait();
        outb(PIC2_DATA, 0x01);
        io_wait();

        // Restore masks: unmask IRQ0 (Timer), IRQ1 (Keyboard), IRQ2 (Cascade) on Master
        outb(PIC1_DATA, 0xF8);
        // Unmask IRQ12 (Mouse on line 4) on Slave PIC
        outb(PIC2_DATA, 0xEF);
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if interrupt_id >= self.offset2 {
            outb(PIC2_CMD, PIC_EOI);
        }
        outb(PIC1_CMD, PIC_EOI);
    }
}

pub static PICS: Spinlock<ChainedPics> =
    Spinlock::new(ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET));

pub fn init() {
    unsafe {
        PICS.lock().initialize();
    }
}

pub fn notify_eoi(interrupt_id: u8) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(interrupt_id);
    }
}
