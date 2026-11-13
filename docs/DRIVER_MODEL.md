# NovaOS Driver Model Specification

NovaOS enforces narrow, decoupled driver interfaces:

## 1. Character & Console Output
* **UART 16550 Serial Driver (`kernel/src/drivers/serial.rs`):** Interacts with port `0x3F8` at 115200 baud, 8N1. Outputs directly to QEMU stdio.
* **VGA Text Driver (`kernel/src/drivers/vga.rs`):** Memory-mapped 80x25 characters with automatic scrolling.

## 2. Graphical Display
* **UEFI GOP Driver (`kernel/src/drivers/framebuffer.rs`):** Writes 32-bit linear RGB pixels to memory mapped buffer.

## 3. Input Subsystem
* **PS/2 Keyboard Driver (`kernel/src/drivers/ps2_keyboard.rs`):** Serviced via IRQ1 (Vector 33) at port `0x60`, translating scancodes through `pc-keyboard`.

## 4. Bus Enumeration
* **PCI Scanner (`kernel/src/drivers/pci.rs`):** Scans PCI buses 0..8 across all 32 slots and functions via configuration ports `0xCF8`/`0xCFC`.
