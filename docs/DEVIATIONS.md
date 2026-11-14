# NovaOS Architecture Deviations & Adoption Log

**Standard Reference:** CONTRACT.md v3.1 / Antigravity Build Protocol  
**Date:** 2026-11-14  

---

## Active Architectural Deviations

### DEV-001: Build Tooling Execution on Windows Host
- **Context:** The build protocol specifies `make build`, `make test`, and `make evidence`.
- **Environment Reality:** The host workspace is running Windows 11 with PowerShell (`pwsh`) and native Rust/Cargo toolchains without native GNU Make / coreutils on standard PATH.
- **Resolution:**
  - `tools/build-image.ps1` executes the multi-package Cargo build sequence and formats the FAT filesystem.
  - `tools/test_runner.py` runs headless QEMU, asserts all test gates, and validates the supervisory loop.
  - `Makefile` has been updated with an `evidence` target for POSIX/GNU environments, while Python/PowerShell scripts provide 1:1 identical automation on Windows.

### DEV-002: Pre-existing Storage & Audio Expansion Drivers
- **Context:** The post-1.0 milestone list schedules `M15` (USB xHCI, AHCI/SATA, NVMe behind block layer) and `M17` (virtio-snd then Intel HDA).
- **Existing Code:** Hardware drivers for AHCI SATA 1.3, NVMe 1.4, USB 3.0 xHCI, and Intel HDA 1.0a were initially written and tested in `kernel/src/drivers/`.
- **Resolution:**
  - These drivers remain intact per the absolute rule: *Never delete, move, or rewrite existing working code.*
  - When reaching M15 and M17, these drivers will be formally unified behind the VFS block and audio layers with the required serial markers, fuzz targets, and test gates without deleting existing code.

### DEV-003: Formal CONTRACT.md Document Availability
- **Context:** The user prompt refers to `CONTRACT.md (v3.1)`. The repository does not currently contain this file at the root.
- **Resolution:** The core contract rules (Adoption Mode, Additive-only changes, Evidence-backed verification, One milestone per session, Prior approval before modifying existing source files) are explicitly recorded and enforced across `docs/STATE.md`, `docs/DEVIATIONS.md`, and session workflows.
