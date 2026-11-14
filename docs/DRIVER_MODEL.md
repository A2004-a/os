# NovaOS Apex — Driver Model & Hardware Architecture

## Driver Lifecycle Trait

All hardware drivers implement a common lifecycle contract:

```rust
pub trait Driver {
    fn probe(&mut self) -> Result<(), &'static str>;
    fn initialize(&mut self) -> Result<(), &'static str>;
    fn start(&mut self) -> Result<(), &'static str>;
    fn stop(&mut self) -> Result<(), &'static str>;
    fn is_online(&self) -> bool;
}
```

## First-Party Drivers (Milestones 0–3)

1. **UART 16550 Serial Driver (`drivers/serial.rs`):** Primary debug and telemetry logging on COM1 port `0x3F8` at 38,400 baud.
2. **GOP Framebuffer Driver (`drivers/framebuffer.rs`):** Linear RGBA/BGRA framebuffer blitting using double-buffered software surfaces.
3. **PIT 8254 & APIC Timer (`interrupts/timer.rs`):** Monotonic system tick at 100 Hz driving quantum preemptive thread context switches.
4. **PS/2 Keyboard & Mouse (`drivers/ps2_keyboard.rs`, `drivers/ps2_mouse.rs`):** IRQ 1 and IRQ 12 event stream decoders with 3-button mouse reporting.
5. **PCI Bus Enumerator (`drivers/pci.rs`):** Port I/O scanner probing buses 0..8 for vendor, device, and class IDs.
6. **VirtIO Block & VirtIO Net (`drivers/virtio_blk.rs`, `drivers/virtio_net.rs`):** Paravirtualized storage and Ethernet packet transceivers.

## Extended Hardware Drivers (Milestone 11)

- **AHCI 1.3 SATA:** Port command list processing and DMA FIS packets.
- **NVM Express (NVMe 1.4):** 64-byte Submission Queue and Doorbell MMIO stride control.
- **USB 3.0 xHCI & HID:** TRB ring management with USB keyboard/mouse boot protocol decoding.
- **Intel High Definition Audio:** Azalia PCM DMA stream pipeline with 48 kHz 16-bit stereo playback.
