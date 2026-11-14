# NovaOS Apex — Known Limitations

*Maintained continuously from Milestone 0 per Section 2 of the Build Contract.*

This document records genuine, verified technical boundaries of NovaOS Apex at each stage of development. In accordance with the non-negotiable project rules, features are never claimed without working code and verifiable test evidence.

---

## Current Status: Milestone 5 (Files, Storage & User Profiles)

### 1. CPU, Scheduling & Synchronization
- **Multicore (SMP):** Single-core execution only (Bootstrap Processor / BSP). Multi-core scheduling across APs is scheduled for Milestone 11.
- **Preemptive Multi-Priority Scheduler:** Operational with 5 priority bands (Idle, Low, Normal, High, Realtime), timer-driven preemption on IRQ 0 (1000 Hz), round-robin time-slicing, and sleep queue management.
- **Synchronization Primitives:** `SpinLock<T>` with interrupt-disabling (`cli`/`sti`) and deadlock detection (>10,000,000 spin cycles), and `KernelEvent` signaling.

### 2. Processes & Ring 3 Userspace
- **Address Space Isolation:** Distinct 4-level paging directory per process with clean kernel higher-half mappings and user memory separated above `0x4000_0000`.
- **ELF64 Executable Loader:** Freestanding 64-bit ELF parser mapping `PT_LOAD` segments, zeroing BSS, and allocating 16 KiB user stack at `0x7FFF_FFFF_0000`.
- **PID 1 Init Daemon & Shell:** Running in Ring 3 userspace (`novaos-apex-init`, `novaos-apex-shell`) with full user-mode memory isolation and hardware privilege enforcement.

### 3. System Calls & IPC
- **Native System Call Engine:** IDT Vector 128 (0x80, DPL=3) operational with strict user pointer and buffer boundary validation enforcing limits below `0x0000_8000_0000_0000`.
- **21 Supported System Calls:** `exit`, `read`, `write`, `open`, `close`, `spawn`, `wait`, `yield`, `sleep`, `get_time`, `map_memory`, `unmap_memory`, `send_ipc`, `receive_ipc`, `stat`, `readdir`, `mkdir`, `unlink`, `chdir`, `getcwd`, `seek`.
- **Inter-Process Communication:** Mailbox queues with 256-byte payload limits, message sequencing, and caller isolation.

### 4. Storage & Filesystem Architecture
- **Virtual File System (VFS):** Unified VFS layer managing mount tables, longest-prefix mount matching, path canonicalization (`.` and `..`), and permission gating.
- **In-Memory RAMFS:** Root filesystem (`/`) housing standard hierarchy (`/bin`, `/etc`, `/dev`, `/tmp`, `/var`, `/users`, `/storage`) and configuration databases (`/etc/os-release`, `/etc/hostname`, `/etc/passwd`, `/etc/group`, `/etc/motd`).
- **Device Filesystem (DevFS):** Virtual device nodes mounted under `/dev` including `/dev/null`, `/dev/zero`, `/dev/random`, `/dev/urandom`, `/dev/console`, `/dev/serial`, `/dev/vda`, and `/dev/fb0`.
- **NovaFS On-Disk Block Filesystem:** 4 KiB block architecture with atomic transaction checkpointing, superblock validation (`NOVAFS01`), direct & indirect block indices, free bitmaps, and sector persistence on VirtIO block storage.
- **User Security & Permissions:** POSIX mode bitmask validation (`rwxrwxrwx`), `Uid`/`Gid` credentials model with root bypass, owner/group/other checks, and user profile directory isolation (`/users/apex/`).

### 5. Networking Status
- VirtIO Network device driver initialized in loopback mode; full TCP/IP networking stack (Ethernet II, ARP, IPv4, UDP, TCP 3-way handshake) and BSD socket API are scheduled for **Milestone 6**.
