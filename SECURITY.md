# NovaOS Kernel Security Architecture

**Security Level:** Hardware Ring 0 Privilege Enforcement  
**Memory Model:** Type-Safe Rust Paging with NX/WP Flags  

---

## 1. Hardware Privilege Separation (Rings)

NovaOS enforces strict hardware execution privilege boundaries using the x86_64 architecture:
* **Ring 0 (Kernel Mode):** Core kernel, page table manipulation, interrupt handlers, hardware I/O ports.
* **Ring 3 (Userland Mode):** Application processes restricted from executing privileged instructions (`CLI`, `STI`, `HLT`, `LIDT`, `LGDT`) or writing to `CR0..CR4`.

---

## 2. Memory Safety & Paging Protection

### 2.1 Compile-Time Memory Guarantees
By building the kernel in Rust (`#![no_std]`), NovaOS eliminates entire classes of critical kernel vulnerabilities at compile time:
* Zero buffer overflows or spatial memory violations in safe code.
* Zero use-after-free or double-free defects via ownership and lifetime tracking.
* Safe shared state synchronization via `spin::Mutex` preventing kernel data races.

### 2.2 Page Table Flags
Every virtual page mapped by the kernel sets hardware MMU security bits:
* `PRESENT (Bit 0)`: Verified before translation.
* `WRITABLE (Bit 1)`: Read-only segments (e.g., kernel code, constants) have writable bit cleared, causing immediate page fault on write attempts.
* `USER_ACCESSIBLE (Bit 2)`: Ring 3 code cannot access Ring 0 pages.
* `NO_EXECUTE / NX (Bit 63)`: Data pages, stack pages, and kernel heap pages are marked non-executable, mitigating shellcode injection.

---

## 3. Kernel Stack Overflow & IST Protection

In traditional C operating systems, a kernel stack overflow clobbers adjacent memory or triggers an immediate hardware triple fault and reboot.

NovaOS configures an **Interrupt Stack Table (IST)** in the Task State Segment (TSS):
* `DOUBLE_FAULT_IST_INDEX = 0`: Points to an emergency 20 KiB isolated memory stack.
* When a stack overflow occurs, the CPU automatically switches to this known-good emergency stack, safely executing the double fault handler and logging registers rather than resetting the machine.

---

## 4. Redzone Invalidation

Standard System V AMD64 ABI permits a 128-byte "redzone" below the stack pointer `%rsp`. In kernel space, asynchronous hardware interrupts (such as PIT timer or keyboard) can fire at any clock cycle, pushing registers to the stack.

NovaOS configures `"disable-redzone": true` in `x86_64-novaos.json` to prevent hardware interrupt frames from corrupting ephemeral stack data.
