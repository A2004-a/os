# NovaOS Apex Changelog

All notable changes to NovaOS Apex are documented in this file.

## [Unreleased] - Milestone 0 (Toolchain & Boot Proof)

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
