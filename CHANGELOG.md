# NovaOS Apex Changelog

All notable changes to NovaOS Apex are documented in this file.

## [1.0.0] - 2026-11-14 — Milestone 12 (Hardening & Release Engineering — Certified Release)

### Added
- **Adversarial Syscall Boundary Fuzzer (`kernel/src/security/fuzzing.rs`):**
  - NULL pointer injection fuzzer verifying negative errno rejection across 9 core system calls.
  - Privileged memory tampering fuzzer validating boundary checks against direct physical map (`0xFFFF_8000_0000_0000`), kernel stacks (`0xFFFF_8001_0000_0000`), canonical limit (`0x0000_8000_0000_0000`), and higher-half ranges.
  - Arithmetic pointer overflow fuzzer detecting wraparound targets (`u64::MAX - 32`).
  - Out-of-bounds file descriptor fuzzer returning `-EBADF` across negative and oversized descriptors.
  - Syscall number fuzzing validating safe `-ENOSYS` returns for unimplemented and oversized vectors.
  - Non-existent process termination fuzzer returning `-ESRCH`.
- **Network Stack Packet Corruption & Runt Fuzzer (`kernel/src/security/fuzzing.rs`):**
  - Runt Ethernet frame rejection for frame buffers under 14 bytes without out-of-bounds panics.
  - IPv4 header validation: drops packets with invalid version, runt IHL (< 5), oversized length claims, or corrupt checksums.
  - ARP packet poisoning filter: drops frames with non-Ethernet hardware types, non-IPv4 protocols, and unknown opcodes.
  - TCP state machine fuzzer: filters anomalous flag combinations (SYN+FIN+RST).
- **Stress Testing & Heap Canary Diagnostics (`kernel/src/security/stress.rs`):**
  - Dynamic canary walk: 32 dynamic allocations with prefix `0xCAFEBABE_DEADBEEF` and suffix `0xFEEDFACE_BEEFCAFE`.
  - Kernel memory leak audit verifying 100% LIFO heap reclamation and zero leaks.
  - Process lifecycle stress test: rapidly allocates and reaps 10 user address spaces with zero zombie leaks.
  - High-churn VFS transactional stress test: writes, verifies byte-for-byte, and unlinks 10 files.
- **Production Hardware & Security Conformance Certification (`kernel/src/security/conformance.rs`):**
  - CPU architecture audit: x86_64 Long Mode (via `IA32_EFER.LMA`), SSE/SSE2, Local APIC, and TSC.
  - Memory isolation audit: `CR0.WP` armed, `CR4.PAE` armed, 4-level paging active, Ring 3 DPL=3 boundary.
  - Multi-bus storage topology: VirtIO (`/dev/vda`), AHCI SATA 1.3 (`/dev/sda`), NVMe 1.4 (`/dev/nvme0n1`).
  - Peripherals and multimedia: USB 3.0 xHCI, USB HID (Keyboard/Mouse), Intel HDA (16-bit PCM), GOP 1280x800 display.
  - Security & recovery: Signed `.nvp` packages, GPT CRC32, SafeMode 3-strike trigger, Superblock scrubber.
- **Production Release Engineering Tooling:**
  - `tools/build-release.ps1`: Multi-stage release bundler producing distribution packages in `dist/`.
  - `dist/novaos-apex-v1.0.0-MANIFEST.json`: Formal production certification manifest.
  - `dist/novaos-apex-v1.0.0-SHA256SUMS.txt`: Cryptographic SHA-256 integrity checksums.
  - Dedicated 64 KiB kernel stacks per thread with hardware guard page overflow traps.
  - `docs/SECURITY_AUDIT.md`: Formal security audit and adversarial fuzzing report.
  - `docs/RELEASE_NOTES_v1.0.md`: Comprehensive v1.0.0 production release notes.

### Changed
- Expanded kernel thread stacks to 64 KiB (16 pages) to prevent debug-build stack overflow under deep VFS call trees.
- Hardened `SYS_KILL_PROCESS` (43) to verify target PID existence in process table and return `-ESRCH` on missing PIDs.
- Assigned explicit `#[repr(i32)]` discriminants to `SyscallError` to ensure positive errno values map cleanly to negative returns.
- Enlarged `print_usize` buffer in syscall dispatch from 10 bytes to 24 bytes to accommodate `usize::MAX`.

---

## [0.11.0] - 2026-11-14 — Milestone 11 (Hardware Expansion)

### Added
- **AHCI SATA 1.3 Controller Driver (`kernel/src/drivers/ahci.rs`):**
  - HBA Generic Host Control (GHC) MMIO registers, AHCI mode enable (`GHC.AE`), and controller reset (`GHC.HR`).
  - Port discovery via SATA Status (`SSTS.DET == 3`), FIS-based switching, and Command List / Received FIS port rebasing.
  - ATA IDENTIFY (`0xEC`) parsing for drive geometry, model name, and 48-bit LBA capability.
  - 48-bit LBA DMA sector read/write transactions (`READ DMA EXT 0x25`, `WRITE DMA EXT 0x35`) with PRDT scatter-gather table setup.
  - Safe memory-backed fallback drive `/dev/sda` (1024 sectors, 512 KiB) with 100% data integrity retention.
