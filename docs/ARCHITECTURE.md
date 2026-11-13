# NovaOS Architecture Manual

**Target Architecture:** `x86_64` (64-Bit Long Mode)  
**Privilege Model:** Hardware Ring 0 (Kernel) & Ring 3 (Userland)  
**Language:** Rust (`#![no_std]`, freestanding)  
**Firmware:** UEFI with Graphics Output Protocol (GOP)  

---

## 1. System Topology

```text
+-----------------------------------------------------------------+
|                       Userland Applications                     |
|            [ init ]      [ shell ]      [ coreutils ]           |
+-----------------------------------------------------------------+
                                 |  SYSCALL / SYSRET
+-----------------------------------------------------------------+
|                         NovaOS Kernel                           |
|                                                                 |
|   +-----------------------+     +---------------------------+   |
|   | Scheduler / Tasks     |     | Memory Management (Paging)|   |
|   | (Round Robin, TCB)    |     | (PML4, Frame Alloc, Heap) |   |
|   +-----------------------+     +---------------------------+   |
|               |                               |                 |
|   +-----------------------+     +---------------------------+   |
|   | Interrupt Dispatcher  |     | Virtual Filesystem (VFS)  |   |
|   | (IDT, Exceptions, PIC)|     | (RamFS, In-Memory Storage)|   |
|   +-----------------------+     +---------------------------+   |
|               |                               |                 |
|   +-----------------------+     +---------------------------+   |
|   | Descriptor Tables     |     | Device Drivers            |   |
|   | (GDT, TSS, IST Stack) |     | (UART, VGA, GOP, PS/2)    |   |
|   +-----------------------+     +---------------------------+   |
+-----------------------------------------------------------------+
                                 |
+-----------------------------------------------------------------+
|                       x86_64 Physical / QEMU CPU                |
+-----------------------------------------------------------------+
```

---

## 2. Core Subsystems

### 2.1 CPU & Hardware Abstraction (`kernel/src/arch/x86_64/`)
* **Port I/O:** Type-safe inline assembly primitives for `inb`, `outb`, `inw`, `outw`, `inl`, `outl`.
* **Control Registers:** Inspection of `CR2` (Page fault address) and `CR3` (PML4 base address).
* **GDT & TSS:** Configures kernel code and data selectors alongside an emergency Interrupt Stack Table (IST) for double-fault isolation.
* **IDT:** 256-gate interrupt descriptor table capturing CPU faults and hardware IRQs.

### 2.2 Memory Management (`kernel/src/memory/`)
* **Physical Frame Allocator:** Sequentially maps 4096-byte frames from usable regions in the UEFI memory map.
* **4-Level Paging:** Hierarchical translation through PML4, PDPT, PD, and PT.
* **Dynamic Kernel Heap:** Initial 256 KiB heap mapped at `0x4444_4444_0000` supporting `alloc` collections.

### 2.3 Device Drivers (`kernel/src/drivers/`)
* **UART 16550:** Serial diagnostics at `0x3F8` mapped to QEMU stdio.
* **VGA Text Mode:** 80x25 memory-mapped buffer at `0xb8000`.
* **GOP Framebuffer:** Direct 32-bit linear RGB pixel rendering on UEFI hardware.
* **PS/2 Keyboard:** Interrupt-driven scancode receiver and ASCII translator.
* **PCI Scanner:** Configuration space reader at ports `0xCF8` / `0xCFC`.
