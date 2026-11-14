# NovaOS State & Milestone Inventory

**Last Updated:** 2026-11-14  
**Operating Mode:** Adoption Mode (CONTRACT.md Section 0A & Section 21)  
**Target Architecture:** x86_64 UEFI 2.10 Freestanding Rust (`#![no_std]`, `#![no_main]`)  
**Baseline Evidence:** `docs/evidence/baseline.md`  

---

## 1. Milestone M0 – M10 Inventory (Pre-1.0 Base)

| Milestone | Subsystem / Scope | Gates | Status | Evidence & Test Location |
| :--- | :--- | :---: | :---: | :--- |
| **M0** | Toolchain, Cargo workspaces, multi-stage builder | N/A | **DONE** | `tools/build-image.ps1`, `Cargo.toml` |
| **M1** | UEFI Bootloader (`BOOTX64.EFI`), GDT/TSS/IDT, IST1 Stack, Exceptions | 4 / 4 | **DONE** | `kernel/src/arch/tests.rs` |
| **M2** | Memory Management, 4-Level Paging, 64MB Heap, Typed Object Handles | 9 / 9 | **DONE** | `kernel/src/memory/tests.rs`, `kernel/src/objects/tests.rs` |
| **M3** | 10 First-Party Device Drivers (UART, GOP, Console, PIT, APIC, Kbd, Mouse, PCI, Blk, Net) | 10 / 10 | **DONE** | `kernel/src/drivers/tests.rs` |
| **M4** | Preemptive Multi-Queue Scheduler (64 threads), Ring 3 Userspace, 21 Syscalls | 5 / 5 | **DONE** | `kernel/src/syscall/tests.rs`, `userland/init/src/main.rs` |
| **M5** | Unified VFS, NovaFS Superblock & Inode Journaling, ramfs, devfs | 6 / 6 | **DONE** | `kernel/src/fs/tests.rs` |
| **M6** | VirtIO-Net Networking Stack (Ethernet II, ARP, IPv4, ICMP, UDP, TCP 3-way, IPC) | 6 / 6 | **DONE** | `kernel/src/network/tests.rs` |
| **M7** | 32-bit ARGB Double-Buffered Software Compositor, DamageTracker, Window Server | 7 / 7 | **DONE** | `kernel/src/graphics/tests.rs` |
| **M8** | Windows 11 Desktop Shell, NovaBar, NovaLauncher, Snap Assist, Workspaces | 105 / 105 | **DONE** | `kernel/src/graphics/windows_features_test.rs` |
| **M9** | Core Apps (Files, SysMon, Settings, DevMgr, Calc, Notepad, Terminal & NovaShell) | 31 / 31 | **DONE** | `kernel/src/graphics/app_tests.rs`, `kernel/src/drivers/terminal_tests.rs` |
| **M10** | App Platform, Signed `.nvp` Store (Ed25519/SHA-256), GPT Partitioner, Recovery | 26 / 26 | **DONE** | `kernel/src/package/store_tests.rs`, `kernel/src/recovery/tests.rs` |

### M0–M10 Gate Check Result: **ALL DONE (203 / 203 GATES PASSING, 0 ERRORS)**
*All foundational milestones M0 through M10 are fully certified in hardware emulation with zero open gates.*

---

## 2. Post-1.0 Milestone Target Status

Following the mandatory sequence in Section 21:

| Target Milestone | Scope Description | Current Status | Notes |
| :--- | :--- | :---: | :--- |
| **M11** | **virtio-gpu display, clipboard service, drag and drop** | **TARGET** | Next active milestone to implement. |
| **M12** | userspace threads, futex syscall, dynamic linking (ELF shared libs, loader) | QUEUED | Blocked on M11 completion. |
| **M13** | musl libc port + POSIX layer (files, processes, pipes, sockets, threads, time) | QUEUED | Blocked on M12 completion. |
| **M14** | busybox-style coreutils, MicroPython, WASM runtime (wasmi) for sandboxed apps | QUEUED | Blocked on M13 completion. |
| **M15** | USB xHCI + USB HID, AHCI/SATA, NVMe (behind the existing block layer) | QUEUED | Drivers exist in kernel; requires block layer unification. |
| **M16** | SMP (AP startup, per-CPU data, per-CPU scheduler queues), demand paging, swap | QUEUED | Blocked on M15 completion. |
| **M17** | minimal browser, audio/video decoders, virtio-snd then Intel HDA | QUEUED | Intel HDA exists in kernel; virtio-snd and decoders queued. |
| **M18** | ACPI AML interpreter, battery, suspend (S3), touchpad (I2C-HID) | QUEUED | Blocked on M17 completion. |
| **M19** | disk encryption (ChaCha20-Poly1305 block layer), Secure Boot chain, IPv6 | QUEUED | Blocked on M18 completion. |
| **M20** | Wi-Fi/Bluetooth (one specific chip only, chosen with user first) | QUEUED | Final target milestone. |

---

## 3. Current Target Milestone

- **Milestone:** **M11**
- **Goal:** Implement `virtio-gpu` hardware accelerated display device, global system clipboard service with cross-process IPC sharing, and desktop drag-and-drop mechanics.
- **Pre-conditions:** M0–M10 verified DONE in `docs/evidence/baseline.md`.
