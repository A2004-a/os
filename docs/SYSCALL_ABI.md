# NovaOS Apex — System Call Application Binary Interface (ABI)

## Invocation Mechanism

System calls are invoked from Ring 3 userspace via the standard software interrupt gate:
```assembly
int 0x80
```

## Register Calling Convention

- `RAX`: System Call Number
- `RDI`: First Argument (arg1)
- `RSI`: Second Argument (arg2)
- `RDX`: Third Argument (arg3)
- `RCX`: Fourth Argument (arg4)
- `R8` : Fifth Argument (arg5)
- `RAX`: Return value (0 or positive indicates success; negative values denote error codes)

## System Call Vector Table

| Number | Name | Arguments | Description |
| :--- | :--- | :--- | :--- |
| `1` | `SYS_EXIT` | `(code: i32)` | Terminate current calling process |
| `2` | `SYS_READ` | `(fd: usize, buf: *mut u8, len: usize)` | Read bytes from open file/stream |
| `3` | `SYS_WRITE` | `(fd: usize, buf: *const u8, len: usize)` | Write bytes to file/stdout/stderr |
| `4` | `SYS_OPEN` | `(path: *const u8, flags: usize, len: usize)` | Open VFS file handle |
| `5` | `SYS_CLOSE` | `(fd: usize)` | Close open file handle |
| `6` | `SYS_SPAWN` | `(path: *const u8, argv: *const u8)` | Spawn child process from binary |
| `7` | `SYS_WAIT` | `(pid: usize, status: *mut i32)` | Wait for child process exit |
| `8` | `SYS_YIELD` | `()` | Yield remaining quantum to scheduler |
| `9` | `SYS_SLEEP` | `(ms: u64)` | Suspend process for duration |
| `10` | `SYS_GET_TIME` | `()` | Return monotonic system uptime in ms |
| `11` | `SYS_MAP_MEMORY`| `(addr: usize, len: usize, prot: usize)` | Map virtual memory pages |
| `12` | `SYS_UNMAP_MEMORY`| `(addr: usize, len: usize)` | Unmap virtual memory pages |
| `13` | `SYS_SEND_IPC` | `(dest_pid: usize, msg: *const u8, len: usize)` | Send synchronous/async message |
| `14` | `SYS_RECEIVE_IPC`| `(src_pid: usize, buf: *mut u8, len: usize)` | Receive message from queue |

## Error Codes

- `-1` (`EPERM`): Operation not permitted
- `-2` (`ENOENT`): No such file or directory
- `-9` (`EBADF`): Bad file descriptor
- `-12` (`ENOMEM`): Out of memory
- `-13` (`EACCES`): Permission denied
- `-14` (`EFAULT`): Bad address (null, unaligned, or kernel pointer violation)
- `-22` (`EINVAL`): Invalid argument
- `-38` (`ENOSYS`): Function not implemented
