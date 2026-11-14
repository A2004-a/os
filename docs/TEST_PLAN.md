# NovaOS Apex — Test Plan & Quality Assurance Gates

## Multi-Layer Verification Hierarchy

Every milestone must satisfy strict test gates before merging:

```text
[Layer 1: Unit & Subsystem Tests]
  - Cargo check and test suites in kernel and userland crates
  - Parameter boundary and unaligned reference safety checks

[Layer 2: Host & Headless Simulation]
  - QEMU headless test runs capturing serial log output
  - ISA debug-exit port 0xF4 for automatic test harness completion

[Layer 3: Fault Injection & Fuzzing]
  - Controlled divide-by-zero, page fault, and double-fault injection
  - 14-syscall hostile argument fuzzer (null pointers, kernel addresses, arithmetic wraps)
  - Network frame fuzzer (truncated Ethernet, ARP, IPv4, TCP, UDP)

[Layer 4: End-to-End Boot Verification]
  - Real UEFI boot proof on QEMU / OVMF and physical x86_64 PC hardware
```

## Milestone Gates

- **Milestone 0:** Pinned toolchain, clean build, serial boot confirmation.
- **Milestones 1–6:** Kernel primitives, memory safety, driver initialization, Ring 3 isolation, VFS/storage, network stack.
- **Milestones 7–10:** Compositor, desktop shell, package manager, GPT partitioning.
- **Milestone 11:** Extended hardware drivers (AHCI, NVMe, xHCI, Audio).
- **Milestone 12:** System fuzzing, conformance certification, release engineering.
