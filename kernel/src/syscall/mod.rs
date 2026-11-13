pub mod abi;

use abi::*;

pub fn dispatch_syscall(number: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    match number {
        SYS_EXIT => {
            crate::serial_println!("[SYSCALL: EXIT] Code: {}", arg1);
            0
        }
        SYS_WRITE => {
            // arg1 = fd, arg2 = buffer pointer, arg3 = length
            if arg2 == 0 || arg3 > 4096 {
                return -1; // Boundary error
            }
            let slice = unsafe { core::slice::from_raw_parts(arg2 as *const u8, arg3) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::print!("{}", s);
                crate::serial_print!("{}", s);
                arg3 as isize
            } else {
                -2
            }
        }
        SYS_YIELD => {
            crate::scheduler::schedule_tick();
            0
        }
        SYS_GET_TIME => {
            crate::interrupts::timer::get_ticks() as isize
        }
        _ => -38, // ENOSYS
    }
}
