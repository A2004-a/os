# NovaOS Apex — Known Limitations

*Maintained continuously from Milestone 0 per Section 2 of the Build Contract.*

This document records genuine, verified technical boundaries of NovaOS Apex at each stage of development. In accordance with the non-negotiable project rules, features are never claimed without working code and verifiable test evidence.

---

## Current Status: Milestone 0 (Toolchain & Boot Proof)

### 1. CPU & Architecture
- **Multicore (SMP):** Single-core execution only (Bootstrap Processor / BSP). Application Processors (APs) are not yet initialized via APIC INIT-SIPI sequences.
- **Microcode & Thermal:** CPU thermal throttling and dynamic frequency scaling (P-states/C-states) are unmanaged.

### 2. Firmware & Boot
- **UEFI Only:** Legacy BIOS / Master Boot Record (CSM) booting is explicitly unsupported. Requires 64-bit UEFI firmware (v2.0+).
- **Secure Boot:** Secure Boot key validation is currently in development (Milestone 1/6 roadmap). Test firmware must run with Secure Boot disabled or self-signed test keys.

### 3. Memory & Paging
- **Milestone 0 Memory:** Kernel relies on identity/early physical mapping before Milestone 2 paging initialization.
- **Swapping:** Virtual memory paging to disk (swap/pagefile) is not implemented.

### 4. Storage & Filesystem
- **Read-Only Boot:** Early boot volume is read-only FAT32 via UEFI boot services. Persistent NovaFS block storage is scheduled for Milestone 5.

### 5. Graphics & Compositing
- **Software Rendering:** GPU hardware acceleration (Intel/AMD/NVIDIA 3D pipelines) is not active; all rendering uses the CPU software rasterizer against the UEFI GOP linear framebuffer.

### 6. Networking
- **Wired Only:** Wi-Fi and Bluetooth wireless stacks are non-goals for early milestones. Networking targets VirtIO-Net and Intel e1000 PCI devices.

### 7. Applications & Binary Compatibility
- **No POSIX / Windows ABI:** NovaOS Apex does not run unmodified Linux ELF or Windows PE binaries. All applications must target the native NovaOS Apex system call ABI and runtime.
