# NovaOS Apex

<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.0--certified-blue.svg?style=for-the-badge&logo=rust" alt="Version 1.0.0" />
  <img src="https://img.shields.io/badge/platform-x86__64-orange.svg?style=for-the-badge&logo=amd" alt="Platform x86_64" />
  <img src="https://img.shields.io/badge/firmware-UEFI%202.10-purple.svg?style=for-the-badge" alt="Firmware UEFI" />
  <img src="https://img.shields.io/badge/build-passing-brightgreen.svg?style=for-the-badge&logo=githubactions" alt="Build Status" />
  <img src="https://img.shields.io/badge/license-MIT-green.svg?style=for-the-badge" alt="License MIT" />
</p>

<p align="center">
  <strong>NovaOS Apex</strong> is an original, bare-metal desktop operating system engineered in freestanding Rust (<code>#![no_std]</code>, <code>#![no_main]</code>) for 64-bit x86_64 PC hardware booted via standard UEFI firmware.
</p>

<p align="center">
  <em>Equaling the practical strengths, polish, and usability of a modern Windows 11-class PC platform—while using an original NovaOS identity, modular monolithic kernel, and zero host OS dependencies.</em>
</p>

<p align="center">
  <img src="docs/assets/novaos_desktop.png" alt="NovaOS Apex Live Desktop Screenshot" width="800" style="border-radius: 8px; box-shadow: 0 8px 24px rgba(0,0,0,0.5);" />
</p>

---

## 🌟 Highlights & Philosophy

- **Bare-Metal Autonomy:** Boots directly from 64-bit UEFI firmware into an original Rust kernel. Zero reliance on Linux, Windows, macOS, Electron, or Chromium post-boot.
- **Production Desktop Quality:** Features a double-buffered 32-bit ARGB software compositor, modern window server with dirty rectangle repainting, Nova Bar, Nova Launcher with live app search, 7-zone Snap Assist tiling, and 4 Virtual Workspaces.
- **Native Applications Suite:** Built-in **Nova Files** (VFS explorer with previews), **Nova System Monitor** (real-time CPU/RAM telemetry & task manager), **Nova Settings** (searchable reversible configuration), **Nova Package Manager** (signed `.nvp` packages), and **Nova Terminal** (interactive graphical shell).
- **First-Party Driver Stack:** 14 built-in native drivers including AHCI SATA 1.3, NVMe 1.4 SSD, USB 3.0 xHCI host controller, USB HID keyboard & mouse, Intel High Definition Audio (HDA 1.0a), UEFI GOP linear framebuffer, and VirtIO block/net.
- **Enterprise-Grade Resilience:** Transactional NovaFS block storage, GUID Partition Table (GPT) partitioner with IEEE 802.3 CRC32, automated OS installer, 3-strike boot failure recovery trigger, snapshot rollbacks, and adversarial security fuzzing.

---

## 🏗️ System Architecture

```text
+--------------------------------------------------------------------------+
|                     Nova Desktop Platform (Ring 3)                       |
|   Nova Launcher   |   Nova Bar Taskbar   |   Snap Assist (7 Zones)       |
|   Nova Files      |   System Monitor     |   Settings & Control Center   |
|   Package Manager |   Nova Terminal      |   4 Virtual Workspaces        |
+--------------------------------------------------------------------------+
|                           NovaUI Toolkit                                 |
|   Event Loop  |  Bitmap Font Rasterizer  |  Widgets (Buttons, Panels)   |
+--------------------------------------------------------------------------+
|                        System Daemons & Services                         |
|   PID 1 Init Daemon  |  Session Manager  |  Recovery & Rollback Scrubber |
+--------------------------------------------------------------------------+
======================= Ring 3 Userspace / Ring 0 Kernel ===================
+--------------------------------------------------------------------------+
|                         NovaOS Apex Kernel Core                          |
|   Syscall Dispatcher (int 0x80, 51 Syscalls) | Capability Access Control |
|   Preemptive Multi-Priority Scheduler (64 Thr)| 4-Level Paging Virtual Mem|
|   Unified VFS (RAMFS, DevFS, NovaFS)         | Network Stack (IPv4/TCP)  |
|   Double-Buffered ARGB Compositor & Blitter  | Typed IPC Mailbox Queues  |
+--------------------------------------------------------------------------+
|                            Device Drivers                                |
|   UEFI GOP Framebuffer  | 16550 UART Serial | 8254 PIT / Local APIC      |
|   PS/2 & USB HID Kbd/Mouse | AHCI SATA 1.3  | NVMe 1.4 Solid-State Drive |
|   USB 3.0 xHCI Host     | Intel HD Audio    | VirtIO Block & Net         |
+--------------------------------------------------------------------------+
|                        Firmware & Hardware Layer                         |
|   UEFI 2.10 (BOOTX64.EFI) | ACPI 2.0 RSDP   | x86_64 Long Mode (CR0/CR4) |
+--------------------------------------------------------------------------+
```

---

## 🎯 12-Milestone Development Roadmap (100% Certified)

Every milestone defined in the Antigravity Build Contract has been realized, tested, and verified via automated test suites in headless QEMU:

| Milestone | Title | Focus & Key Deliverables | Status |
| :---: | :--- | :--- | :---: |
| **M0** | **Toolchain & Boot Proof** | Pinned Rust toolchain, UEFI target, image builder, serial logger, QEMU boot test, CI | ✅ **Verified** |
| **M1** | **UEFI Boot Manager & Entry** | Memory map, GOP framebuffer discovery, ACPI RSDP, GDT, TSS, IDT, exceptions (#DE, #UD, #PF, #DF) | ✅ **Verified** |
| **M2** | **Memory & Object Model** | 4-level paging, physical frame allocator (4 GiB), canary heap manager (32 MiB), typed capability handles | ✅ **Verified** |
| **M3** | **Interrupts, Devices & Drivers** | PIT 8254 timer, 8259 PIC, PS/2 keyboard/mouse, PCI enumerator, VirtIO block & net, PC speaker audio | ✅ **Verified** |
| **M4** | **Scheduling, Ring 3 & Syscalls** | Preemptive scheduler, user address spaces, ELF64 dynamic loader, 21 system calls, Ring 3 PID 1 Init | ✅ **Verified** |
| **M5** | **Files, Storage & User Profiles** | Unified VFS, ramfs, devfs (`/dev/null`, `/dev/fb0`), NovaFS block persistence, multi-user profiles (`/users/amiyo`) | ✅ **Verified** |
| **M6** | **Networking & Multi-User Security**| Ethernet II, ARP table cache, IPv4 routing, ICMP echo ping, UDP/TCP sockets, multi-user port security | ✅ **Verified** |
| **M7** | **Graphics, Compositor & NovaUI** | 32-bit ARGB double-buffer compositor, dirty rect tracking, window hierarchy, drag-to-move, click-to-focus | ✅ **Verified** |
| **M8** | **Nova Desktop Shell** | Nova Bar taskbar, Nova Launcher start menu with live search, Snap Assist (7 docking zones), 4 Virtual Workspaces | ✅ **Verified** |
| **M9** | **Core Native Applications** | Nova Files, Nova System Monitor, Nova Settings, Nova Package Manager, Nova Terminal | ✅ **Verified** |
| **M10** | **App Platform, Installer & Recovery** | Signed `.nvp` packages (SHA-256 / Ed25519), GPT partitioner (CRC32), automated OS installer, 3-strike recovery scrubber | ✅ **Verified** |
| **M11** | **Hardware Expansion Subsystem** | AHCI SATA 1.3 (`/dev/sda`), NVMe 1.4 SSD (`/dev/nvme0n1`), USB 3.0 xHCI & HID, Intel HD Audio (HDA 1.0a) | ✅ **Verified** |
| **M12** | **Hardening & Release Engineering** | Adversarial syscall fuzzer, runt packet fuzzer, heap canary leak walk, stress testing, release distribution | ✅ **Verified** |

---

## 📂 Repository Structure

```text
novaos-apex/
├── Cargo.toml                  # Workspace manifest
├── rust-toolchain.toml         # Pinned nightly Rust toolchain configuration
├── Makefile                    # Standard targets: build, image, run, debug, test, test-qemu, clean
├── README.md                   # System overview and landing page
├── CHANGELOG.md                # Comprehensive release log (v1.0.0)
├── LICENSE                     # MIT License
├── boot/                       # 64-bit UEFI Boot Manager
│   ├── uefi/                   # UEFI loader application (BOOTX64.EFI)
│   └── linker/                 # Higher-half kernel linker script (kernel.ld)
├── kernel/                     # Monolithic kernel core
│   ├── src/
│   │   ├── arch/x86_64/        # GDT, TSS, IDT, interrupt handlers, CPU registers
│   │   ├── cpu/                # CPUID, control registers, exception dispatch
│   │   ├── memory/             # 4-level paging, frame allocator, canary heap, stacks
│   │   ├── scheduler/          # Preemptive multi-priority scheduler & thread queues
│   │   ├── process/            # Process control blocks, address spaces, ELF64 loader
│   │   ├── syscall/            # Native int 0x80 syscall engine (51 system calls)
│   │   ├── ipc/                # Typed mailbox channels & inter-process messaging
│   │   ├── drivers/            # AHCI SATA, NVMe, USB 3.0 xHCI, HID, HDA, GOP, Timer, UART
│   │   ├── fs/                 # Unified VFS, RAMFS, DevFS, NovaFS on-disk filesystem
│   │   ├── network/            # Ethernet II, ARP, IPv4, ICMP, UDP, TCP stack
│   │   ├── graphics/           # 2D ARGB compositor, window server, Nova Bar, Nova Launcher
│   │   ├── storage/            # GPT partition engine, IEEE 802.3 CRC32, installer, recovery
│   │   └── security/           # Adversarial fuzzing, heap canary audit, conformance engine
├── userland/                   # Ring 3 applications and libraries
│   ├── runtime/                # Freestanding syscall runtime library
│   ├── init/                   # PID 1 system supervisory init daemon
│   ├── shell/                  # Interactive userland CLI shell
│   ├── coreutils/              # Standard command utilities (ls, cat, ping, ifconfig, etc.)
│   ├── toolkit/                # NovaUI graphics toolkit and UI widgets
│   └── apps/                   # Native apps (Files, Monitor, Settings, Packages, Terminal)
├── dist/                       # Production release distribution artifacts
│   ├── novaos-apex-v1.0.0-MANIFEST.json   # Release manifest
│   └── novaos-apex-v1.0.0-SHA256SUMS.txt  # Cryptographic verification checksums
├── tools/                      # Multi-stage image builders, QEMU scripts, test runner
│   ├── build-image.ps1         # Multi-stage OS image builder
│   ├── build-release.ps1       # Distribution package generator
│   ├── run-qemu.ps1            # Interactive QEMU launch script
│   └── test_runner.py          # Automated headless QEMU test harness
└── docs/                       # Comprehensive architecture documentation
    ├── ARCHITECTURE.md         # System architecture manual
    ├── ROADMAP.md              # 12-milestone development roadmap
    ├── SECURITY_AUDIT.md       # Milestone 12 security audit & fuzzing report
    ├── RELEASE_NOTES_v1.0.md   # Official v1.0.0 release notes
    ├── KNOWN_LIMITATIONS.md    # Active technical boundaries & hardware constraints
    ├── HARDWARE_SUPPORT.md     # Hardware driver compatibility guide
    ├── SYSCALL_ABI.md          # 51 system calls reference and register ABI
    └── TEST_PLAN.md            # Acceptance criteria & verification methodology
```

---

## ⚡ Quick Start & Verification

### Prerequisites
- **Rust Toolchain:** Pinned nightly toolchain with `rust-src`, `llvm-tools-preview`, and targets `x86_64-unknown-none` and `x86_64-unknown-uefi`.
- **Emulator:** [QEMU](https://www.qemu.org/) (`qemu-system-x86_64`) with OVMF UEFI firmware.
- **Python 3:** For automated headless test runner.

### 1. Build Operating System Image
```bash
# On Linux / macOS:
make image

# On Windows PowerShell:
.\tools\build-image.ps1
```

### 2. Run Automated Verification Test Suite
Executes the headless test runner asserting all exception gates, memory allocators, hardware drivers, desktop composition, security fuzzing, and Ring 3 userspace execution:
```bash
python tools/test_runner.py
```

### 3. Launch Interactive Graphical Desktop in QEMU
```bash
# On Linux / macOS:
make run

# On Windows PowerShell:
.\tools\run-qemu.ps1
```

### 4. Build Production Release Package
Generates distribution artifacts with SHA-256 integrity checksums in `dist/`:
```powershell
.\tools\build-release.ps1
```

---

## 📚 Documentation Index

- [Windows 11 Feature Catalog (105 Gates)](docs/WINDOWS_FEATURES.md)
- [System Architecture Manual](docs/ARCHITECTURE.md)
- [12-Milestone Development Roadmap](docs/ROADMAP.md)
- [Security Audit & Fuzzing Report](docs/SECURITY_AUDIT.md)
- [v1.0.0 Release Notes](docs/RELEASE_NOTES_v1.0.md)
- [Hardware Support & Drivers](docs/HARDWARE_SUPPORT.md)
- [System Call ABI & Reference](docs/SYSCALL_ABI.md)
- [Known Limitations Log](docs/KNOWN_LIMITATIONS.md)
- [Test Plan & Verification Methodology](docs/TEST_PLAN.md)

---

## 📜 License

NovaOS Apex is licensed under the [MIT License](LICENSE).
