# NovaOS Apex — Known Limitations

*Maintained continuously from Milestone 0 per Section 2 of the Build Contract.*

This document records genuine, verified technical boundaries of NovaOS Apex at each stage of development. In accordance with the non-negotiable project rules, features are never claimed without working code and verifiable test evidence.

---

## Current Status: Milestone 3 (Interrupts, Devices & Drivers)

### 1. CPU & Architecture
- **Multicore (SMP):** Single-core execution only (Bootstrap Processor / BSP). Application Processors (APs) are not yet initialized via APIC INIT-SIPI sequences (scheduled for Milestone 11).
- **Microcode & Thermal:** CPU thermal throttling and dynamic frequency scaling (P-states/C-states) are unmanaged.
- **Hardware Interrupts Active:** 8259 PIC remapped (IRQs 0..15 -> Vectors 32..47) and hardware interrupts unmasked for PIT Timer (IRQ 0), PS/2 Keyboard (IRQ 1), and PS/2 Mouse (IRQ 12).

### 2. Firmware & Boot
- **UEFI Only:** Legacy BIOS / Master Boot Record (CSM) booting is explicitly unsupported. Requires 64-bit UEFI firmware (v2.0+).
- **Boot Handoff:** UEFI ExitBootServices transfers execution cleanly with active GOP framebuffer, ACPI 2.0/1.0 RSDP, and UEFI memory map descriptors passed via `BootInfo`.

### 3. Memory & Paging
- **4-Level Paging Active:** x86_64 PML4, PDPT, PD, and PT are operational with 1 GiB lower identity mapping and 1 GiB higher-half direct physical mapping at `0xFFFF_8000_0000_0000`.
- **Physical Frame Allocator:** Bitmap-backed allocator manages up to 4 GiB physical RAM at 4 KiB frame granularity, parsing UEFI memory descriptors and enforcing double-free rejection.
- **Kernel Dynamic Heap:** `GlobalAlloc` slab/block allocator with 2 MiB initial buffer, canary guard checks (`0xCAFEBABE...`), and double-free detection.
- **Hardware Stack Guard Pages:** Kernel stacks feature unmapped bottom guard pages (`PAGE_PRESENT` clear) for hardware stack overflow protection.

### 4. Peripherals & Drivers (Section 10 Order)
- **1. 16550 UART Serial:** Fully operational on COM1 (`0x3F8`) at 38,400 baud, 8N1.
- **2. GOP Framebuffer:** Linear 32-bit rasterizer operational on UEFI-discovered base address.
- **3. Text Console:** 8x16 monospace font terminal with line wrap, scrolling, and ANSI colors.
- **4. Timer / APIC:** 8254 PIT running at 1000 Hz (1 ms tick) with uptime and sleep primitives; CPUID Local APIC detected.
- **5. PS/2 Keyboard:** Scancode Set 1/2 decoder operational with shift/ctrl/alt modifier tracking.
- **6. PS/2 Mouse:** 3-byte streaming packet decoder active with position clamped to screen bounds.
- **7. PCI Enumeration:** Type 0/1 header scanning across 8 buses with BAR discovery.
- **8. VirtIO Block:** 512-byte sector read/write supported with in-memory fallback block store.
- **9. VirtIO Network:** Packet TX/RX and MAC address extraction active with virtual loopback fallback.
- **10. Basic Audio:** PC Speaker tone synthesizer active via PIT Channel 2 (`0x61`).

### 5. Future Hardware Roadmap
- USB 3.0 xHCI / USB HID (Milestone 11)
- NVMe 1.4 & AHCI SATA 1.3 (Milestone 11)
- Intel HD Audio / AC97 Codecs (Milestone 11)
- Wi-Fi & Bluetooth (Future)
- GPU Acceleration (Future)

### 6. Process & Userland
- Preemptive scheduling, Ring 3 isolation, ELF64 user processes, and system call gates (`int 0x80`) are scheduled for **Milestone 4**.
