# NovaOS Known Limitations & Architectural Boundaries

**Document ID:** NOVA-LIMIT-001  
**Version:** 0.1.0  
**Status:** Approved  

---

## 1. Product Scope & Truth in Advertising

NovaOS is explicitly designed as a **hybrid desktop environment application** running on top of Windows, macOS, or Linux. It is **not** an operating system kernel, firmware, bootloader, or hardware virtualization hypervisor.

To maintain integrity and transparency with users and developers, the following limitations are explicitly documented:

---

## 2. Host Operating System Boundaries

1. **No Kernel or Hardware Driver Access**: NovaOS cannot install hardware device drivers, modify UEFI/BIOS parameters, control power management beyond what the host OS exposes to user-space applications, or format physical disk partitions.
2. **No Global Window Manager Interception**: NovaOS manages virtual windows within its own application canvas. It cannot tile or intercept external host windows (e.g. native Microsoft Word or Google Chrome windows running on Windows/macOS/Linux).
3. **No Unrestricted Host Filesystem Access**: By design, NovaOS cannot read or modify arbitrary host directories (such as `C:\Windows\System32`, `C:\Program Files`, `/etc`, or `/System`) unless the host user explicitly selects and authorizes a folder via the native operating system file picker.
4. **Terminal Shell Sandboxing**: Terminal sessions execute using the host's native shell binary (`powershell.exe`, `cmd.exe`, `/bin/bash`, `/bin/zsh`). While NovaOS confines working directories and intercepts command invocation parameters, host shell commands are still subject to the user's host OS permissions. NovaOS does not provide kernel-level cgroups/jail isolation unless running inside a dedicated host container.
5. **Display Server Integration**: Multi-monitor displays are handled through Electron's `screen` API. High-DPI scaling is handled via Chromium's device pixel ratio. NovaOS does not replace Windows DWM, macOS Quartz Compositor, or Linux Wayland/X11.

---

## 3. Storage & Virtualization Boundaries

1. **Virtual Filesystem (*Nova Home*)**: Files saved to `nova://documents` or other virtual paths are persisted locally within an isolated application data directory managed by NovaOS. They are not visible to external host applications unless explicitly exported.
2. **Software Center**: Applications in the Nova Software Center are curated web/JavaScript components conforming to the `AppManifest` specification. NovaOS does not install native `.exe`, `.msi`, `.dmg`, `.pkg`, or `.deb` system packages.

