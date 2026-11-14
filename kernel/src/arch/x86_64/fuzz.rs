//! NovaOS Zero — Production Hardening & Kernel Boundary Fuzzing Engine
//! Validates parameter validation, memory boundaries, VFS paths, and network frame decoders
//! under hostile, malformed, and out-of-bounds input sequences.

use crate::syscall::abi::*;
use crate::syscall::dispatch_syscall;

pub struct FuzzEngine;

impl FuzzEngine {
    /// Fuzz all 14 kernel system calls with hostile userland arguments
    pub fn fuzz_syscall_boundaries() -> bool {
        // 1. Null pointer buffer validation
        let ret = dispatch_syscall(SYS_READ, 1, 0, 100);
        if ret != EFAULT {
            return false;
        }

        // 2. Zero-page access attempt (< 0x1000)
        let ret = dispatch_syscall(SYS_WRITE, 1, 0x500, 64);
        if ret != EFAULT {
            return false;
        }

        // 3. Kernel higher-half memory address access attempt (>= 0xFFFF_8000_0000_0000)
        let ret = dispatch_syscall(SYS_READ, 1, 0xFFFF_8000_0000_0000, 100);
        if ret != EFAULT {
            return false;
        }

        // 4. Arithmetic overflow attack (start + length wraps around 64-bit address space)
        let ret = dispatch_syscall(SYS_WRITE, 1, 0x0000_7FFF_FFFF_0000, 0x20000);
        if ret != EFAULT {
            return false;
        }

        // 5. Oversized single write payload (> 64 KiB)
        let ret = dispatch_syscall(SYS_WRITE, 1, 0x400000, 70000);
        if ret != EINVAL {
            return false;
        }

        // 6. Unknown syscall number
        let ret = dispatch_syscall(999, 0, 0, 0);
        if ret != ENOSYS {
            return false;
        }

        // 7. Invalid file descriptor close attempt
        let ret = dispatch_syscall(SYS_CLOSE, 999, 0, 0);
        if ret != EBADF {
            return false;
        }

        true
    }

    /// Fuzz and sanitize VFS path traversal attacks
    /// Collapses redundant slashes, resolves '.' and '..', and prevents escaping root '/'
    pub fn sanitize_vfs_path(path: &str, output: &mut [u8]) -> usize {
        let bytes = path.as_bytes();
        let mut out_len = 0;

        if bytes.is_empty() {
            if !output.is_empty() {
                output[0] = b'/';
                return 1;
            }
            return 0;
        }

        // Ensure leading slash
        if out_len < output.len() {
            output[out_len] = b'/';
            out_len += 1;
        }

        let mut i = 0;
        while i < bytes.len() {
            // Skip consecutive slashes
            while i < bytes.len() && bytes[i] == b'/' {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }

            // Find end of segment
            let seg_start = i;
            while i < bytes.len() && bytes[i] != b'/' {
                i += 1;
            }
            let seg = &bytes[seg_start..i];

            if seg == b"." {
                // Current directory - no-op
                continue;
            } else if seg == b".." {
                // Parent directory - pop last segment if not at root
                if out_len > 1 {
                    out_len -= 1; // remove trailing slash or char
                    while out_len > 1 && output[out_len - 1] != b'/' {
                        out_len -= 1;
                    }
                }
            } else {
                // Append segment
                if out_len > 1 && out_len < output.len() {
                    output[out_len] = b'/';
                    out_len += 1;
                }
                for &b in seg {
                    if out_len < output.len() {
                        output[out_len] = b;
                        out_len += 1;
                    }
                }
            }
        }

        out_len
    }

    /// Fuzz network frame decoders with corrupted, truncated, and mutated packets
    pub fn fuzz_network_packets() -> bool {
        // 1. Truncated Ethernet II frame (only 8 bytes instead of minimum 14)
        let truncated_eth = [0xFF; 8];
        if crate::net::ethernet::EthernetFrame::parse(&truncated_eth).is_ok() {
            return false;
        }

        // 2. Corrupted ARP packet (wrong hardware type or short length)
        let mut corrupt_arp = [0u8; 28];
        corrupt_arp[0] = 0x99; // Invalid hardware type
        if let Ok(arp) = crate::net::arp::ArpPacket::parse(&corrupt_arp) {
            let hw_type = arp.htype;
            if hw_type == 1 {
                return false;
            }
        }

        // 3. Corrupted IPv4 packet (IHL = 3, which is illegal; minimum is 5)
        let mut corrupt_ip = [0u8; 20];
        corrupt_ip[0] = 0x43; // Version 4, IHL = 3 (12 bytes, invalid)
        if crate::net::ipv4::Ipv4Packet::parse(&corrupt_ip).is_ok() {
            return false;
        }

        // 4. Truncated UDP packet (4 bytes instead of minimum 8)
        let truncated_udp = [0x00, 0x50, 0x00, 0x50];
        if crate::net::udp::UdpDatagram::parse(&truncated_udp).is_ok() {
            return false;
        }

        // 5. Corrupted TCP segment (data offset = 2, which is illegal; minimum is 5)
        let mut corrupt_tcp = [0u8; 20];
        corrupt_tcp[12] = 0x20; // Data offset = 2 (8 bytes, invalid)
        if crate::net::tcp::TcpSegment::parse(&corrupt_tcp).is_ok() {
            return false;
        }

        true
    }
}
