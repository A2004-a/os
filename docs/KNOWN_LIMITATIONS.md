# NovaOS Apex — Known Limitations

*Maintained continuously from Milestone 0 per Section 2 of the Build Contract.*

This document records genuine, verified technical boundaries of NovaOS Apex at each stage of development. In accordance with the non-negotiable project rules, features are never claimed without working code and verifiable test evidence.

---

## Current Status: Milestone 2 (Memory & Kernel Object Model)

### 1. CPU & Architecture
- **Multicore (SMP):** Single-core execution only (Bootstrap Processor / BSP). Application Processors (APs) are not yet initialized via APIC INIT-SIPI sequences.
- **Microcode & Thermal:** CPU thermal throttling and dynamic frequency scaling (P-states/C-states) are unmanaged.
- **Hardware Interrupts:** 8259 PIC and local APIC timers are unmasked/uninitialized; hardware interrupts are held disabled until Milestone 3 timer and device drivers are introduced.

### 2. Firmware & Boot
- **UEFI Only:** Legacy BIOS / Master Boot Record (CSM) booting is explicitly unsupported. Requires 64-bit UEFI firmware (v2.0+).
- **Secure Boot:** Secure Boot key validation is currently in development. Test firmware must run with Secure Boot disabled or self-signed test keys.
- **Boot Handoff:** UEFI ExitBootServices transfers execution cleanly with active GOP framebuffer, ACPI 2.0/1.0 RSDP, and UEFI memory map descriptors passed via `BootInfo`.

### 3. Memory & Paging
- **4-Level Paging Active:** x86_64 PML4, PDPT, PD, and PT are operational with 1 GiB lower identity mapping and 1 GiB higher-half direct physical mapping at `0xFFFF_8000_0000_0000`.
- **Physical Frame Allocator:** Bitmap-backed allocator manages up to 4 GiB physical RAM at 4 KiB frame granularity, parsing UEFI memory descriptors and enforcing double-free rejection.
- **Kernel Dynamic Heap:** `GlobalAlloc` slab/block allocator with 2 MiB initial buffer, canary guard checks (`0xCAFEBABE...`), and double-free detection.
- **Hardware Stack Guard Pages:** Kernel stacks feature unmapped bottom guard pages (`PAGE_PRESENT` clear) for hardware stack overflow protection.
- **Swapping:** Virtual memory paging to disk (swap/pagefile) is not implemented.

### 4. Kernel Objects & Capabilities
- **Typed Handle Model:** Typed handle table (256 entries) enforces strict capability rights masks (READ, WRITE, EXECUTE, TRANSFER, DUPLICATE, CLOSE, MAP, SIGNAL, WAIT), rejects type confusion, and manages reference-counted lifecycles across 8 resource categories: Process, Thread, File, Socket, Event, Device, Window, and SharedMemory.

### 5. Storage & Filesystem
- **Read-Only Boot:** Early boot volume is read-only FAT32 via UEFI boot services. Persistent NovaFS block storage is scheduled for Milestone 5.

### 6. Graphics & Compositing
- **Software Rendering:** GPU hardware acceleration (Intel/AMD/NVIDIA 3D pipelines) is not active; all rendering uses the CPU software rasterizer against the UEFI GOP linear framebuffer discovered at boot.

### 7. Networking
- **Wired Only:** Wi-Fi and Bluetooth wireless stacks are non-goals for early milestones. Networking targets VirtIO-Net and Intel e1000 PCI devices in Milestone 6.

### 8. Applications & Binary Compatibility
- **No POSIX / Windows ABI:** NovaOS Apex does not run unmodified Linux ELF or Windows PE binaries. All applications must target the native NovaOS Apex system call ABI and runtime (Milestone 4).
