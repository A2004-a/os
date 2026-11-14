# NovaOS Apex Changelog

All notable changes to NovaOS Apex are documented in this file.

## [0.3.0] - 2026-11-14 — Milestone 3 (Interrupts, Devices, and Drivers)

### Added
- **First-Party Driver Infrastructure:**
  - Implemented common `Driver` lifecycle trait with strict `probe`, `init`, `start`, and `stop` states.
  - Implemented all 10 device drivers in the strict order required by Section 10 of the Build Contract.
- **Peripherals & Driver Implementations:**
  1. **16550 UART Serial (`drivers/serial.rs`):** First-party COM1 `0x3F8` driver with scratch register probing, FIFO queueing, and 8N1 transmission.
  2. **GOP Linear Framebuffer (`drivers/framebuffer.rs`):** 32-bit pixel rasterizer with `draw_pixel`, `draw_rect`, `clear`, and `blit` primitives.
  3. **Text Console (`drivers/console.rs`):** Monospace 8x16 bitmap terminal font, hardware-scrolled display buffer, ANSI colors, and cursor management.
  4. **Timer & APIC (`drivers/timer.rs` & `drivers/pic.rs`):**
     - Dual 8259 PIC remapped (IRQs 0..15 to IDT vectors 32..47) with cascading and EOI dispatch.
     - 8254 PIT running at 1000 Hz (1 ms tick) with uptime and sleep primitives.
     - CPUID Local APIC probe with IA32_APIC_BASE MSR address discovery.
  5. **PS/2 Keyboard (`drivers/keyboard.rs`):** IRQ 1 handler, Scancode Set 1/2 decoder, modifier tracking (Shift, Ctrl, Alt, CapsLock), and circular event buffer.
  6. **PS/2 Mouse (`drivers/mouse.rs`):** IRQ 12 handler, 3-byte streaming packet decoder, button states (Left, Right, Middle), and screen boundary clamped coordinates.
  7. **PCI Bus Enumerator (`drivers/pci.rs`):** Configuration space scanner (ports `0xCF8`/`0xCFC`) scanning 8 bus topologies, reading Vendor/Device IDs, BARs, and class codes.
  8. **VirtIO Block Storage (`drivers/virtio_block.rs`):** VirtIO PCI storage controller with VirtQueue negotiation and 512-byte sector read/write transactions.
  9. **VirtIO Network Adapter (`drivers/virtio_net.rs`):** Ethernet II frame transmission and reception, MAC address extraction, and loopback verification.
  10. **Basic Audio Abstraction (`drivers/audio.rs`):** PC Speaker square-wave sound synthesizer driven by PIT Channel 2 (`0x61`).
- **Interrupt Routing & IDT Expansion:**
  - Extended 256-gate IDT with ISR trampolines for hardware IRQ vectors 32..47.
  - Exception dispatcher routing IRQ 0 to PIT timer, IRQ 1 to PS/2 keyboard, IRQ 12 to PS/2 mouse, with automated EOI signaling.
  - Global hardware interrupts enabled via `sti()`.
- **Automated Verification Suite:**
  - `drivers/tests.rs` verifying all 10 device drivers sequentially.

## [0.2.0] - 2026-11-14 — Milestone 2 (Memory and Kernel Object Model)

### Added
- Bitmap physical frame allocator managing 4 GiB of RAM at 4 KiB page granularity.
- Four-level paging (PML4, PDPT, PD, PT) with higher-half mapping at `0xFFFF_8000_0000_0000`.
- Kernel dynamic heap allocator (`GlobalAlloc`) with 2 MiB buffer, canary guards, and tracking.
- Kernel stacks provisioned with hardware unmapped guard pages.
- Typed handle table (256 slots) managing 8 kernel object categories with capability rights masks.

## [0.1.0] - 2026-11-14 — Milestone 1 (UEFI Boot Manager & Kernel Entry)

### Added
- UEFI SimpleFileSystem reader loading `/novaos.cfg` and `/novaos-apex.elf` from FAT volumes.
- Freestanding ELF64 loader and validator.
- GOP linear framebuffer discovery and ACPI 2.0/1.0 RSDP locator.
- 64-bit GDT/TSS with dedicated 32 KiB emergency double-fault stack on IST1.
- 256-gate IDT with trampolines for all 32 x86_64 architecture exceptions.
- Deliberate exception verification suite testing `#DE`, `#UD`, `#PF`, and `#DF`.

## [0.0.1] - 2026-11-14 — Milestone 0 (Toolchain & Boot Proof)

### Added
- Root Cargo workspace targeting freestanding Rust toolchain on `x86_64-unknown-none` and `x86_64-unknown-uefi`.
- Custom linker script, cross-platform build scripts, Makefile, and CI smoke test.
