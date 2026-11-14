# NovaOS Apex — Architecture Manual

## Overview

NovaOS Apex is structured as a modular monolithic kernel running in x86_64 Ring 0, paired with an isolated Ring 3 userland operating system platform.

```text
+-------------------------------------------------------------+
|                     Nova Desktop Platform                   |
|  Nova Launcher | Nova Bar | Snap Engine | Native Applications|
+-------------------------------------------------------------+
|                      NovaUI Toolkit                         |
|      Widgets | Layouts | Event Loop | Drawing Primitives    |
+-------------------------------------------------------------+
|                      System Services                        |
|  init (PID 1) | session | package | network | audio | search |
+-------------------------------------------------------------+
======================= Ring 3 / Ring 0 =======================
+-------------------------------------------------------------+
|                      NovaOS Apex Kernel                     |
|  Syscall Dispatcher (int 0x80) | Capability Access Control  |
|  Priority Preemptive Scheduler | 4-Level Virtual Memory     |
|  VFS & NovaFS Block Engine     | VirtIO / Hardware Drivers  |
|  Software Compositor & Blitter | IPC Message Queues         |
+-------------------------------------------------------------+
|                    Hardware / UEFI Firmware                 |
+-------------------------------------------------------------+
```

## Core Design Principles

1. **Memory Safety & Zero Undefined Behavior:** Rust's type system and ownership semantics enforce strict resource lifetimes for kernel objects, descriptors, and buffers.
2. **Capability-Based Object Handles:** Userspace processes never interact with raw kernel pointers. All interaction is intermediated through typed 64-bit capability handles with discrete permissions (`READ`, `WRITE`, `EXEC`, `SPAWN`, `NETWORK`, `STORAGE`).
3. **Fault-Tolerant Compositing:** The desktop compositor isolates application surfaces. If an application hangs or crashes, the desktop shell and other windows continue running without graphical degradation.
4. **Crash-Resilient Storage:** NovaFS employs atomic transaction commits, superblock checkpoints, and recovery rollbacks to prevent on-disk corruption during sudden power losses.
