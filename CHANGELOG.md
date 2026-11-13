# NovaOS Kernel Changelog

All notable changes to the NovaOS bare-metal operating system kernel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0-baremetal] - 2026-11-14

### Added
- **Clean Slate Bare-Metal Architecture**: Initialized freestanding `x86_64` operating system kernel in pure Rust (`#![no_std]`, `#![no_main]`).
- **UEFI Bootloader Handover**: Integrated `bootloader` crate for 64-bit Long Mode entry, linear framebuffer mapping, and physical memory offset extraction.
- **Global Descriptor Table (GDT) & TSS**:
  - Implemented kernel code segment descriptor.
  - Configured Task State Segment (TSS) with an emergency Interrupt Stack Table (IST) on Double Faults, preventing fatal triple faults on stack overflow.
- **Interrupt Descriptor Table (IDT)**:
  - 32 CPU Exception handlers including Division Error, Breakpoint, General Protection Fault, and Page Fault with CR2 diagnosis.
  - Chained 8259 Programmable Interrupt Controller (PIC) remapping IRQs to vectors `32..47`.
  - Programmable Interval Timer (PIT / IRQ0) tick handler.
  - PS/2 Keyboard (IRQ1) hardware interrupt handler.
- **Memory Management Subsystem**:
  - 4-level paging (`PML4` -> `PDPT` -> `PD` -> `PT`) via `x86_64` offset page table mapping.
  - `BootInfoFrameAllocator` for physical 4096-byte page frame allocation from UEFI memory descriptors.
- **Dynamic Kernel Heap Allocator**:
  - 256 KiB heap mapped at virtual address `0x4444_4444_0000`.
  - Linked-list allocator enabling standard `alloc` primitives: `Box`, `Vec`, `String`, `Arc`, `BTreeMap`.
- **Hardware Device Drivers**:
  - UART 16550 serial port at COM1 (`0x3F8`) with `serial_print!` and `serial_println!` macros.
  - VGA 80x25 text mode buffer (`0xb8000`) with thread-safe spinlock wrapper and automated scrolling.
  - PS/2 Keyboard driver with asynchronous scancode queue.
- **Preemptive & Cooperative Multitasking**:
  - Waker-driven async executor with `Task` and `TaskId` management.
  - Low-power CPU halting via `hlt` instruction during idle periods.
- **Testing & Emulation Harness**:
  - Integration test suite: `basic_boot`, `should_panic`, and `stack_overflow`.
  - Windows automation scripts for toolchain setup (`install-toolchain.ps1`), image building (`build.ps1`), and QEMU execution (`run.ps1`).

---

## [0.1.0-electron-prototype] - 2026-11-14
- *Archived hybrid desktop environment prototype (tagged in Git as `v0.1.0-electron-prototype`).*