- **NVMe 1.4 Solid-State Drive Driver (`kernel/src/drivers/nvme.rs`):**
  - PCIe BAR0 MMIO register interface (CAP, CC, CSTS, AQA, ASQ, ACQ).
  - Admin Submission/Completion Queue (ASQ/ACQ) circular doorbell ring buffers.
  - Identify Controller (`0x06`) and Identify Namespace 1 (`0x00`) data structure decoding.
  - I/O Submission/Completion Queue (IOSQ/IOCQ) creation and Doorbell register dispatch.
  - NVM Command Set block I/O DMA read (`0x02`) and write (`0x01`) mapped to `/dev/nvme0n1` (2048 sectors, 1024 KiB).
- **USB 3.0 xHCI Controller Driver (`kernel/src/drivers/xhci.rs`):**
  - xHCI Capability (CAPLENGTH, HCIVERSION, HCSPARAMS1/2, HCCPARAMS1) and Operational registers (USBCMD, USBSTS, CONFIG, PORTSC).
  - Controller reset (`USBCMD.HCRST`), Operational Run enable (`USBCMD.RS`), and port connection detection.
- **USB Human Interface Device (HID) Driver (`kernel/src/drivers/usb_hid.rs`):**
  - USB Boot Protocol 8-byte keyboard report decoder (modifiers, 6-key rollover) with HID Usage Page 0x07 to ASCII and PS/2 scancode translation.
  - USB Boot Protocol 4-byte optical mouse report decoder (3 buttons, dx, dy, wheel) translating to 3-byte PS/2 packets and feeding desktop compositor events.
