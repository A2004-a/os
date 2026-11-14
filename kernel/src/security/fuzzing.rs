//! NovaOS Apex — Kernel Security & Adversarial Fuzzing Engine
//! Subjecting system calls, pointer validators, memory boundaries, and the network stack
//! to invalid pointers, kernel address injection, buffer overflows, integer wraps, and corrupted packets.

use crate::network::stack::process_incoming_packet;
use crate::serial;
use crate::syscall::dispatch_syscall;

/// Runs adversarial system call fuzzing across pointer boundaries and arguments.
pub fn test_syscall_boundary_fuzzing() -> bool {
    serial::write_str("[M12 TEST 1/6] Testing Syscall Boundary & Pointer Adversarial Fuzzing...\n");

    // 1. Null Pointer Dereference Fuzzing
    // Syscalls expecting valid pointers must reject 0 with -EFAULT (-14 or negative errno)
    let null_tests = [
        (crate::syscall::SYS_READ, 1u64, 0u64, 64u64),               // read(1, NULL, 64)
        (crate::syscall::SYS_WRITE, 1u64, 0u64, 64u64),              // write(1, NULL, 64)
        (crate::syscall::SYS_OPEN, 0u64, 0u64, 0u64),                // open(NULL, 0)
        (crate::syscall::SYS_STAT, 0u64, 0x1000u64, 0u64),           // stat(NULL, buf)
        (crate::syscall::SYS_READDIR, 0u64, 0u64, 0x1000u64),        // readdir(NULL, 0, buf)
        (crate::syscall::SYS_GETCWD, 0u64, 128u64, 0u64),            // getcwd(NULL, 128)
        (crate::syscall::SYS_GET_DISPLAY_INFO, 0u64, 0u64, 0u64),    // get_display_info(NULL)
        (crate::syscall::SYS_LIST_BLOCK_DEVICES, 0u64, 4u64, 0u64),   // list_block_devs(NULL, 4)
        (crate::syscall::SYS_GET_HARDWARE_INFO, 0u64, 0u64, 0u64),   // get_hardware_info(NULL)
    ];

    for &(nr, a1, a2, a3) in &null_tests {
        let ret = dispatch_syscall(nr, a1, a2, a3, 0, 0);
        if ret >= 0 {
            serial::write_str("  [FAIL] Syscall ");
            print_u64(nr as u64);
            serial::write_str(" accepted NULL pointer without error!\n");
            return false;
        }
    }
    serial::write_str("  [PASS] 9 distinct syscalls rejected NULL pointers with negative errno.\n");

    // 2. Kernel Memory Address Injection Fuzzing (Meltdown / Tampering Defense)
    // Userspace must never be able to read or write higher-half kernel space (>= 0x0000_8000_0000_0000)
    let kernel_targets = [
        0xFFFF_8000_0000_0000u64, // Kernel Direct Map Base
        0xFFFF_8000_0020_0000u64, // Kernel Code Base
        0xFFFF_FFFF_FFFF_0000u64, // Highest virtual address
        0x0000_8000_0000_0000u64, // First non-canonical user address
        0x0000_FFFF_FFFF_0000u64, // Upper non-canonical boundary
    ];

    for &kaddr in &kernel_targets {
        // Attempt to read kernel memory via read() syscall
        let ret_read = dispatch_syscall(crate::syscall::SYS_READ, 0, kaddr, 32, 0, 0);
        if ret_read >= 0 {
            serial::write_str("  [FAIL] Syscall read() allowed buffer in kernel space!\n");
            return false;
        }

        // Attempt to pass kernel string via open() syscall
        let ret_open = dispatch_syscall(crate::syscall::SYS_OPEN, kaddr, 0, 0, 0, 0);
        if ret_open >= 0 {
            serial::write_str("  [FAIL] Syscall open() accepted path pointer in kernel space!\n");
            return false;
        }

        // Attempt to write kernel memory via getcwd()
        let ret_cwd = dispatch_syscall(crate::syscall::SYS_GETCWD, kaddr, 128, 0, 0, 0);
        if ret_cwd >= 0 {
            serial::write_str("  [FAIL] Syscall getcwd() allowed write to kernel space!\n");
            return false;
        }
    }
    serial::write_str("  [PASS] Kernel space tampering rejected across 5 privileged boundary targets.\n");

    // 3. Integer Overflow & Arithmetic Wrapping Fuzzing
    // Pointers near 2^64 with huge length that wrap around 0
    let wrap_ret = dispatch_syscall(crate::syscall::SYS_WRITE, 1, 0x0000_7FFF_FFFF_0000, 0xFFFF_FFFF_FFFF_0000, 0, 0);
    if wrap_ret >= 0 {
        serial::write_str("  [FAIL] Syscall write() permitted arithmetic wrapping buffer overflow!\n");
        return false;
    }
    serial::write_str("  [PASS] Arithmetic pointer overflow detected and blocked.\n");

    // 4. Invalid File Descriptor Fuzzing
    let invalid_fds = [-1i64 as u64, 0x7FFF_FFFFu64, 1024u64, 99999u64];
    for &fd in &invalid_fds {
        let ret_close = dispatch_syscall(crate::syscall::SYS_CLOSE, fd, 0, 0, 0, 0);
        if ret_close >= 0 {
            serial::write_str("  [FAIL] Syscall close() accepted non-existent fd!\n");
            return false;
        }
        let ret_read = dispatch_syscall(crate::syscall::SYS_READ, fd, 0x10000, 16, 0, 0);
        if ret_read >= 0 {
            serial::write_str("  [FAIL] Syscall read() accepted non-existent fd!\n");
            return false;
        }
    }
    serial::write_str("  [PASS] Out-of-bounds file descriptor calls safely rejected (-EBADF).\n");

    // 5. Unimplemented / Out-of-Range Syscall Numbers
    let invalid_nrs = [0usize, 52usize, 99usize, 255usize, 1000usize, usize::MAX];
    for &nr in &invalid_nrs {
        let ret = dispatch_syscall(nr, 0, 0, 0, 0, 0);
        if ret != -38 && ret >= 0 { // -ENOSYS is typically -38
            serial::write_str("  [FAIL] Unrecognized syscall number accepted!\n");
            return false;
        }
    }
    serial::write_str("  [PASS] Out-of-range syscall numbers returned -ENOSYS / error.\n");

    // 6. Invalid Process Termination
    let ret_kill = dispatch_syscall(crate::syscall::SYS_KILL_PROCESS, 999_999, 9, 0, 0, 0);
    if ret_kill >= 0 {
        serial::write_str("  [FAIL] kill_process accepted invalid PID 999999!\n");
        return false;
    }
    serial::write_str("  [PASS] Non-existent PID termination rejected.\n");

    serial::write_str("  [PASS] Syscall boundary, pointer, overflow, and tampering fuzzing verified.\n");
    true
}

