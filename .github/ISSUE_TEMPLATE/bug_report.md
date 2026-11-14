---
name: Bug Report
about: Create a report to help us improve NovaOS Apex
title: '[BUG] '
labels: 'bug'
assignees: ''
---

**Describe the Bug**
A clear and concise description of what the bug is.

**Subsystem Affected**
- [ ] Bootloader / UEFI
- [ ] Kernel Core / Memory
- [ ] Hardware Drivers (AHCI, NVMe, USB, Audio, GOP)
- [ ] File Systems / VFS / NovaFS
- [ ] Networking Stack
- [ ] Graphics / Compositor / Window Server
- [ ] Desktop Shell / NovaUI
- [ ] Userland Applications

**To Reproduce**
Steps to reproduce the behavior:
1. Boot NovaOS Apex in QEMU with command: `...`
2. Execute step: `...`
3. See error.

**Expected Behavior**
A clear and concise description of what you expected to happen.

**QEMU Serial / Screen Output**
```text
Paste UART serial log output here.
```

**Environment Information**
- Host OS: Windows / Linux / macOS
- QEMU Version: `qemu-system-x86_64 --version`
- Rust Toolchain: `rustc --version`
- NovaOS Version: v1.0.0