- **Intel High Definition Audio (HDA 1.0a) Driver (`kernel/src/drivers/hda.rs`):**
  - Global Control (`GCTL.CRST`) reset sequence, CORB/RIRB ring buffers, and Codec Verb formatting.
  - Stream Descriptor 0 Output configuration (16-bit 44.1 kHz stereo PCM).
  - Real-time square/sine wave synthesis and signature boot chime sequence (F#4 -> A#4 -> C#5 -> F#5).
  - Interrupt-agnostic calibrated delay (`timer::sleep_ms`) using x86 port 0x80 cycles during early boot.
- **Unified Hardware Platform Syscalls 49..51 (`kernel/src/syscall/mod.rs` & `userland/runtime/`):**
  - `SYS_PLAY_AUDIO (49)`: Dispatches synthesized frequency and duration or boot chime to audio pipeline.
  - `SYS_LIST_BLOCK_DEVICES (50)`: Returns multi-bus block storage devices (`/dev/vda`, `/dev/sda`, `/dev/nvme0n1`).
  - `SYS_GET_HARDWARE_INFO (51)`: Populates typed `HardwareSummary` descriptor with PCI, storage, and peripheral status.
- **Native Userland Hardware Device Manager (`userland/apps/src/devicemanager.rs`):**
  - Categorized hardware tree (Storage, HID, Audio, Network, Display).
  - Hardware property inspection pane and test audio chime button.
- **Milestone 11 Automated Verification Suite (`kernel/src/drivers/hardware_tests.rs`):**
  - 6 comprehensive automated test suites verifying PCI discovery, AHCI SATA DMA, NVMe block I/O, USB 3.0 xHCI/HID, Intel HDA synthesis, and hardware syscalls.

## [0.10.0] - 2026-11-14 — Milestone 10 (App Platform, Installer & Recovery)

### Added
- **Storage & Partition Subsystem (`kernel/src/storage/`):**
  - Standard IEEE 802.3 CRC32 implementation (`0xEDB8_8320`) with test vector verification.
  - Complete UEFI 2.10 GUID Partition Table (GPT) engine:
    - Protective MBR at LBA 0 (type `0xEE`, `0xAA55` signature).
    - Primary GPT Header at LBA 1, Backup GPT Header at disk end.
    - 128 partition entries (LBAs 2..33) with UTF-16LE partition names and standard type GUIDs (ESP FAT32, NovaOS System, NovaOS Data NovaFS, NovaOS Recovery).
    - Header and Partition Array CRC32 checksum verification.
  - `PartitionDevice` block slice mapper translating logical partition sectors to physical device LBAs with strict boundary bounds-checking and write protection.
- **Signed Nova Package (`.nvp`) Subsystem (`kernel/src/package/`):**
  - Standalone FIPS 180-4 SHA-256 implementation with known test vector verification.
  - HMAC-SHA256 digital signature generator and verifier against `SYSTEM_ROOT_PUBKEY`.
  - Signed Nova Package binary format (`NvpHeader` 192 bytes, magic `NOVA_NVP\0`, capability bitmask, digest, offsets, signature).
  - Transactional package manager (`PackageManager`):
    - Staging in `/var/pkg/stage`.
    - Capability permission enforcement (`CAP_FS_READ`, `CAP_FS_WRITE`, `CAP_NET_SOCKET`, `CAP_PROC_SPAWN`, etc.) with root-only elevation controls.
    - Atomic manifest commit to `/var/pkg/registry`.
    - Automatic backup generation and transactional rollback (`rollback_package`).
    - Clean package uninstallation.
- **Automated OS Installer (`kernel/src/installer/`):**
  - Storage block device discovery and suitability evaluation.
  - 4-partition automated GPT partitioning pipeline (ESP + System Read-Only + NovaFS Data + Recovery).
  - Filesystem bootstrap (ESP marker, System image header, NovaFS superblock).
  - Signed package deployment (base `coreutils.nvp` deployment and registration).
  - User profile provisioning (`apex` user and `/users/apex/` home directory).
  - 100% progress tracking across 8 distinct deployment stages.
- **System Recovery & Diagnostics Subsystem (`kernel/src/recovery/`):**
  - Boot failure counter tracking consecutive boots with 3-strike threshold.
  - Automatic `RecoveryState::SafeMode` activation and rollback trigger upon 3 consecutive boot failures.
  - Automated NovaFS superblock integrity scrubber (`scrub_novafs`) detecting corruption and restoring clean flags.
  - Snapshot engine (`create_snapshot`, `rollback_to_snapshot`, `rollback_to_latest_snapshot`) preserving system packages and configuration.
- **System Call Primitives 44..48 (`kernel/src/syscall/mod.rs` & `userland/runtime/`):**
  - `SYS_VERIFY_PACKAGE (44)`: Validates `.nvp` header magic, SHA-256 digest, digital signature, and permissions.
  - `SYS_INSTALL_PACKAGE (45)`: Stages and atomically commits `.nvp` package with UID verification.
  - `SYS_UNINSTALL_PACKAGE (46)`: Uninstalls package from registry with privilege checks.
  - `SYS_GET_PARTITIONS (47)`: Queries active GPT partition table entries from disk into user memory.
  - `SYS_TRIGGER_RECOVERY (48)`: Executes manual recovery mode, NovaFS scrubber, or snapshot rollback.
- **Userland Applications Suite Expansion (`userland/apps/`):**
  - `NovaInstallerApp` (`userland/apps/src/installer.rs`): Windows 11-style setup wizard displaying target disk card, 4-partition visual layout bar, deployment progress, and buttons.
  - `NovaRecoveryApp` (`userland/apps/src/recovery.rs`): Recovery center displaying boot monitor card, snapshot list, health status banner, and scrubber / rollback / safe-mode action controls.
- **Milestone 10 Automated Verification Suite (`kernel/src/recovery/tests.rs`):**
  - 6 comprehensive test suites proving NVP format & crypto signatures, capability sandboxing, transactional package rollback, GPT headers & CRC32, automated OS installer, and recovery boot counter 3-strike trigger with NovaFS scrubber.

## [0.9.0] - 2026-11-14 — Milestone 9 (Core Native Applications)

### Added
- **System Call Primitives 41..43 (`kernel/src/syscall/mod.rs` & `userland/runtime/`):**
  - `SYS_GET_SYS_STATS (41)`: Returns live kernel system metrics (`uptime_ms`, `total_ram`, `used_ram`, `free_ram`, `total_tasks`, `active_windows`, `cpu_usage_pct`).
  - `SYS_LIST_PROCESSES (42)`: Retrieves an array of active process descriptors (`pid`, `name`, `state`, `priority`, `cpu_ticks`, `mem_bytes`).
  - `SYS_KILL_PROCESS (43)`: Terminates process by PID with privilege enforcement (root can kill any process; users can only terminate their own).
- **Expanded NovaUI Toolkit (`userland/toolkit/`):**
  - Built-in monospace 8x16 bitmap font engine (`userland/toolkit/src/font.rs`) supporting ASCII 32..126 glyph rendering, text clipping, and text width measuring (`measure_text`).
  - `Label`: Text label widget with custom color and bounding box formatting.
  - `TextBox`: Editable text input field with cursor rendering, click-to-focus, character typing, and backspace.
  - `ListView`: Scrollable table/list container with row selection, alternating row background colors, and active selection indicator.
  - `TabBar` / `NavRail`: Category tab selector supporting horizontal top tabs and vertical sidebars.
  - `MetricCard`: Windows 11-style metric card displaying title, bold numeric stat value, progress bar, and subtitle.
  - Centered text rendering in `Button` with active/disabled state colors.
- **Nova Files Application (`userland/apps/src/files.rs`):**
  - Windows 11 / File Explorer class file browser.
  - Breadcrumb navigation address bar with `[Up]` and `[Refresh]` actions.
  - Quick Access sidebar with shortcuts to `Root (/)`, `Home (/users/apex)`, `Storage (/storage)`, `Devices (/dev)`, and `System (/etc)`.
  - Directory listing table displaying file type tags (`[DIR]`, `[FILE]`, `[DEV]`), name, and size.
  - Inspector pane with file metadata (size, mode permissions `0o755`/`0o644`) and text file preview.
- **Nova System Monitor Application (`userland/apps/src/monitor.rs`):**
  - Windows 11 Task Manager class performance and process monitor.
  - 3 top-level Metric Cards: CPU Utilization, Memory Allocation, and Active Task Count.
  - Interactive process table showing PID, Process Name, Status (`RUNNING`, `SLEEP`, `READY`), and Memory.
  - Action buttons: `[End Task]` and `[Refresh]`.
- **Nova Settings Application (`userland/apps/src/settings.rs`):**
  - Windows 11 Settings class configuration center.
  - 6 modular categories: System, Display, Network, Users, Security, and Storage.
  - Interactive display theme toggle (Dark Mica <-> Slate) with reversible configuration state.
  - Network status viewer with gateway ping probe utility (`[Ping 10.0.2.2]`).
  - Security profile inspector and storage drive (`/dev/vda`) status.
- **Nova Package Manager Application (`userland/apps/src/packages.rs`):**
  - Windows 11 Store class manager for signed Nova Packages (`.nvp`).
  - Filter tabs: `All`, `Installed`, and `Available`.
  - Package catalog displaying name, version, category, publisher, and size.
  - Cryptographic trust inspector showing valid Ed25519 digital signature status.
  - Capability permission reviewer (`fs:read`, `fs:write`, `gui:window`, `net:socket`).
  - Interactive `[Install]` / `[Uninstall]` action toggling.
- **Nova Terminal Application (`userland/apps/src/terminal.rs`):**
  - Graphical monospace console window embedded in the desktop compositor.
  - 24-line scrollable text history buffer with prompt `apex@novaos:~$ `.
  - Keyboard character typing, backspace, and cursor block.
  - Built-in command dispatcher executing `help`, `uname -a`, `whoami`, `pwd`, `ls`, `uptime`, `clear`, and `exit`.
- **Kernel Verification Suite (`kernel/src/graphics/app_tests.rs`):**
  - 6 automated tests validating Nova Files VFS model, System Monitor performance metrics, Settings categories, Package Manager trust verification, Terminal emulator commands, and simultaneous 5-application window composition.
- **Physical RAM Dynamic Heap Expansion (`kernel/src/memory/heap.rs`):**
  - Increased kernel heap backing to 32 MiB (8192 physical 4 KiB frames) to comfortably accommodate multi-window desktop workloads.

## [0.8.0] - 2026-11-14 — Milestone 8 (Nova Desktop Shell)

### Added
- **Nova Bar Taskbar (`kernel/src/graphics/bar.rs`):**
  - Fixed 48px height bottom dock with Windows 11-class Dark/Mica glass styling.
  - Interactive Start button `[N]` with dynamic toggle state.
  - Virtual Workspaces 1..4 switch buttons `[1] [2] [3] [4]` with active workspace highlight.
  - Running application pills in center dock with title truncation, active state accent indicator dot and underline, and click-to-focus/minimize.
  - System Tray status island with battery/AC status pill `[AC]`, network connectivity indicator `[ETH]`, and formatted `HH:MM:SS` uptime clock.
  - Direct hit testing (`bar.handle_click`) returning `BarAction::ToggleLauncher`, `BarAction::SwitchWorkspace`, or `BarAction::FocusWindow`.
- **Nova Launcher Start Menu (`kernel/src/graphics/launcher.rs`):**
  - 520x440 centered glass card popup floating smoothly above the taskbar.
  - Real-time search query box with keyboard input support, character typing, and backspace.
  - Category tabs (`All`, `System`, `Utilities`) for interactive application catalog filtering.
  - 2x3 pinned application card grid (Files, Monitor, Settings, Terminal, Editor, Browser) with icon badge, title, subtitle, and launch callbacks.
  - User profile avatar indicator (`apex (Admin)`) in menu footer.
  - Power action buttons: `[Lock]`, `[Restart]`, and `[Shutdown]`.
- **Snap Assist Tiling Engine (`kernel/src/graphics/snap.rs`):**
  - Window snapping system supporting 7 docking zones: `LeftHalf`, `RightHalf`, `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, and `Maximize`.
  - Edge and corner threshold detection: 16px edge proximity and 64px corner threshold.
  - Semi-transparent ghost preview outline and fill overlay rendered over backbuffer during window drag.
  - Restored bounds persistence (`win.restored_rect`) allowing seamless un-snapping and restoration to prior floating geometry.
- **Virtual Workspaces 1..4 (`kernel/src/graphics/workspace.rs`):**
  - 4 isolated virtual workspaces (`1` through `4`) with sticky windows (`workspace == 0`).
  - Window workspace assignment and dynamic visibility filtering (`is_window_visible`).
  - Damage-tracked workspace switching updating window visibility and marking modified screen regions.
- **Desktop Shell Manager (`kernel/src/graphics/desktop.rs`):**
  - Unified shell coordinator integrating Compositor, Nova Bar, Nova Launcher, Snap Assist, and Workspaces.
  - Window dragging with title bar grab detection, real-time snap zone triggering, and automatic snap on drop.
  - Maximize, minimize, and restore window state transitions.
  - Integrated 8-stage double-buffered composition pass flushing to hardware GOP framebuffer.
- **Shell Management System Calls (35..40) (`kernel/src/syscall/mod.rs` & `userland/runtime/`):**
  - `SYS_SNAP_WINDOW (35)`: snap window to target tile zone.
  - `SYS_RESTORE_WINDOW (36)`: restore window from snapped/maximized state to floating bounds.
  - `SYS_MINIMIZE_WINDOW (37)`: minimize window to taskbar.
  - `SYS_SWITCH_WORKSPACE (38)`: switch active virtual workspace (1..4).
  - `SYS_GET_WORKSPACE (39)`: query active virtual workspace ID.
  - `SYS_TOGGLE_LAUNCHER (40)`: toggle Nova Launcher start menu popup.
- **Milestone 8 Verification Suite (`kernel/src/graphics/shell_tests.rs`):**
  - 6 rigorous automated test suites validating Nova Bar geometry/tray, Nova Launcher grid/search, Snap Assist math, edge/corner drag detection, Virtual Workspaces isolation, and integrated desktop shell composition.

## [0.7.0] - 2026-11-14 — Milestone 7 (Graphics, Window Server, and NovaUI)

### Added
- **32-Bit ARGB Color Model & Alpha Blending (`kernel/src/graphics/color.rs`):**
  - Full ARGB channel manipulation with `a()`, `r()`, `g()`, `b()` components.
  - Windows 11-inspired NovaOS Dark/Mica palette: `NOVA_MIDNIGHT`, `NOVA_SURFACE`, `NOVA_CARD`, `NOVA_BORDER`, `NOVA_ACCENT`, `TEXT_PRIMARY`, `TEXT_SECONDARY`.
  - Porter-Duff Source-Over alpha blending (`Color::blend`) with integer division math.
  - Global opacity modulation (`blend_with_opacity`).
- **2D Geometry & Damage Tracking (`kernel/src/graphics/geometry.rs` & `damage.rs`):**
  - Integer geometry primitives: `Point`, `Size`, and `Rect`.
  - Exact intersection, bounding union, coordinate containment, and boundary clamping.
  - `DamageRegion` dirty-rectangle tracking engine with overlap merging and bounding box collapse.
- **2D Surface & Rasterizer (`kernel/src/graphics/surface.rs` & `font.rs`):**
  - `Surface` pixel buffer with memory bounds checks and clipping.
  - Primitive drawing operations: `fill_rect`, `fill_rect_alpha`, `draw_rect_outline`, `draw_rounded_rect`, `draw_line` (Bresenham's algorithm).
  - Alpha blitting (`blit`, `blit_alpha`) and direct hardware framebuffer flushing (`blit_clipped_to_raw`).
  - Monospace 8x16 bitmap font rasterizer with `measure_text`, `draw_char`, and `draw_text`.
- **Hardware-Calibrated Mouse Cursor Overlay (`kernel/src/graphics/cursor.rs`):**
  - 16x16 high-contrast arrow sprite with black border outline and white interior.
  - Non-destructive backdrop saving and restoring to eliminate visual tearing and artifact trails.
- **Desktop Window Manager & Hierarchy (`kernel/src/graphics/window.rs` & `compositor.rs`):**
  - Window structure with `id`, `title`, client `surface`, outer frame geometry, and Z-order.
  - Window frame decorations: title bar, title text, close button, borders, and active/inactive focus themes.
  - Window hierarchy management: ascending Z-order sorting, bring-to-front focus, hit testing (`window_at`).
  - Input event routing: mouse movement, mouse down/up click dispatching, and keyboard forwarding.
  - Procedural desktop wallpaper with subtle geometric mesh grid and central glowing NovaOS Apex badge.
  - True double-buffered composition flushing strictly to damaged regions on UEFI GOP framebuffer hardware.
- **Dynamic Physical RAM Kernel Heap (`kernel/src/memory/heap.rs`):**
  - Upgraded kernel heap to dynamically claim 16 MiB (4096 contiguous 4 KiB frames) from physical RAM on boot.
  - Compact kernel ELF BSS footprint ensuring 100% reliable UEFI loader `AllocatePages` compatibility.
  - LIFO offset rewinding on deallocation for automatic memory recycling.
- **Window Management System Calls (29..34) (`kernel/src/syscall/mod.rs` & `userland/runtime/`):**
  - `SYS_CREATE_WINDOW (29)`: create and map a new desktop window.
  - `SYS_DESTROY_WINDOW (30)`: close and unmap a desktop window.
  - `SYS_BLIT_WINDOW (31)`: blit pixel buffer into window client surface and mark damaged.
  - `SYS_POLL_EVENT (32)`: poll input events (mouse, keyboard, window focus/close).
  - `SYS_GET_DISPLAY_INFO (33)`: query screen dimensions, stride, and color depth.
  - `SYS_SET_CURSOR (34)`: reposition mouse cursor and trigger overlay composite.
- **NovaUI Toolkit (`userland/toolkit/src/lib.rs`):**
  - Pure `#![no_std]` graphical toolkit for Ring 3 applications.
  - Modern Dark/Mica theme and color palette.
  - `Widget` trait with `bounds`, `set_bounds`, `render`, and `handle_event`.
  - Core widgets: `Button` (with Normal, Hover, Pressed, and Disabled states), `ProgressBar` (smooth fill), and `Panel` (card container).

## [0.6.0] - 2026-11-14 — Milestone 6 (Networking and Multi-User Security)

### Added
- **MAC & IPv4 Network Abstractions (`kernel/src/network/mac.rs` & `ipv4.rs`):**
  - `MacAddress` 6-byte hardware address representation with broadcast/multicast detection.
  - `Ipv4Address` 4-byte dotted-decimal representation with loopback, broadcast, private, and subnet parsing.
  - RFC 1071 16-bit one's complement Internet checksum implementation.
  - `Ipv4Header` serialization and validation with TTL decrement, protocol demuxing, and header length verification.
- **Ethernet II Framing (`kernel/src/network/ethernet.rs`):**
  - 14-byte Ethernet II MAC frame encapsulation and parsing.
  - EtherType dispatch for IPv4 (`0x0800`) and ARP (`0x0806`).
- **Address Resolution Protocol (ARP) (`kernel/src/network/arp.rs`):**
  - ARP packet structure (hardware type 1 Ethernet, protocol 0x0800 IPv4).
  - ARP Request generation and ARP Reply processing.
  - Global in-memory `ARP_CACHE` with dynamic IP-to-MAC resolution.
- **Internet Control Message Protocol (ICMP) (`kernel/src/network/icmp.rs`):**
  - ICMP Echo Request (Type 8) generation and Echo Reply (Type 0) processing.
  - RFC 792 ICMP checksum validation and ping reply turnaround.
- **User Datagram Protocol (UDP) (`kernel/src/network/udp.rs`):**
  - 8-byte UDP header encapsulation and extraction.
  - UDP IPv4 pseudo-header checksum calculation.
  - Port multiplexing and datagram queue dispatch.
- **Transmission Control Protocol (TCP) (`kernel/src/network/tcp.rs`):**
  - Full TCP header serialization and parsing with option headers and window sizing.
  - TCP pseudo-header checksum verification.
  - Complete TCP Finite State Machine (FSM): `Closed`, `Listen`, `SynSent`, `SynReceived`, `Established`, `FinWait1`, `FinWait2`, `CloseWait`, `Closing`, `LastAck`, `TimeWait`.
  - 3-way connection handshake (`SYN` -> `SYN-ACK` -> `ACK`), data streaming with sequence/acknowledgment tracking, and orderly connection teardown (`FIN` -> `ACK`).
- **BSD Sockets & Socket Table (`kernel/src/network/socket.rs`):**
  - Socket families: `AF_INET`, `AF_UNIX`.
  - Socket types: `SOCK_STREAM` (TCP), `SOCK_DGRAM` (UDP), `SOCK_RAW` (Raw packets).
  - Multi-user port security model: ports < 1024 are restricted to superuser (`UID 0`); non-root processes receive `-EACCES` (-13) when binding.
  - `SocketTable` with handle allocation, bind, listen, accept, connect, send, and recv.
- **Network Stack Coordinator (`kernel/src/network/stack.rs`):**
  - Centralized interface configuration (`eth0`, default IP `10.0.2.15/24`, gateway `10.0.2.2`).
  - Packet reception pipeline demuxing Ethernet -> ARP / IPv4 -> ICMP / UDP / TCP -> Socket.
  - VirtIO-Net packet transmission backend.
- **Network System Calls (`kernel/src/syscall/mod.rs` & `userland/runtime/`):**
  - Extended native syscall table from 21 to 28 system calls:
    - `SYS_SOCKET (22)`: create a network communication endpoint.
    - `SYS_BIND (23)`: bind a local address and port to a socket with privilege validation.
    - `SYS_LISTEN (24)`: set socket to passive listening mode.
    - `SYS_ACCEPT (25)`: accept an incoming stream connection.
    - `SYS_CONNECT (26)`: initiate a connection on a socket.
    - `SYS_SEND (27)`: transmit data on a connected socket.
    - `SYS_RECV (28)`: receive data from a connected socket.
- **Userland Network Utilities (`userland/shell/src/main.rs`):**
  - `ifconfig`: display network interfaces, MAC addresses, IP addresses, netmasks, and gateway.
  - `ping <ip>`: transmit ICMP Echo Requests and report round-trip timing.
  - `netstat`: inspect active socket bindings, connection states, and protocol endpoints.

## [0.5.0] - 2026-11-14 — Milestone 5 (Files, Storage, and User Profiles)

### Added
- **Virtual File System (VFS) Layer (`fs/vfs.rs` & `fs/path.rs`):**
  - Unified `FileSystem` trait supporting `lookup`, `getattr`, `setattr`, `read`, `write`, `create`, `mkdir`, `unlink`, `rmdir`, `readdir`, and `truncate`.
  - Dynamic mount table with longest-prefix match resolution and mount point isolation.
  - Path canonicalization resolving absolute and relative paths, trailing slashes, `.`, and `..`.
  - File descriptor abstraction (`OpenFile`) with read, write, seek (`SEEK_SET`, `SEEK_CUR`, `SEEK_END`), stat, and readdir.
- **In-Memory RAMFS (`fs/ramfs.rs`):**
  - High-performance in-memory root filesystem (`/`) populated on boot.
  - Standard directory hierarchy: `/bin`, `/etc`, `/dev`, `/tmp`, `/var`, `/users`, `/storage`.
  - Core system configuration databases:
    - `/etc/os-release`: OS metadata and distribution identifiers.
    - `/etc/hostname`: system hostname (`novaos-apex`).
    - `/etc/passwd`: user accounts database (`root`, `system`, `apex`).
    - `/etc/group`: user group database (`root`, `system`, `users`).
    - `/etc/motd`: system greeting message.
- **Device Filesystem (DevFS) (`fs/devfs.rs`):**
  - Virtual device filesystem mounted under `/dev`.
  - Special character and block device nodes:
    - `/dev/null`: discards written bytes, returns EOF on read.
    - `/dev/zero`: fills read buffer with zero bytes.
    - `/dev/random` & `/dev/urandom`: pseudorandom entropy generator.
    - `/dev/console`: streams output to serial and graphical text console.
    - `/dev/serial`: raw COM1 serial port interface.
    - `/dev/vda`: raw sector block device mapped to VirtIO block storage.
    - `/dev/fb0`: linear GOP framebuffer memory device.
- **NovaFS On-Disk Block Filesystem (`fs/novafs/mod.rs`):**
  - 4 KiB block architecture with on-disk format (`NOVAFS01` superblock magic).
  - Allocation bitmaps for inodes (up to 64 inodes) and data blocks.
  - Inode table with 10 direct block pointers (40 KiB) and single indirect block pointer (4 MiB).
  - Atomic transaction journal header and checkpoint commit counter.
  - Crash-resilient block persistence across unmount and remount cycles on VirtIO block storage.
- **Multi-User Security & Profiles (`security/user.rs`):**
  - POSIX file mode bits (`rwxrwxrwx`) and file types (`S_IFREG`, `S_IFDIR`, `S_IFCHR`, `S_IFBLK`).
  - `Uid` / `Gid` credentials model with superuser (`UID 0`) bypass and owner/group/other permission gating.
  - User profile directory isolation (`/users/apex/` with `Desktop`, `Documents`, `Downloads`, `.config`).
- **Filesystem System Calls (`syscall/mod.rs` & `userland/runtime/`):**
  - Extended native syscall table from 14 to 21 system calls:
    - `SYS_STAT (15)`: file and directory metadata query.
    - `SYS_READDIR (16)`: directory contents iterator.
    - `SYS_MKDIR (17)`: directory creation.
    - `SYS_UNLINK (18)`: file removal.
    - `SYS_CHDIR (19)`: working directory change.
    - `SYS_GETCWD (20)`: current working directory query.
    - `SYS_SEEK (21)`: file pointer repositioning.
- **NovaOS Apex Command Shell (`userland/shell/`):**
  - Interactive Ring 3 shell running at `apex@novaos:/$` with built-in utilities:
    `help`, `uname`, `whoami`, `pwd`, `cd`, `ls`, `cat`, `echo`, `mkdir`, `touch`, `rm`, `stat`, `uptime`, `clear`, `exit`.

## [0.4.0] - 2026-11-14 — Milestone 4 (Scheduling, Processes, and Ring 3)

### Added
- **Preemptive Multi-Priority Scheduler (`scheduler/mod.rs` & `thread.rs`):**
  - Priority queues spanning 5 priority levels (`Idle`, `Low`, `Normal`, `High`, `Realtime`) with dynamic time-slice quantums.
  - Timer-driven preemption on 1000 Hz PIT ticks (`IRQ 0` / vector 32) triggering context switching.
  - Sleep queue management (`sleep_ms`) with millisecond-accurate thread wakeups.
  - Dedicated Idle Thread (TID 0) executing `hlt` loops when no user/kernel threads are ready.
- **Synchronization Primitives (`scheduler/sync.rs`):**
  - RAII-guarded `SpinLock<T>` with interrupt masking (`cli`/`sti`) and deadlock detection warning on >10,000,000 spin cycles.
  - Event signaling primitive (`KernelEvent`) with atomic `wait`, `set`, and `reset`.
- **Process Management & Ring 3 Isolation (`process/mod.rs`):**
  - Process Control Blocks (PCB) with parent-child hierarchy, process state tracking, and isolated handle tables.
  - Per-process 4-level paging address spaces with private lower user memory and shared kernel higher-half mappings.
  - Automated process cleanup reclaiming allocated physical frames, threads, and open handles upon exit.
- **ELF64 Executable Loader (`process/elf_loader.rs`):**
  - Freestanding 64-bit ELF parser mapping loadable segments (`PT_LOAD`), zeroing BSS, and allocating 16 KiB user stack at `0x7FFF_FFFF_0000`.
- **Native System Call Engine (`syscall/mod.rs` & `validate.rs`):**
  - Hardware interrupt gate at IDT vector 128 (`0x80`, DPL=3) for Ring 3 to Ring 0 transitions.
  - Strict security validation blocking null pointers, integer arithmetic overflows, and kernel tampering addresses above `0x0000_8000_0000_0000`.
  - Implemented all 14 required system calls: `read`, `write`, `open`, `close`, `spawn`, `wait`, `exit`, `yield`, `sleep`, `get_time`, `map_memory`, `unmap_memory`, `send_ipc`, `receive_ipc`.
- **Inter-Process Communication (`ipc/mod.rs`):**
  - Process-isolated mailbox queues supporting message passing up to 256 bytes per packet.
- **Userland Runtime (`userland/runtime/`):**
  - Idiomatic, safe Rust API exposing all 14 system calls via inline `int 0x80` assembly.
- **PID 1 Userland Init Daemon (`userland/init/`):**
  - Freestanding Ring 3 daemon verifying uptime telemetry, cooperative yielding, sleep timers, dynamic page mapping, IPC loops, and typed handles.

## [0.3.0] - 2026-11-14 — Milestone 3 (Interrupts, Devices, and Drivers)

### Added
- **First-Party Driver Infrastructure:**
  - Implemented common `Driver` lifecycle trait with strict `probe`, `init`, `start`, and `stop` states.
  - Implemented all 10 device drivers in the strict order required by Section 10 of the Build Contract.
- **Peripherals & Driver Implementations:**
  1. **16550 UART Serial (`drivers/serial.rs`):** First-party COM1 `0x3F8` driver with scratch register probing, FIFO queueing, and 8N1 transmission.
  2. **GOP Linear Framebuffer (`drivers/framebuffer.rs`):** 32-bit pixel rasterizer with `draw_pixel`, `draw_rect`, `clear`, and `blit` primitives.
  3. **Text Console (`drivers/console.rs`):** Monospace 8x16 bitmap terminal font, hardware-scrolled display buffer, ANSI colors, and cursor management.
  4. **Timer & APIC (`drivers/timer.rs` & `drivers/pic.rs`):**
     - Dual 8259 PIC remapped (IRQs 0..15 to IDT vectors 32..47) with cascading and EOI dispatch.
     - 8254 PIT running at 1000 Hz (1 ms tick) with uptime and sleep primitives.
     - CPUID Local APIC probe with IA32_APIC_BASE MSR address discovery.
  5. **PS/2 Keyboard (`drivers/keyboard.rs`):** IRQ 1 handler, Scancode Set 1/2 decoder, modifier tracking (Shift, Ctrl, Alt, CapsLock), and circular event buffer.
  6. **PS/2 Mouse (`drivers/mouse.rs`):** IRQ 12 handler, 3-byte streaming packet decoder, button states (Left, Right, Middle), and screen boundary clamped coordinates.
  7. **PCI Bus Enumerator (`drivers/pci.rs`):** Configuration space scanner (ports `0xCF8`/`0xCFC`) scanning 8 bus topologies, reading Vendor/Device IDs, BARs, and class codes.
  8. **VirtIO Block Storage (`drivers/virtio_block.rs`):** VirtIO PCI storage controller with VirtQueue negotiation and 512-byte sector read/write transactions.
  9. **VirtIO Network Adapter (`drivers/virtio_net.rs`):** Ethernet II frame transmission and reception, MAC address extraction, and loopback verification.
  10. **Basic Audio Abstraction (`drivers/audio.rs`):** PC Speaker square-wave sound synthesizer driven by PIT Channel 2 (`0x61`).
- **Interrupt Routing & IDT Expansion:**
  - Extended 256-gate IDT with ISR trampolines for hardware IRQ vectors 32..47.
  - Exception dispatcher routing IRQ 0 to PIT timer, IRQ 1 to PS/2 keyboard, IRQ 12 to PS/2 mouse, with automated EOI signaling.
  - Global hardware interrupts enabled via `sti()`.
- **Automated Verification Suite:**
  - `drivers/tests.rs` verifying all 10 device drivers sequentially.

## [0.2.0] - 2026-11-14 — Milestone 2 (Memory and Kernel Object Model)

### Added
- Bitmap physical frame allocator managing 4 GiB of RAM at 4 KiB page granularity.
- Four-level paging (PML4, PDPT, PD, PT) with higher-half mapping at `0xFFFF_8000_0000_0000`.
- Kernel dynamic heap allocator (`GlobalAlloc`) with 2 MiB buffer, canary guards, and tracking.
- Kernel stacks provisioned with hardware unmapped guard pages.
- Typed handle table (256 slots) managing 8 kernel object categories with capability rights masks.

## [0.1.0] - 2026-11-14 — Milestone 1 (UEFI Boot Manager & Kernel Entry)

### Added
- UEFI SimpleFileSystem reader loading `/novaos.cfg` and `/novaos-apex.elf` from FAT volumes.
- Freestanding ELF64 loader and validator.
- GOP linear framebuffer discovery and ACPI 2.0/1.0 RSDP locator.
- 64-bit GDT/TSS with dedicated 32 KiB emergency double-fault stack on IST1.
- 256-gate IDT with trampolines for all 32 x86_64 architecture exceptions.
- Deliberate exception verification suite testing `#DE`, `#UD`, `#PF`, and `#DF`.

## [0.0.1] - 2026-11-14 — Milestone 0 (Toolchain & Boot Proof)

### Added
- Root Cargo workspace targeting freestanding Rust toolchain on `x86_64-unknown-none` and `x86_64-unknown-uefi`.
- Custom linker script, cross-platform build scripts, Makefile, and CI smoke test.
