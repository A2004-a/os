# NovaOS Apex Changelog

All notable changes to NovaOS Apex are documented in this file.

## [0.1.0] - 2026-11-14 — Milestone 1 (UEFI Boot Manager & Kernel Entry)

### Added
- **UEFI Boot Manager Handoff:**
  - UEFI SimpleFileSystem protocol reader capable of loading `/novaos.cfg` and `/novaos-apex.elf` directly from FAT volumes.
  - Freestanding ELF64 executable loader and validator (`boot/uefi/src/elf.rs`) checking magic, 64-bit architecture, little-endian, machine type (x86_64), and allocating memory for PT_LOAD segments.
  - Automatic GOP linear framebuffer discovery collecting base physical address, size, resolution, and scanline stride.
  - ACPI 2.0 (`8868e871...`) and ACPI 1.0 (`eb9d2d30...`) RSDP pointer discovery via UEFI configuration tables.
  - UEFI memory map collector querying physical memory descriptors via `GetMemoryMap`.
  - Clean `ExitBootServices` firmware handoff with retry resilience.
  - Unified `BootInfo` structure shared between UEFI bootloader and kernel.
- **Kernel Architecture Subsystem:**
  - 64-bit GDT configuring kernel code (0x08), kernel data (0x10), user data (0x18|3), user code (0x20|3), and 16-byte TSS descriptor (0x28).
  - Task State Segment (TSS) with dedicated 32 KiB emergency double-fault stack assigned to IST1.
  - 256-gate Interrupt Descriptor Table (IDT) with handlers for all 32 x86_64 architecture exceptions.
  - Dedicated naked assembly ISR trampolines saving and restoring full CPU context (`rax`..`r15`, `rflags`, `rip`, `rsp`, `cs`, `ss`).
  - Serial panic subsystem (`kernel/src/panic.rs`) dumping faulting registers, stack pointer, and file location.
- **Deliberate Diagnostic Verification Suite:**
  - Test 1: Divide-by-Zero (`#DE`, vector 0) handled with instruction skip and diagnostic output.
  - Test 2: Invalid Opcode (`#UD`, vector 6) handled with `ud2` skip and diagnostic output.
  - Test 3: Page Fault (`#PF`, vector 14) reading faulting virtual address from `CR2` and error bits.
  - Test 4: Double Fault (`#DF`, vector 8) verifying hardware stack switch to dedicated 32 KiB IST1 emergency stack, proving zero silent resets.

## [0.0.1] - 2026-11-14 — Milestone 0 (Toolchain & Boot Proof)

### Added
- Root Cargo workspace targeting freestanding Rust toolchain on `x86_64-unknown-none` and `x86_64-unknown-uefi`.
- Pinned `rust-toolchain.toml` with `rust-src` and `llvm-tools-preview`.
- Custom linker script (`boot/linker/kernel.ld`) configuring higher-half virtual address mapping at `0xFFFF_8000_0020_0000`.
- 64-bit UEFI Boot Manager (`boot/uefi`) implementing early UART 16550 serial output at COM1 (`0x3F8`), GOP framebuffer discovery, and UEFI system table handoff.
- Cross-platform build and execution tooling in `tools/`:
  - `build-image` / `build-image.ps1`
  - `run-qemu` / `run-qemu.ps1`
  - `debug-gdb` / `debug-gdb.ps1`
  - `test-qemu` / `test-qemu.ps1`
- Root `Makefile` implementing required commands: `build`, `image`, `run`, `debug`, `test`, `test-qemu`, `clean`.
- Continuous Integration workflow (`.github/workflows/ci.yml`) for automated UEFI compilation and QEMU smoke testing.
- Complete documentation set in `docs/` covering architecture, boot, memory, drivers, security, and known limitations.
