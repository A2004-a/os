# NovaOS Apex

**NovaOS Apex** is an original, bare-metal desktop operating system engineered in freestanding Rust (`#![no_std]`, `#![no_main]`) for 64-bit x86_64 hardware booted via standard UEFI firmware.

Apex combines an original microkernel/modular-monolithic architecture with modern desktop expectations: smooth workflows, capability-based sandboxing, a 2D compositing window server, signed application packages (`.nvp`), and resilient system recovery.

---

## Technical Foundation

- **Target Architecture:** x86_64 (AMD64)
- **Firmware Support:** UEFI 2.10 (PE32+ boot manager)
- **Kernel Language:** Freestanding Rust (`no_std`, `no_main`) with minimal isolated assembly
- **Kernel Model:** Modular monolithic kernel with typed capability handles
- **Display Pipeline:** UEFI GOP linear framebuffer to 2D Software Compositor & Window Server
- **Application Model:** Ring 3 address spaces with signed Nova Packages (`.nvp`)

---

## Directory Structure

```text
novaos-apex/
├── Cargo.toml                  # Cargo workspace definition
├── rust-toolchain.toml         # Pinned nightly Rust toolchain
├── Makefile                    # Standard targets: build, image, run, debug, test, test-qemu, clean
├── README.md                   # System overview and entry point
├── CHANGELOG.md                # Detailed release notes and milestone log
├── LICENSE                     # MIT License
├── boot/
│   ├── uefi/                   # 64-bit UEFI Boot Manager (BOOTX64.EFI)
│   └── linker/                 # Higher-half kernel linker script (kernel.ld)
├── kernel/                     # Monolithic kernel core
├── userland/
│   ├── runtime/                # Ring 3 syscall runtime bindings (int 0x80)
│   ├── init/                   # PID 1 system initialization daemon
│   ├── shell/                  # Interactive command shell
│   ├── coreutils/              # Standard command utilities
│   ├── toolkit/                # NovaUI graphics toolkit primitives
│   └── apps/                   # Native core applications
├── tools/                      # Cross-platform build and QEMU test scripts
└── docs/                       # Complete architecture manuals
```

---

## Quick Start

### Build Image
```bash
make image
# Or on Windows PowerShell:
.\tools\build-image.ps1
```

### Run in QEMU
```bash
make run
# Or on Windows PowerShell:
.\tools\run-qemu.ps1
```

### Run Headless Smoke Test
```bash
make test-qemu
# Or on Windows PowerShell:
.\tools\test-qemu.ps1
```

---

## Project Status

- **Milestone 0 (Toolchain & Boot Proof):** **Complete**
- Current Target: **Milestone 1 (UEFI Boot Manager & Kernel Entry)**