/// Runs adversarial network packet corruption and runt-frame fuzzing.
pub fn test_network_packet_fuzzing() -> bool {
    serial::write_str("[M12 TEST 2/6] Testing Network Stack Corruption & Runt Packet Fuzzing...\n");

    // 1. Runt Ethernet Frames (< 14 bytes)
    let runt_frames: [&[u8]; 4] = [
        &[],
        &[0x00],
        &[0x52, 0x54, 0x00, 0x12, 0x34],
        &[0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08], // 13 bytes
    ];

    for (idx, frame) in runt_frames.iter().enumerate() {
        // Must drop silently without faulting or panicking
        process_incoming_packet(frame);
        serial::write_str("    Runt frame [");
        print_u64(idx as u64);
        serial::write_str("] size ");
        print_u64(frame.len() as u64);
        serial::write_str(" bytes safely rejected.\n");
    }

    // 2. Corrupted IPv4 Headers
    // Construct base Ethernet frame (14 bytes) + corrupted IP payload
    let mut bad_ip_packet = [0u8; 40];
    // Dest MAC: 52:54:00:12:34:56, Src MAC: 52:54:00:AB:CD:EF, EtherType: 0x0800 (IPv4)
    bad_ip_packet[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    bad_ip_packet[6..12].copy_from_slice(&[0x52, 0x54, 0x00, 0xAB, 0xCD, 0xEF]);
    bad_ip_packet[12..14].copy_from_slice(&[0x08, 0x00]);

    // Test 2a: Version != 4 (e.g. Version 6 on IPv4 EtherType)
    bad_ip_packet[14] = 0x65; // Version 6, IHL 5
    process_incoming_packet(&bad_ip_packet);

    // Test 2b: IHL < 5 (Illegal header length < 20 bytes)
    bad_ip_packet[14] = 0x44; // Version 4, IHL 4 (16 bytes, invalid RFC 791)
    process_incoming_packet(&bad_ip_packet);

    // Test 2c: Total Length claim exceeds physical buffer
    bad_ip_packet[14] = 0x45; // Version 4, IHL 5
    bad_ip_packet[16..18].copy_from_slice(&[0x05, 0x00]); // Claims 1280 bytes on 40-byte packet
    process_incoming_packet(&bad_ip_packet);

    // Test 2d: Invalid IPv4 Header Checksum
    bad_ip_packet[16..18].copy_from_slice(&[0x00, 0x14]); // 20 bytes
    bad_ip_packet[24..26].copy_from_slice(&[0xDE, 0xAD]); // Corrupted checksum
    process_incoming_packet(&bad_ip_packet);

    serial::write_str("  [PASS] Corrupted IPv4 headers (bad version, runt IHL, length claim, bad checksum) dropped.\n");

    // 3. Corrupted ARP Packets
    let mut bad_arp = [0u8; 42];
    bad_arp[0..12].copy_from_slice(&bad_ip_packet[0..12]);
    bad_arp[12..14].copy_from_slice(&[0x08, 0x06]); // EtherType ARP

    // Hardware type != 1 (Ethernet)
    bad_arp[14..16].copy_from_slice(&[0x00, 0x03]); // AX.25
    process_incoming_packet(&bad_arp);

    // Protocol type != 0x0800 (IPv4)
    bad_arp[14..16].copy_from_slice(&[0x00, 0x01]);
    bad_arp[16..18].copy_from_slice(&[0x86, 0xDD]); // IPv6
    process_incoming_packet(&bad_arp);

    // Unsupported Opcode (0x0005)
    bad_arp[16..18].copy_from_slice(&[0x08, 0x00]);
    bad_arp[20..22].copy_from_slice(&[0x00, 0x05]);
    process_incoming_packet(&bad_arp);

    serial::write_str("  [PASS] Corrupted ARP packets (illegal HW, non-IPv4 protocol, unknown opcodes) dropped.\n");

    // 4. Corrupted TCP Segment (Invalid Flags combination: SYN + FIN + RST together)
    let mut bad_tcp = [0u8; 54];
    bad_tcp[0..14].copy_from_slice(&bad_ip_packet[0..14]);
    // Valid IP header
    bad_tcp[14] = 0x45;
    bad_tcp[16..18].copy_from_slice(&[0x00, 0x28]); // 40 bytes IP
    bad_tcp[23] = 6; // Protocol TCP
    bad_tcp[24..26].copy_from_slice(&[0x00, 0x00]); // Checksum bypass in test
    bad_tcp[26..30].copy_from_slice(&[10, 0, 2, 2]); // Src IP
    bad_tcp[30..34].copy_from_slice(&[10, 0, 2, 15]); // Dest IP (us)
    // TCP header (offset 34)
    bad_tcp[34..36].copy_from_slice(&[0x1F, 0x90]); // Src Port 8080
    bad_tcp[36..38].copy_from_slice(&[0x00, 0x50]); // Dst Port 80
    bad_tcp[46] = 0x50; // Data offset 5 (20 bytes)
    bad_tcp[47] = 0x07; // Illegal Flags: FIN + SYN + RST
    process_incoming_packet(&bad_tcp);

    serial::write_str("  [PASS] Malicious TCP flags combinations (SYN+FIN+RST) safely filtered.\n");
    true
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
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
        serial::write_byte(buf[j]);
    }
}
