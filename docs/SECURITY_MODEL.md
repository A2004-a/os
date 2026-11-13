# NovaOS Security Model Specification

**Security Level:** Ring 0 (Kernel) & Ring 3 (Userland) Separation  
**Memory Safety:** Rust Type Invariants & MMU Hardware Protection  

---

## 1. Hardware Privilege Levels

NovaOS relies on hardware-enforced CPU privilege rings on `x86_64`:
* **Ring 0 (Kernel Space):** Executes microkernel code, direct hardware I/O instructions (`in`/`out`), page table mutations, and interrupt dispatch.
* **Ring 3 (Userland Space):** Constrained execution environment for applications, shells, and background servers. Cannot execute privileged instructions (`cli`, `sti`, `hlt`, `lidt`, `lgdt`, `mov crX`).

---

## 2. Address Space Isolation

Every userland process operates within its own dedicated 4-level paging structure:
* **Higher-Half Kernel Mapping:** The kernel mappings (`0xFFFF_8000_0000_0000`+) are marked **Supervisor-Only** (clearing the `USER_ACCESSIBLE` bit in page table entries). Any userland attempt to read or write to kernel memory results in an immediate CPU Page Fault.
* **Lower-Half User Mapping:** Userland code and heap are mapped between `0x0000_0000_0000_0000` and `0x0000_7FFF_FFFF_FFFF`.
* **No-Execute (NX) Enforcement:** All data, heap, and stack pages are flagged with bit 63 (`NO_EXECUTE`), preventing arbitrary code execution in writable buffers.

---

## 3. Kernel Stack Protection & IST Emergency Handling

To defend against kernel stack exhaustion and silent memory corruption:
* A 20 KiB dedicated emergency stack is allocated and wired to **Interrupt Stack Table (IST)** index 0 in the Task State Segment (TSS).
* When a kernel stack overflow triggers a Double Fault, the CPU switches stacks to IST0 automatically, guaranteeing safe diagnostic logging and thread termination rather than triple-faulting the processor.

---

## 4. System Call Boundary Validation

All userland-to-kernel communication via `SYSCALL` enforces validation rules:
* **Pointer Confinement:** Userland buffer pointers must reside strictly within lower-half canonical address space (`< 0x0000_8000_0000_0000`).
* **Length Bounds:** Maximum single-transaction buffer sizes are clamped (e.g. 4 KiB max per chunk).
* **Capability Handles:** File descriptors and process handles reference per-process lookup tables; handles cannot be forged or accessed cross-process.
