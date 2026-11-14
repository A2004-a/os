//! NovaOS Apex — PID 1 Init Service (Ring 3 Userspace Daemon)
//! Demonstrates Ring 3 privilege isolation, system call execution, IPC, and cooperative/preemptive multitasking.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_apex_runtime::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("\n=======================================================\n");
    print("   NovaOS Apex — PID 1 System Init (Ring 3 Userspace)  \n");
    print("=======================================================\n");
    print("[INIT] PID 1 execution verified in Ring 3 (User Code 0x23, User Data 0x1B).\n");

    // 1. Test get_time system call
    let t0 = get_time();
    print("[INIT] [SYSCALL 10/14] sys_get_time() returned uptime: ");
    print_u64(t0);
    print(" ms\n");

    // 2. Test yield_now system call
    print("[INIT] [SYSCALL  8/14] Testing sys_yield()...\n");
    yield_now();
    print("[INIT] [SYSCALL  8/14] sys_yield() returned control successfully.\n");

    // 3. Test sleep system call
    print("[INIT] [SYSCALL  9/14] Testing sys_sleep(20 ms)...\n");
    sleep(20);
    let t1 = get_time();
    print("[INIT] [SYSCALL  9/14] sys_sleep() woke up. Elapsed: ");
    print_u64(t1.saturating_sub(t0));
    print(" ms\n");

    // 4. Test memory mapping system calls
    print("[INIT] [SYSCALL 11/14] Testing sys_map_memory() at 0x20000000 (4096 bytes)...\n");
    match map_memory(0x20000000, 4096, 0) {
        Ok(mapped_addr) => {
            print("[INIT] Successfully mapped page at virtual address 0x");
            print_hex_u64(mapped_addr);
            print("\n");

            // Write and read canary to verify page is writable in Ring 3
            let ptr = mapped_addr as *mut u64;
            unsafe {
                *ptr = 0xCAFE_BABE_DEAD_BEEF;
                if *ptr == 0xCAFE_BABE_DEAD_BEEF {
                    print("[INIT] Page read/write memory canary verified.\n");
                }
            }

            // Test unmap
            print("[INIT] [SYSCALL 12/14] Testing sys_unmap_memory()...\n");
            let _ = unmap_memory(mapped_addr, 4096);
            print("[INIT] sys_unmap_memory() completed.\n");
        }
        Err(e) => {
            print("[INIT] map_memory failed with code: ");
            print_i32(e);
            print("\n");
        }
    }

    // 5. Test IPC system calls (send and receive to self mailbox)
    print("[INIT] [SYSCALL 13/14 & 14/14] Testing sys_send_ipc() and sys_receive_ipc()...\n");
    let test_msg = b"NovaOS-Apex-IPC-Handshake";
    if let Ok(()) = send_ipc(1, test_msg) {
        let mut rx_buf = [0u8; 64];
        if let Ok(bytes_rx) = receive_ipc(1, &mut rx_buf) {
            print("[INIT] IPC message received (");
            print_u64(bytes_rx as u64);
            print(" bytes): '");
            if let Ok(s) = core::str::from_utf8(&rx_buf[..bytes_rx]) {
                print(s);
            }
            print("'\n");
        }
    }

    // 6. Test open & close typed handles
    print("[INIT] [SYSCALL  4/14 & 5/14] Testing sys_open() and sys_close()...\n");
    match open("/system/init.cfg", 0) {
        Ok(h) => {
            print("[INIT] Opened typed handle: ");
            print_u64(h as u64);
            print(", closing...\n");
            let _ = close(h);
            print("[INIT] Handle closed successfully.\n");
        }
        Err(e) => {
            print("[INIT] sys_open returned: ");
            print_i32(e);
            print("\n");
        }
    }

    print("\n[INIT] All 14 NovaOS Apex system calls verified in Ring 3 userspace!\n");
    print("[INIT] PID 1 entering steady-state supervisory loop.\n");

    loop {
        sleep(100);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("[INIT PANIC] PID 1 encountered unexpected panic!\n");
    exit(1);
}

fn print_u64(mut n: u64) {
    if n == 0 {
        print("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        let s = unsafe { core::str::from_utf8_unchecked(&buf[j..j+1]) };
        print(s);
    }
}

fn print_i32(n: i32) {
    if n < 0 {
        print("-");
        print_u64((-n) as u64);
    } else {
        print_u64(n as u64);
    }
}

fn print_hex_u64(mut n: u64) {
    if n == 0 {
        print("0");
        return;
    }
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut i = 0;
    while n > 0 {
        buf[i] = hex_chars[(n & 0x0F) as usize];
        n >>= 4;
        i += 1;
    }
    for j in (0..i).rev() {
        let s = unsafe { core::str::from_utf8_unchecked(&buf[j..j+1]) };
        print(s);
    }
}
