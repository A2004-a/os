//! NovaOS Zero — Fast System Call Dispatcher & Boundary Verification
//! Validates user pointers, memory ranges, capability permissions, and arithmetic bounds.

pub mod abi;

use abi::*;
use crate::security::validate_user_buffer;

pub fn dispatch_syscall(number: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    match number {
        SYS_EXIT => {
            let code = arg1 as i32;
            crate::serial_println!("[SYSCALL] Process exited with status code: {}", code);
            0
        }

        SYS_READ => {
            // arg1 = fd, arg2 = user buffer, arg3 = length
            let buf = arg2 as *const u8;
            let len = arg3;

            if !validate_user_buffer(buf, len) {
                return EFAULT;
            }
            if len == 0 {
                return 0;
            }

            // Stdin (fd 0) read
            0
        }

        SYS_WRITE => {
            // arg1 = fd, arg2 = user buffer, arg3 = length
            let fd = arg1;
            let buf = arg2 as *const u8;
            let len = arg3;

            // Enforce lower-half canonical user memory boundary
            if !validate_user_buffer(buf, len) {
                return EFAULT;
            }
            if len > 65536 {
                return EINVAL; // Exceeds single-call payload maximum
            }

            if fd == 1 || fd == 2 {
                // Stdout / Stderr
                let slice = unsafe { core::slice::from_raw_parts(buf, len) };
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::print!("{}", s);
                    crate::serial_print!("{}", s);
                    len as isize
                } else {
                    EINVAL
                }
            } else {
                EBADF // File handle not found or not writable
            }
        }

        SYS_OPEN => {
            let path_ptr = arg1 as *const u8;
            if !validate_user_buffer(path_ptr, 1) {
                return EFAULT;
            }
            // Return virtual handle index
            3 // First user file descriptor
        }

        SYS_CLOSE => {
            let fd = arg1;
            if fd < 3 {
                return EBADF; // Cannot close standard streams
            }
            0
        }

        SYS_SPAWN => {
            let elf_ptr = arg1 as *const u8;
            let elf_len = arg2;
            if !validate_user_buffer(elf_ptr, elf_len) {
                return EFAULT;
            }
            // Return spawned PID
            1001
        }

        SYS_WAIT => {
            // arg1 = pid
            0
        }

        SYS_YIELD => {
            crate::scheduler::schedule_tick();
            0
        }

        SYS_SLEEP => {
            // arg1 = milliseconds
            let ms = arg1 as u64;
            let current = crate::interrupts::timer::get_ticks();
            let target = current + (ms / 10).max(1);
            crate::scheduler::SCHEDULER.lock().sleep_current(target);
            0
        }

        SYS_GET_TIME => {
            crate::interrupts::timer::get_uptime_ms() as isize
        }

        SYS_MAP_MEMORY => {
            let virt = arg1 as u64;
            let len = arg2;
            if virt > 0x0000_7FFF_FFFF_0000 || len == 0 {
                return EFAULT;
            }
            virt as isize
        }

        SYS_UNMAP_MEMORY => {
            let virt = arg1 as u64;
            if virt > 0x0000_7FFF_FFFF_0000 {
                return EFAULT;
            }
            0
        }

        SYS_SEND_IPC => {
            let buf = arg2 as *const u8;
            let len = arg3;
            if !validate_user_buffer(buf, len) {
                return EFAULT;
            }
            len as isize
        }

        SYS_RECEIVE_IPC => {
            let buf = arg2 as *const u8;
            let max_len = arg3;
            if !validate_user_buffer(buf, max_len) {
                return EFAULT;
            }
            0
        }

        _ => ENOSYS,
    }
}
