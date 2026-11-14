# NovaOS Apex Changelog

All notable changes to NovaOS Apex are documented in this file.

## [0.2.0] - 2026-11-14 — Milestone 2 (Memory and Kernel Object Model)

### Added
- **Physical Frame Allocator (`kernel/src/memory/frame.rs`):**
  - Parses UEFI memory map descriptors into conventional and usable physical RAM ranges.
  - Bitmap-backed physical allocator managing 4 GiB of physical space at 4 KiB page granularity (1,048,576 frames).
  - Single frame and contiguous multi-frame allocation and deallocation.
  - Enforced double-free detection, bounds checking, and physical memory reservation for low memory (1 MiB), kernel text/data, framebuffer, and UEFI tables.
- **Four-Level Paging Subsystem (`kernel/src/memory/paging.rs`):**
  - Full x86_64 PML4, PDPT, PD, and PT page directory tables.
  - 1 GiB lower identity mapping (2 MiB huge pages) and 1 GiB higher-half direct physical map at `0xFFFF_8000_0000_0000`.
  - Virtual memory mapper/unmapper with fine-grained access flags (`PRESENT`, `WRITABLE`, `GLOBAL`, `NO_EXECUTE`, `USER`).
  - Active conflict detection (`AlreadyMapped`) and translation engine.
  - New kernel PML4 loaded into `CR3`.
- **Kernel Dynamic Heap Allocator (`kernel/src/memory/heap.rs`):**
  - Global memory allocator (`GlobalAlloc`) backing Rust `extern crate alloc`.
  - 2 MiB initial kernel heap buffer with slab/block allocation.
  - Prefix canary (`0xCAFEBABE_DEADBEEF`) and suffix canary (`0xFEEDFACE_BEEFCAFE`) for buffer underflow/overflow detection.
  - Memory poisoning on free (`0xAA`) and double-free rejection.
  - Debug allocation tracker recording active bytes, peak usage, and allocation counts.
- **Kernel Stacks with Hardware Guard Pages (`kernel/src/memory/stack.rs`):**
  - Stack allocation placing an unmapped guard page at the bottom of each kernel stack to catch stack overflows via `#PF`.
- **Kernel Object & Typed Handle Subsystem (`kernel/src/objects/`):**
  - Strongly-typed enum covering all 8 fundamental kernel object categories: `Process`, `Thread`, `File`, `Socket`, `Event`, `Device`, `Window`, and `SharedMemory`.
  - Fine-grained capability rights bitflags (`READ`, `WRITE`, `EXECUTE`, `TRANSFER`, `DUPLICATE`, `CLOSE`, `MAP`, `SIGNAL`, `WAIT`).
  - Typed handle table (256 slots) preventing type confusion attacks.
  - Handle duplication with capability masking and reference-counted object lifecycle reclamation.
- **Automated Verification Suites:**
  - Memory subsystem test suite (`memory/tests.rs`) covering frame allocation, contiguous runs, double-free rejection, page mapping, translation, unmapping, heap alloc/dealloc, canaries, and stack guard pages.
  - Object model test suite (`objects/tests.rs`) covering typed object creation, type confusion rejection, capability rights enforcement, handle duplication, and reclamation on close.

## [0.1.0] - 2026-11-14 — Milestone 1 (UEFI Boot Manager & Kernel Entry)

### Added
- UEFI SimpleFileSystem reader loading `/novaos.cfg` and `/novaos-apex.elf` from FAT volumes.
- Freestanding ELF64 loader and validator checking magic, 64-bit, little-endian, EM_X86_64, and loading PT_LOAD segments.
- GOP linear framebuffer discovery collecting base physical address, resolution, and scanline stride.
- ACPI 2.0 / 1.0 RSDP discovery from UEFI configuration tables.
- UEFI memory map extraction via `GetMemoryMap`.
- Clean `ExitBootServices` firmware handoff with retry resilience.
- Stable `BootInfo` structure shared between bootloader and kernel.
- 64-bit GDT configuring kernel code (0x08), kernel data (0x10), user data (0x18|3), user code (0x20|3), and TSS (0x28).
- Task State Segment (TSS) with dedicated 32 KiB emergency double-fault stack on IST1.
- 256-gate IDT with trampolines for all 32 x86_64 architecture exceptions.
- Serial panic subsystem dumping registers, stack pointer, and file location.
- Deliberate exception verification suite testing `#DE`, `#UD`, `#PF`, and `#DF`.

## [0.0.1] - 2026-11-14 — Milestone 0 (Toolchain & Boot Proof)

### Added
- Root Cargo workspace targeting freestanding Rust toolchain on `x86_64-unknown-none` and `x86_64-unknown-uefi`.
- Custom linker script (`boot/linker/kernel.ld`) configuring higher-half virtual address mapping at `0xFFFF_8000_0020_0000`.
- 64-bit UEFI Boot Manager (`boot/uefi`) with COM1 UART serial output.
- Cross-platform build and execution tooling in `tools/`.
- Continuous Integration workflow (`.github/workflows/ci.yml`).
