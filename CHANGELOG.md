# NovaOS Apex Changelog

All notable changes to NovaOS Apex are documented in this file.

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
