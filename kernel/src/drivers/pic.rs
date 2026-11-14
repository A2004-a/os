//! NovaOS Apex — Dual 8259 Programmable Interrupt Controller (PIC) Driver
//! Remaps hardware IRQs 0..15 to IDT vectors 32..47 and manages IRQ masking & EOI.

use core::arch::asm;

pub const PIC1_COMMAND: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_COMMAND: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

pub const PIC_EOI: u8 = 0x20;

pub const PIC1_OFFSET: u8 = 32; // Vectors 32..39
pub const PIC2_OFFSET: u8 = 40; // Vectors 40..47

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

#[inline(always)]
unsafe fn io_wait() {
    // Port 0x80 is an unused diagnostic port frequently used for small I/O delay
    outb(0x80, 0);
}

pub fn init() {
    unsafe {
        // Save masks
        let _mask1 = inb(PIC1_DATA);
        let _mask2 = inb(PIC2_DATA);

        // ICW1: Start initialization in cascade mode
        outb(PIC1_COMMAND, 0x11);
        io_wait();
        outb(PIC2_COMMAND, 0x11);
        io_wait();

        // ICW2: Vector offsets
        outb(PIC1_DATA, PIC1_OFFSET);
        io_wait();
        outb(PIC2_DATA, PIC2_OFFSET);
        io_wait();

        // ICW3: Cascade wiring
        outb(PIC1_DATA, 0x04); // Tell Master PIC there is Slave at IRQ2 (0000 0100)
        io_wait();
        outb(PIC2_DATA, 0x02); // Tell Slave PIC its cascade identity (0000 0010)
        io_wait();

        // ICW4: 8086/88 mode
        outb(PIC1_DATA, 0x01);
        io_wait();
        outb(PIC2_DATA, 0x01);
        io_wait();

        // Unmask IRQ0 (Timer), IRQ1 (Keyboard), IRQ2 (Cascade to Slave), IRQ12 (PS/2 Mouse)
        // Master: IRQ0=0, IRQ1=0, IRQ2=0, others=1 -> 0b1111_1000 = 0xF8
        // Slave: IRQ12 (bit 4)=0, others=1 -> 0b1110_1111 = 0xEF
        outb(PIC1_DATA, 0xF8);
        outb(PIC2_DATA, 0xEF);
    }
}

pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

pub fn mask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let bit = if irq < 8 { irq } else { irq - 8 };
    unsafe {
        let mask = inb(port) | (1 << bit);
        outb(port, mask);
    }
}

pub fn unmask_irq(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let bit = if irq < 8 { irq } else { irq - 8 };
    unsafe {
        let mask = inb(port) & !(1 << bit);
        outb(port, mask);
    }
}
