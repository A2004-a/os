# NovaOS — Bare-Metal x86_64 Operating System Kernel

[![Architecture](https://img.shields.io/badge/arch-x86__64-blue.svg)](https://en.wikipedia.org/wiki/X86-64)
[![Language](https://img.shields.io/badge/language-Rust%20(no__std)-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)

**NovaOS** is a genuine, freestanding bare-metal operating system kernel built from line zero in Rust. It boots on physical computer hardware or emulated hypervisors (QEMU, VirtualBox, VMware) with full control over Ring 0 CPU execution, memory paging, descriptor tables, hardware interrupts, and device drivers.

---

## Key Kernel Capabilities

* **UEFI Bootloader Handover:** Boots directly into 64-bit Long Mode via typed bootloader protocol.
* **Global Descriptor Table (GDT) & TSS:** Configures kernel segments with an emergency **Interrupt Stack Table (IST)** to guarantee recovery from stack overflows without triple faulting.
* **Interrupt Descriptor Table (IDT):** Complete exception matrix (Divide Error, Breakpoint, General Protection Fault, Page Fault with CR2 diagnosis) and remapped 8259 PIC IRQ lines.
* **4-Level Virtual Memory Paging:** Level 4 PML4 paging tables with physical-memory offset mapping and UEFI memory region allocator.
* **Dynamic Kernel Heap:** 256 KiB heap mapped at `0x4444_4444_0000` supporting `alloc` collections (`Box`, `Vec`, `String`, `Arc`).
* **Hardware Drivers:**
  * **UART 16550 Serial (COM1):** Direct kernel diagnostics via `serial_println!`.
  * **VGA Text Buffer (0xb8000):** Memory-mapped 80x25 color terminal with automated row scrolling via `println!`.
  * **PS/2 Keyboard:** Hardware interrupt driver with asynchronous scancode decoder.
* **Multitasking Engine:** Waker-driven async executor that puts the CPU into low-power sleep via `hlt` when idle.

---

## Directory Structure

```text
├── .cargo/
│   └── config.toml               # Cargo runner for QEMU, target spec, rustflags
├── Cargo.toml                    # Kernel manifest and dependencies
├── rust-toolchain.toml           # Pins nightly toolchain & bare-metal components
├── x86_64-novaos.json            # Custom bare-metal target specification
├── src/
│   ├── main.rs                   # Kernel entry point (_start) & banner
│   ├── lib.rs                    # Reusable kernel library & test runner
│   ├── vga_buffer.rs             # VGA text driver (print!/println!)
│   ├── serial.rs                 # UART 16550 serial driver (serial_print!)
│   ├── gdt.rs                    # GDT & Task State Segment (TSS) with IST
│   ├── interrupts.rs             # IDT, CPU exceptions, 8259 PIC, and timer
│   ├── memory.rs                 # Physical frame allocator & 4-level paging
│   ├── allocator.rs              # Dynamic heap allocator (Box, Vec, String)
│   └── task/                     # Async multitasking subsystem
│       ├── mod.rs                # Task and TaskId abstractions
│       ├── executor.rs           # Waker-driven preemptive/idle executor
│       ├── simple_executor.rs    # Basic FIFO test executor
│       └── keyboard.rs           # Asynchronous interrupt-driven keyboard queue
├── tests/
│   ├── basic_boot.rs             # Automated QEMU boot test
│   ├── should_panic.rs           # Panic recovery verification
│   └── stack_overflow.rs         # Double fault IST stack overflow verification
├── scripts/
│   ├── install-toolchain.ps1     # Automated Windows toolchain setup
│   ├── build.ps1                 # Build bootable raw disk image (.bin)
│   └── run.ps1                   # Launch kernel inside QEMU emulator
└── ARCHITECTURE.md               # Detailed systems and architecture manual
```

---

## Quickstart

### 1. Automated Toolchain Installation
Run PowerShell as Administrator:
```powershell
.\scripts\install-toolchain.ps1
```
*This installs Rust Nightly, `rust-src`, `llvm-tools-preview`, `cargo-bootimage`, and QEMU for x86_64.*

### 2. Build the Kernel & Bootable Disk Image
```powershell
.\scripts\build.ps1
```
*Produces `target\x86_64-novaos\debug\bootimage-novaos-kernel.bin`.*

### 3. Launch NovaOS inside QEMU
```powershell
.\scripts\run.ps1
```
*Launches the virtual machine with 512 MB RAM, serial output mapped to stdio, and interactive keyboard input.*

### 4. Run Automated Hardware Verification Tests
```powershell
cargo test
```

---

## Technical Documentation
For in-depth explanations of 4-level paging, IDT exception vectors, TSS stacks, and async waker implementation, refer to:
* **[ARCHITECTURE.md](file:///c:/Users/amiyo%20garai/OneDrive/Documents/AMIYO%20OS/ARCHITECTURE.md)**: Full kernel systems manual.
