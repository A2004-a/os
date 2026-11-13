# NovaOS Engineering Roadmap

NovaOS is built in disciplined, verified milestones. Capabilities are marked active only after emulator test verification.

| Milestone | Deliverable | Status | Verification Criteria |
|---|---|---|---|
| **M0** | Reproducible QEMU + OVMF toolchain | Complete | `cargo build`, `tools/run-qemu.ps1` operational |
| **M1** | UEFI loader and kernel serial output | Complete | `uefi-loader` boots, writes banner to COM1 |
| **M2** | GDT, IDT, exception diagnostics | Complete | Division error, page fault, double fault IST stack verified |
| **M3** | Physical memory, paging, heap | Complete | 4-level paging, UEFI frame allocator, 256 KiB dynamic heap |
| **M4** | Keyboard, framebuffer, timer, PCI | Complete | PS/2 input, GOP RGB rendering, PIT ticks, PCI bus scan |
| **M5** | Cooperative then preemptive scheduler | Active | Round-robin scheduler, Task/TCB structures, timer time-slicing |
| **M6** | Ring 3, syscalls, ELF processes | In Progress | Syscall dispatcher table (`SYS_WRITE`, `SYS_READ`, etc.), PCB |
| **M7** | RAM FS, FAT, virtio storage | In Progress | In-memory RamFS implemented; VirtIO block driver scheduled |
| **M8** | Shell and core command-line userland | In Progress | Text console shell and userland CLI stubs initialized |
| **M9** | Networking | Scheduled | VirtIO-net driver, ARP, IPv4, TCP/UDP sockets |
| **M10** | Window server and graphical desktop | Scheduled | 2D software compositor, input event routing, window manager |
| **M11** | Package system, app model, security | Scheduled | Signed capability manifests, sandboxed process isolation |
| **M12** | Hardware validation & boot media | Scheduled | Physical UEFI PC testing, bootable ISO/USB image generation |
