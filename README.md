# NovaOS — Bare-Metal x86_64 Operating System

[![Architecture](https://img.shields.io/badge/arch-x86__64-blue.svg)](https://en.wikipedia.org/wiki/X86-64)
[![Language](https://img.shields.io/badge/language-Rust%20(no__std)-orange.svg)](https://www.rust-lang.org/)
[![Firmware](https://img.shields.io/badge/firmware-UEFI%20(OVMF)-purple.svg)](https://uefi.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)

**NovaOS** is an original, genuine bare-metal operating system built from an empty repository in freestanding Rust (`#![no_std]`, `#![no_main]`).

It boots natively through 64-bit UEFI on physical hardware and QEMU virtual machines. It has zero dependency on Electron, Chromium, Linux, Windows, macOS, or any external hosted runtime.

---

## 1. Verified Architecture

```text
novaos/
├── Cargo.toml                    # Root workspace manifest
├── rust-toolchain.toml           # Pins nightly toolchain & targets (none, uefi)
├── Makefile                      # Standard developer build automation
├── README.md                     # System overview & getting started guide
├── LICENSE                       # Apache-2.0 open-source license
├── docs/                         # Systems engineering specifications
│   ├── ARCHITECTURE.md           # Subsystem topology and execution model
│   ├── BOOT_PROCESS.md           # UEFI handover protocol and Long Mode entry
│   ├── MEMORY_MODEL.md           # 4-level paging and kernel heap map
│   ├── SYSCALL_ABI.md            # Calling conventions and system call table
│   ├── DRIVER_MODEL.md           # Device driver specifications
│   ├── SECURITY_MODEL.md         # Ring 0 / Ring 3 privilege separation
│   ├── TEST_PLAN.md              # QEMU exit codes and automated tests
│   ├── HARDWARE_SUPPORT.md       # Hardware and emulator compatibility matrix
│   ├── ROADMAP.md                # 12-milestone engineering schedule
│   └── KNOWN_LIMITATIONS.md      # Transparent boundaries and status report
├── boot/
│   ├── uefi-loader/              # 64-bit UEFI loader (BOOTX64.EFI)
│   └── linker/                   # Bare-metal linker scripts (kernel.ld)
├── kernel/                       # Core x86_64 microkernel
│   ├── src/
│   │   ├── arch/x86_64/          # CPU GDT, TSS, IDT, port I/O, control registers
│   │   ├── boot/                 # BootInfo ABI contract
│   │   ├── cpu/                  # CPUID discovery and feature detection
│   │   ├── memory/               # 4-level paging, frame allocator, dynamic heap
│   │   ├── interrupts/           # 8259 PIC remapping and PIT timer ticks
│   │   ├── scheduler/            # Round-robin scheduler and task management
│   │   ├── process/              # Process Control Blocks and address spaces
│   │   ├── syscall/              # Fast syscall dispatcher
│   │   ├── drivers/              # UART 16550, VGA, GOP framebuffer, PS/2, PCI
│   │   ├── fs/                   # Virtual Filesystem & RamFS
│   │   ├── ipc/                  # Ring-buffer message passing channels
│   │   └── graphics/             # 2D drawing primitives & console font
│   └── tests/                    # QEMU integration tests
├── userland/
│   ├── init/                     # PID 1 userspace entry stub
│   ├── shell/                    # Interactive command shell
│   └── coreutils/                # Fundamental system utilities
└── tools/
    ├── build-image.ps1           # Automated FAT32 EFI disk builder
    ├── run-qemu.ps1              # QEMU launcher with OVMF firmware & serial stdio
    └── debug-gdb.ps1             # GDB debugger configuration
```

---

## 2. Developer Commands

### On Windows (PowerShell):
```powershell
# 1. Build UEFI Bootloader and Kernel Image
.\tools\build-image.ps1

# 2. Boot NovaOS in QEMU (OVMF UEFI + Serial stdio)
.\tools\run-qemu.ps1

# 3. Headless Verification Testing in QEMU
.\tools\run-qemu.ps1 -NoGui

# 4. Attach GDB Debugger
.\tools\debug-gdb.ps1
```

### On Linux / macOS / WSL (Make):
```bash
make build       # Compile loader and kernel
make image       # Generate bootable disk directory
make run         # Launch QEMU with OVMF
make debug       # Launch paused with GDB server on :1234
make test-qemu   # Headless automated boot test
make test        # Run host unit tests
```

---

## 3. Engineering Documentation
* [ARCHITECTURE.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/ARCHITECTURE.md)
* [BOOT_PROCESS.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/BOOT_PROCESS.md)
* [MEMORY_MODEL.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/MEMORY_MODEL.md)
* [SYSCALL_ABI.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/SYSCALL_ABI.md)
* [DRIVER_MODEL.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/DRIVER_MODEL.md)
* [SECURITY_MODEL.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/SECURITY_MODEL.md)
* [TEST_PLAN.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/TEST_PLAN.md)
* [HARDWARE_SUPPORT.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/HARDWARE_SUPPORT.md)
* [ROADMAP.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/ROADMAP.md)
* [KNOWN_LIMITATIONS.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/docs/KNOWN_LIMITATIONS.md)
