# NovaOS Apex — Security Model & Capability Permissions

## Privilege Hierarchy

NovaOS Apex enforces hard separation between Ring 0 (Kernel Supervisor) and Ring 3 (Userland):

- **Ring 0:** Hardware I/O ports (`in`/`out`), control registers (`CR0`, `CR3`, `CR4`), MSRs, and page table updates are strictly confined to the kernel core.
- **Ring 3:** All applications, shell commands, and background services execute in Ring 3 with page table `User/Supervisor = 1`.

## Capability Bitmask System

Every userland task owns an immutable or revokable capability bitmask:

| Flag | Constant | Description |
| :--- | :--- | :--- |
| `CAP_NONE` | `0x00` | Default unprivileged process |
| `CAP_NET` | `0x01` | Sockets, DNS lookup, raw packet transmission |
| `CAP_STORAGE` | `0x02` | Direct block I/O and arbitrary VFS file writes |
| `CAP_IPC` | `0x04` | Inter-process messaging to system daemons |
| `CAP_GRAPHICS` | `0x08` | Direct Window Server surface management |
| `CAP_AUDIO` | `0x10` | Hardware sound output stream control |
| `CAP_ADMIN` | `0x20` | Elevated microkernel configuration |

## Multi-User Identity
- Standard UID/GID mapping (`root` = 0, `nova` = 1000, `guest` = 1001).
- Discretionary Access Control (DAC) permissions: `rwxr-xr-x` octal flags on NovaFS inodes.
