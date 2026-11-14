# NovaOS Apex — Known Limitations

*Maintained continuously from Milestone 0 per Section 2 of the Build Contract.*

This document records genuine, verified technical boundaries of NovaOS Apex at each stage of development. In accordance with the non-negotiable project rules, features are never claimed without working code and verifiable test evidence.

---

## Current Status: Milestone 6 (Networking & Multi-User Security)

### 1. CPU, Scheduling & Synchronization
- **Multicore (SMP):** Single-core execution only (Bootstrap Processor / BSP). Multi-core scheduling across APs is scheduled for Milestone 11.
- **Preemptive Multi-Priority Scheduler:** Operational with 5 priority bands (Idle, Low, Normal, High, Realtime), timer-driven preemption on IRQ 0 (1000 Hz), round-robin time-slicing, and sleep queue management.
- **Synchronization Primitives:** `SpinLock<T>` with interrupt-disabling (`cli`/`sti`) and deadlock detection (>10,000,000 spin cycles), and `KernelEvent` signaling.

### 2. Processes & Ring 3 Userspace
- **Address Space Isolation:** Distinct 4-level paging directory per process with clean kernel higher-half mappings, user memory separated above `0x4000_0000`, and user CR3 kernel transition mapping covering lower 1 GiB, GOP framebuffer (2 GiB..3 GiB), and MMIO (3 GiB..4 GiB).
- **ELF64 Executable Loader:** Freestanding 64-bit ELF parser mapping `PT_LOAD` segments, zeroing BSS, and allocating 16 KiB user stack at `0x7FFF_FFFF_0000`.
- **PID 1 Init Daemon & Shell:** Running in Ring 3 userspace (`novaos-apex-init`, `novaos-apex-shell`) with full user-mode memory isolation and hardware privilege enforcement.

### 3. System Calls & IPC
- **Native System Call Engine:** IDT Vector 128 (0x80, DPL=3) operational with strict user pointer and buffer boundary validation enforcing limits below `0x0000_8000_0000_0000`.
- **28 Supported System Calls:** `exit`, `read`, `write`, `open`, `close`, `spawn`, `wait`, `yield`, `sleep`, `get_time`, `map_memory`, `unmap_memory`, `send_ipc`, `receive_ipc`, `stat`, `readdir`, `mkdir`, `unlink`, `chdir`, `getcwd`, `seek`, `socket`, `bind`, `listen`, `accept`, `connect`, `send`, `recv`.
- **Inter-Process Communication:** Mailbox queues with 256-byte payload limits, message sequencing, and caller isolation.

### 4. Storage & Filesystem Architecture
- **Virtual File System (VFS):** Unified VFS layer managing mount tables, longest-prefix mount matching, path canonicalization (`.` and `..`), and permission gating.
- **In-Memory RAMFS:** Root filesystem (`/`) housing standard hierarchy (`/bin`, `/etc`, `/dev`, `/tmp`, `/var`, `/users`, `/storage`) and configuration databases (`/etc/os-release`, `/etc/hostname`, `/etc/passwd`, `/etc/group`, `/etc/motd`).
- **Device Filesystem (DevFS):** Virtual device nodes mounted under `/dev` including `/dev/null`, `/dev/zero`, `/dev/random`, `/dev/urandom`, `/dev/console`, `/dev/serial`, `/dev/vda`, and `/dev/fb0`.
- **NovaFS On-Disk Block Filesystem:** 4 KiB block architecture with atomic transaction checkpointing, superblock validation (`NOVAFS01`), direct & indirect block indices, free bitmaps, and sector persistence on VirtIO block storage.
- **User Security & Permissions:** POSIX mode bitmask validation (`rwxrwxrwx`), `Uid`/`Gid` credentials model with root bypass, owner/group/other checks, and user profile directory isolation (`/users/apex/`).

### 5. Networking Subsystem
- **Network Interface:** `eth0` configured with IPv4 address `10.0.2.15/24` and gateway `10.0.2.2`.
- **Protocols Supported:**
  - Ethernet II 14-byte MAC framing (`0x0800` IPv4, `0x0806` ARP).
  - ARP request generation, reply processing, and in-memory dynamic ARP cache.
  - IPv4 packet parsing, generation, and RFC 1071 16-bit Internet checksum calculation.
  - ICMP Echo Request / Echo Reply ping handling.
  - UDP datagram multiplexing and checksum validation.
  - TCP transmission control with full 3-way handshake (`SYN` -> `SYN-ACK` -> `ACK`), connection state machine (`Listen`, `SynSent`, `SynReceived`, `Established`, `CloseWait`), payload buffering, and connection teardown.
- **BSD Socket Table:** Kernel socket abstraction supporting `SOCK_STREAM`, `SOCK_DGRAM`, and `SOCK_RAW`.
- **Multi-User Port Security:** Privileged ports (< 1024) are strictly restricted to root (`UID 0`); regular users (`UID >= 1000`) cannot bind privileged ports (returns `-EACCES`).
- **Network Utilities:** Userland shell commands `ifconfig`, `ping`, and `netstat` implemented.

### 6. Graphics & Desktop GUI Status
- Hardware text console (8x16 font) rendering on GOP linear framebuffer at 1280x800.
- High-level 2D software compositor, alpha blending, window hierarchy, dirty rect tracking, and NovaUI widgets are scheduled for **Milestone 7**.
