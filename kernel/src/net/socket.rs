//! NovaOS Zero — Kernel Socket Abstraction Layer
//! Provides UDP datagram and TCP stream endpoints for userspace tasks.

use super::ipv4::Ipv4Address;
use super::tcp::TcpState;

pub const MAX_SOCKETS: usize = 16;
pub const SOCKET_BUF_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Udp,
    Tcp,
}

#[derive(Clone, Copy)]
pub struct Socket {
    pub id: u32,
    pub socket_type: SocketType,
    pub local_port: u16,
    pub remote_ip: Option<Ipv4Address>,
    pub remote_port: Option<u16>,
    pub tcp_state: TcpState,
    pub rx_buf: [u8; SOCKET_BUF_SIZE],
    pub rx_len: usize,
    pub is_active: bool,
}

impl Socket {
    pub const fn empty() -> Self {
        Socket {
            id: 0,
            socket_type: SocketType::Udp,
            local_port: 0,
            remote_ip: None,
            remote_port: None,
            tcp_state: TcpState::Closed,
            rx_buf: [0; SOCKET_BUF_SIZE],
            rx_len: 0,
            is_active: false,
        }
    }
}

pub struct SocketTable {
    sockets: [Socket; MAX_SOCKETS],
}

impl SocketTable {
    pub const fn new() -> Self {
        SocketTable {
            sockets: [Socket::empty(); MAX_SOCKETS],
        }
    }

    /// Allocate a new socket endpoint
    pub fn create_socket(&mut self, socket_type: SocketType) -> Option<u32> {
        for (i, sock) in self.sockets.iter_mut().enumerate() {
            if !sock.is_active {
                *sock = Socket {
                    id: (i + 1) as u32,
                    socket_type,
                    local_port: 0,
                    remote_ip: None,
                    remote_port: None,
                    tcp_state: match socket_type {
                        SocketType::Tcp => TcpState::Closed,
                        SocketType::Udp => TcpState::Established,
                    },
                    rx_buf: [0; SOCKET_BUF_SIZE],
                    rx_len: 0,
                    is_active: true,
                };
                return Some((i + 1) as u32);
            }
        }
        None
    }

    /// Bind a local port to a socket
    pub fn bind(&mut self, sock_id: u32, port: u16) -> Result<(), &'static str> {
        let index = (sock_id as usize).wrapping_sub(1);
        if index >= MAX_SOCKETS || !self.sockets[index].is_active {
            return Err("Invalid socket handle");
        }

        // Ensure port is not already bound
        for (i, s) in self.sockets.iter().enumerate() {
            if i != index && s.is_active && s.local_port == port {
                return Err("Port already in use");
            }
        }

        self.sockets[index].local_port = port;
        if self.sockets[index].socket_type == SocketType::Tcp {
            self.sockets[index].tcp_state = TcpState::Listen;
        }

        Ok(())
    }

    /// Push received payload into socket receive queue
    pub fn deliver_payload(&mut self, dest_port: u16, data: &[u8]) -> bool {
        for s in self.sockets.iter_mut() {
            if s.is_active && s.local_port == dest_port {
                let copy_len = data.len().min(SOCKET_BUF_SIZE - s.rx_len);
                s.rx_buf[s.rx_len..s.rx_len + copy_len].copy_from_slice(&data[..copy_len]);
                s.rx_len += copy_len;
                return true;
            }
        }
        false
    }

    /// Read data from a socket's receive buffer
    pub fn receive(&mut self, sock_id: u32, buf: &mut [u8]) -> Result<usize, &'static str> {
        let index = (sock_id as usize).wrapping_sub(1);
        if index >= MAX_SOCKETS || !self.sockets[index].is_active {
            return Err("Invalid socket handle");
        }

        let sock = &mut self.sockets[index];
        if sock.rx_len == 0 {
            return Ok(0); // Would block / empty queue
        }

        let to_copy = buf.len().min(sock.rx_len);
        buf[..to_copy].copy_from_slice(&sock.rx_buf[..to_copy]);

        // Shift remaining buffer bytes
        sock.rx_buf.copy_within(to_copy..sock.rx_len, 0);
        sock.rx_len -= to_copy;

        Ok(to_copy)
    }

    /// Close a socket
    pub fn close(&mut self, sock_id: u32) -> Result<(), &'static str> {
        let index = (sock_id as usize).wrapping_sub(1);
        if index >= MAX_SOCKETS || !self.sockets[index].is_active {
            return Err("Invalid socket handle");
        }

        self.sockets[index].is_active = false;
        self.sockets[index].tcp_state = TcpState::Closed;
        Ok(())
    }
}
