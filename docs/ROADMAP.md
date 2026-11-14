# NovaOS Apex — 12-Milestone Development Roadmap

| Milestone | Title | Focus Area | Status |
| :--- | :--- | :--- | :--- |
| **M0** | **Toolchain & Boot Proof** | Pinned Rust toolchain, UEFI target, image builder, serial logger, QEMU boot test, CI | **Completed** |
| **M1** | **UEFI Boot Manager & Kernel Entry** | Memory map, GOP framebuffer discovery, ACPI RSDP, GDT, TSS, IDT, exceptions | **Completed** |
| **M2** | **Memory & Object Model** | 4-level paging, physical frame allocator, heap manager, capability handles | **Completed** |
| **M3** | **Interrupts, Devices & Drivers** | PIT 8254 timer, 8259 PIC, PS/2 keyboard/mouse, PCI enumerator, VirtIO block | **Completed** |
| **M4** | **Scheduling, Ring 3 & Syscalls** | Priority scheduler, context switching, Ring 3 isolation, int 0x80 gate (14 syscalls), PID 1 | **Completed** |
| **M5** | **Files, Storage & User Profiles** | VFS abstraction, persistent NovaFS block storage, user directories, shell | **Completed** |
| **M6** | **Networking & Multi-User Security** | VirtIO-Net, Ethernet II, ARP, IPv4, UDP, TCP 3-way handshake, socket table, multi-user | **Completed** |
| **M7** | **Graphics, Window Server & NovaUI** | 2D software compositor, alpha blending, window hierarchy, Z-order, widgets | **Completed** |
| **M8** | **Nova Desktop Shell** | Nova Bar taskbar, Nova Launcher, Snap Assist engine, Virtual Workspaces 1..4 | **Completed** |
| **M9** | **Core Native Applications** | Nova Files, Nova System Monitor, Nova Settings, Package Manager | **Completed** |
| **M10** | **App Platform, Installer & Recovery** | Signed .nvp packages, GPT disk partitioner, automated OS installer, recovery rollback | **Completed** |
| **M11** | **Hardware Expansion** | AHCI SATA 1.3, NVMe 1.4, USB 3.0 xHCI & HID, Intel HD Audio | **Completed** |
| **M12** | **Hardening & Release Engineering** | Syscall & network fuzzing, hardware conformance certification, release ISO | **Completed** |
| **M13** | **Windows 11 Desktop & 105 Gates** | 105-feature verification across Desktop, Taskbar, Explorer, Settings, Diagnostics, Apps & Kernel | **Completed** |
