//! NovaOS Apex — BSD Network Socket Abstraction
//! Manages SOCK_STREAM (TCP), SOCK_DGRAM (UDP), and SOCK_RAW handles with multi-user port security.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::network::ipv4::Ipv4Address;
use crate::network::tcp::{TcpConnection, TcpPacket, TcpState};
use crate::scheduler::sync::SpinLock;
use crate::security::{Credentials, ROOT_UID};

pub const AF_INET: u32 = 2;
pub const SOCK_STREAM: u32 = 1;
pub const SOCK_DGRAM: u32 = 2;
pub const SOCK_RAW: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SocketHandle(pub u32);

pub struct Socket {
    pub domain: u32,
    pub sock_type: u32,
    pub protocol: u32,
    pub local_ip: Ipv4Address,
    pub local_port: u16,
    pub remote_ip: Ipv4Address,
    pub remote_port: u16,
    pub tcp_conn: Option<TcpConnection>,
    pub rx_queue: Vec<Vec<u8>>,
    pub owner_uid: u32,
}

pub struct SocketTable {
    pub sockets: BTreeMap<u32, Socket>,
    next_handle: u32,
}

impl SocketTable {
    pub fn new() -> Self {
        Self {
            sockets: BTreeMap::new(),
            next_handle: 100, // Socket handles start at 100
        }
    }

    pub fn create_socket(
        &mut self,
        domain: u32,
        sock_type: u32,
        protocol: u32,
        creds: &Credentials,
    ) -> Result<SocketHandle, i32> {
        if domain != AF_INET {
            return Err(-22); // EINVAL
        }

        let handle = SocketHandle(self.next_handle);
        self.next_handle += 1;

        let tcp_conn = if sock_type == SOCK_STREAM {
            Some(TcpConnection::new(
                Ipv4Address::LOCALHOST,
                0,
                Ipv4Address::UNSPECIFIED,
                0,
            ))
        } else {
            None
        };

        let sock = Socket {
            domain,
            sock_type,
            protocol,
            local_ip: Ipv4Address::UNSPECIFIED,
            local_port: 0,
            remote_ip: Ipv4Address::UNSPECIFIED,
            remote_port: 0,
            tcp_conn,
            rx_queue: Vec::new(),
            owner_uid: creds.euid.0,
        };

        self.sockets.insert(handle.0, sock);
        Ok(handle)
    }

    pub fn bind(
        &mut self,
        handle: SocketHandle,
        ip: Ipv4Address,
        port: u16,
        creds: &Credentials,
    ) -> Result<(), i32> {
        // Multi-User Port Security: Privileged ports (< 1024) require root UID 0
        if port < 1024 && creds.euid != ROOT_UID {
            return Err(-13); // EACCES (Permission denied)
        }

        let sock = self.sockets.get_mut(&handle.0).ok_or(-9)?; // EBADF
        sock.local_ip = ip;
        sock.local_port = port;

        if let Some(conn) = &mut sock.tcp_conn {
            conn.local_ip = ip;
            conn.local_port = port;
        }

        Ok(())
    }

    pub fn listen(&mut self, handle: SocketHandle, _backlog: usize) -> Result<(), i32> {
        let sock = self.sockets.get_mut(&handle.0).ok_or(-9)?;
        if sock.sock_type != SOCK_STREAM {
            return Err(-22); // EINVAL
        }

        if let Some(conn) = &mut sock.tcp_conn {
            conn.state = TcpState::Listen;
            Ok(())
        } else {
            Err(-22)
        }
    }

    pub fn connect(
        &mut self,
        handle: SocketHandle,
        remote_ip: Ipv4Address,
        remote_port: u16,
    ) -> Result<Option<TcpPacket>, i32> {
        let sock = self.sockets.get_mut(&handle.0).ok_or(-9)?;
        sock.remote_ip = remote_ip;
        sock.remote_port = remote_port;

        if sock.local_port == 0 {
            sock.local_port = 49152; // Ephemeral port
        }
        if sock.local_ip == Ipv4Address::UNSPECIFIED {
            sock.local_ip = Ipv4Address::new(10, 0, 2, 15);
        }

        if sock.sock_type == SOCK_STREAM {
            let conn = sock.tcp_conn.as_mut().ok_or(-22)?;
            conn.local_ip = sock.local_ip;
            conn.local_port = sock.local_port;
            conn.remote_ip = remote_ip;
            conn.remote_port = remote_port;
            conn.state = TcpState::SynSent;

            // Generate initial SYN packet
            let syn = TcpPacket::new(
                conn.local_port,
                conn.remote_port,
                conn.local_seq,
                0,
                crate::network::tcp::TCP_SYN,
                conn.local_ip,
                conn.remote_ip,
                &[],
            );
            conn.local_seq = conn.local_seq.wrapping_add(1);
            Ok(Some(syn))
        } else {
            Ok(None)
        }
    }

    pub fn send(&mut self, handle: SocketHandle, data: &[u8]) -> Result<usize, i32> {
        let sock = self.sockets.get_mut(&handle.0).ok_or(-9)?;
        if sock.sock_type == SOCK_STREAM {
            let conn = sock.tcp_conn.as_mut().ok_or(-22)?;
            if conn.state != TcpState::Established {
                return Err(-107); // ENOTCONN
            }
            conn.tx_buffer.extend_from_slice(data);
            Ok(data.len())
        } else {
            sock.rx_queue.push(data.to_vec());
            Ok(data.len())
        }
    }

    pub fn recv(&mut self, handle: SocketHandle, buf: &mut [u8]) -> Result<usize, i32> {
        let sock = self.sockets.get_mut(&handle.0).ok_or(-9)?;
        if sock.sock_type == SOCK_STREAM {
            let conn = sock.tcp_conn.as_mut().ok_or(-22)?;
            if conn.rx_buffer.is_empty() {
                return Ok(0);
            }
            let to_copy = buf.len().min(conn.rx_buffer.len());
            buf[..to_copy].copy_from_slice(&conn.rx_buffer[..to_copy]);
            conn.rx_buffer.drain(..to_copy);
            Ok(to_copy)
        } else {
            if sock.rx_queue.is_empty() {
                return Ok(0);
            }
            let packet = sock.rx_queue.remove(0);
            let to_copy = buf.len().min(packet.len());
            buf[..to_copy].copy_from_slice(&packet[..to_copy]);
            Ok(to_copy)
        }
    }

    pub fn close(&mut self, handle: SocketHandle) -> Result<(), i32> {
        if self.sockets.remove(&handle.0).is_some() {
            Ok(())
        } else {
            Err(-9)
        }
    }

    pub fn get_socket_mut(&mut self, handle: SocketHandle) -> Option<&mut Socket> {
        self.sockets.get_mut(&handle.0)
    }
}

pub static SOCKET_TABLE: SpinLock<SocketTable> = SpinLock::new("socket_table", SocketTable {
    sockets: BTreeMap::new(),
    next_handle: 100,
});
