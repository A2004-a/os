# NovaOS Apex v1.0.0 Release Notes

**Version:** 1.0.0 "Apex Final"  
**Release Date:** November 2026  
**Architecture:** x86_64 (64-bit Long Mode)  
**Firmware:** UEFI 2.10  
**Kernel:** Freestanding Modular Monolithic Rust Kernel (`#![no_std]`, `#![no_main]`)  
**Certification:** Windows 11-Class Desktop Operating System Benchmark  

---

## 1. Product Overview

**NovaOS Apex** is an original, bare-metal desktop operating system designed to equal the practical strengths and polish expected from a modern Windows 11-class PC platform while maintaining an independent, original identity and architecture.

NovaOS Apex boots directly from UEFI firmware, initializes its own kernel, manages memory, devices, processes, files, networking, and graphics, and provides an integrated desktop shell, native applications, package management, installer, and recovery engines—with **zero reliance on Electron, Chromium, Linux, Windows, macOS, or any host OS**.

---

## 2. Architecture Milestones Summary (M0 – M12)

Every milestone defined in the Antigravity Build Contract has been realized and verified through automated test suites in headless QEMU:

| Milestone | Subsystem | Key Capabilities Delivered | Status |
|---|---|---|---|
| **M0** | **Toolchain & Boot Proof** | Custom `x86_64-unknown-none` target, Cargo workspaces, multi-stage image builder, headless QEMU/OVMF test runner | **Verified** |
| **M1** | **UEFI Boot Manager & Entry** | UEFI PE/COFF loader (`BOOTX64.EFI`), memory map parser, GOP framebuffer, ACPI RSDP discovery, 64-bit GDT/TSS/IDT, dedicated IST1 double-fault stack | **Verified** |
| **M2** | **Memory & Object Model** | Physical frame allocator (4 GiB address tracking), 4-level paging mapper/unmapper, canary-protected heap allocator (32 MiB buffer), typed-handle object model | **Verified** |
| **M3** | **Interrupts & Device Drivers** | UART 16550 serial, GOP linear framebuffer (1280x800), text console (160x50), 8254 PIT / Local APIC (1000 Hz), PS/2 keyboard & mouse, PCI bus enumerator, VirtIO block & net, PC speaker audio | **Verified** |
| **M4** | **Scheduling, Ring 3 & Syscalls** | Preemptive multi-priority scheduler (64 threads), user address space isolation, ELF64 dynamic loading, 21 system calls (`sys_read`, `sys_write`, `sys_open`, `sys_spawn`, `sys_ipc`, etc.) | **Verified** |
| **M5** | **Files, Storage & Persistence** | Unified VFS, ramfs, devfs (`/dev/null`, `/dev/zero`, `/dev/console`), NovaFS transactional filesystem with superblock integrity and inode journaling, multi-user profiles (`/users/amiyo`) | **Verified** |
| **M6** | **Networking Stack & IPC** | Ethernet II frame parsing, ARP table resolution, IPv4 routing & checksumming, ICMP echo reply, UDP & TCP socket abstractions, DNS resolver, high-speed typed IPC channels | **Verified** |
| **M7** | **Display, Compositor & Window Server** | Double-buffered 32-bit ARGB software compositor, dirty rectangle partial repainting, Window Server with z-order stack, titlebar decoration, drag-to-move, click-to-focus | **Verified** |
| **M8** | **Desktop Shell & Productivity** | Nova Bar (start button, running tasks, system tray, clock), Nova Launcher with live app search, Snap Assist (Half-Left, Half-Right, Fullscreen, 2x2 Grid), 4 Virtual Workspaces | **Verified** |
| **M9** | **Core Desktop Applications** | **Nova Files** (breadcrumbs, previews, permissions), **Nova System Monitor** (CPU/RAM telemetry, process manager), **Nova Settings** (searchable reversible config), **Nova Package Manager** (catalog, signatures), **Nova Terminal** (command execution, scrolling) | **Verified** |
| **M10** | **App Platform, Installer & Recovery** | Signed `.nvp` packages with SHA-256 digests and capability sandboxes, GPT disk partitioner with IEEE 802.3 CRC32, automated OS installer, 3-strike boot failure recovery trigger and rollback scrubber | **Verified** |
| **M11** | **Hardware Expansion Subsystem** | PCI Bus Mastering discovery, **AHCI SATA 1.3** controller with 48-bit LBA DMA (`/dev/sda`), **NVMe 1.4** SSD controller with Admin/IO queues (`/dev/nvme0n1`), **USB 3.0 xHCI** with USB HID keyboard/mouse packet decoders, **Intel High Definition Audio (HDA 1.0a)** with 16-bit PCM tone synthesizer | **Verified** |
| **M12** | **Hardening & Release Engineering** | Adversarial syscall boundary fuzzer (NULL ptrs, kernel tampering, overflows), network stack runt packet fuzzer, heap canary leak walk (100% LIFO reclamation), process lifecycle stress test, production conformance certification | **Verified** |

---

## 3. Verified Hardware Support

NovaOS Apex features native, first-party drivers built into the kernel core:

- **Storage:**
  - VirtIO Block Device (`/dev/vda`)
  - AHCI SATA 1.3 Controller (`/dev/sda`) with 48-bit LBA DMA
  - NVMe 1.4 Solid-State Drive (`/dev/nvme0n1`) with dedicated Submission/Completion Queues
- **Input & Peripherals:**
  - USB 3.0 xHCI Host Controller
  - USB HID Keyboard (Standard 8-byte boot protocol)
  - USB HID Mouse (3-byte streaming relative motion)
  - Legacy PS/2 Keyboard (Scancode Set 1 & modifiers)
  - Legacy PS/2 Mouse (3-byte streaming packet mode)
- **Multimedia & Display:**
  - UEFI GOP Linear Framebuffer (1280x800, 32-bit ARGB)
  - Intel High Definition Audio (HDA 1.0a) with 48 kHz 16-bit stereo PCM streaming
  - PC Speaker Square-Wave Synthesizer
- **Networking:**
  - VirtIO Network Adapter (MAC 52:54:00:12:34:56) with loopback and packet filtering

---

## 4. Release Distribution Artifacts

The production release artifacts are packaged in `dist/novaos-apex-v1.0.0/`:
- `EFI/BOOT/BOOTX64.EFI`: 64-bit UEFI Boot Manager
- `novaos-apex.elf`: Modular Monolithic x86_64 Kernel
- `init`: Ring 3 System Init Daemon (PID 1)
- `shell`: Ring 3 Graphical Desktop Shell
- `apps`: Ring 3 Core Application Suite
- `novaos.cfg`: Boot Manager Configuration
- `novaos-apex-v1.0.0-SHA256SUMS.txt`: Cryptographic SHA-256 integrity checksums
- `novaos-apex-v1.0.0-MANIFEST.json`: Formal release certification manifest

---

## 5. Verification Commands

To verify and boot NovaOS Apex in QEMU:
```powershell
# Build image
powershell -ExecutionPolicy Bypass -File tools/build-image.ps1

# Run automated test suite
python tools/test_runner.py

# Launch interactive graphical desktop in QEMU
powershell -ExecutionPolicy Bypass -File tools/run-qemu.ps1
```
