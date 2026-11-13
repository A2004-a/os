# NovaOS System Call ABI Specification

NovaOS utilizes the `SYSCALL` / `SYSRET` x86_64 fast system call interface.

## 1. Register Calling Conventions

| Argument | Register |
|---|---|
| Syscall Number | `RAX` |
| Argument 1 | `RDI` |
| Argument 2 | `RSI` |
| Argument 3 | `RDX` |
| Return Value | `RAX` |

## 2. Syscall Table

| Number | Name | Arguments | Description |
|---|---|---|---|
| `1` | `SYS_EXIT` | `arg1 = exit_code` | Terminate calling process |
| `2` | `SYS_READ` | `arg1 = fd, arg2 = buf, arg3 = len` | Read bytes from descriptor |
| `3` | `SYS_WRITE` | `arg1 = fd, arg2 = buf, arg3 = len` | Write bytes to descriptor |
| `6` | `SYS_YIELD` | *(none)* | Yield CPU timeslice |
| `12` | `SYS_GET_TIME` | *(none)* | Return current system timer ticks |
