# NovaOS Verification & Test Plan

**Test Harness:** QEMU `isa-debug-exit` (`iobase=0xf4,iosize=0x04`) + Serial Output Stream  
**Integration Strategy:** Automated headless execution in CI and local PowerShell / Bash scripts  

---

## 1. Test Levels

### 1.1 Host-Side Rust Unit Tests (`cargo test --lib`)
* Validates algorithms that do not require physical hardware access:
  * Address parsing and canonical virtual address checks.
  * Bitfield manipulations for page table entries.
  * Syscall argument validation tables.
  * In-memory file system (RamFS) path matching and node searches.

### 1.2 QEMU Headless Integration Tests
* Executes bare-metal kernel in QEMU:
  * `basic_boot`: Validates that UEFI loader successfully branches to `kernel_entry` and serial output reaches stdio.
  * `should_panic`: Verifies that kernel assertions correctly invoke the panic handler and log diagnostic frames.
  * `stack_overflow`: Deliberately induces infinite recursion to verify that the TSS Interrupt Stack Table catches the double fault without crashing the VM.

---

## 2. QEMU Test Exit Protocol

Tests communicate success or failure to QEMU via an emulated I/O port device:
* Port: `0xf4`
* Success Exit Code: `(0x10 << 1) | 1` = `33`
* Failure Exit Code: `(0x11 << 1) | 1` = `35`

---

## 3. Automated Test Execution

```powershell
# Run host unit tests
cargo test --package novaos-kernel --lib

# Run full QEMU test suite
.\tools\run-qemu.ps1 -NoGui
```
