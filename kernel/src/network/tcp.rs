//! NovaOS Apex — Transmission Control Protocol (TCP)
//! Connection-oriented transport layer with 3-way handshake, sequence tracking, and connection FSM.

extern crate alloc;
use alloc::vec::Vec;
use crate::network::ipv4::{calculate_checksum, Ipv4Address, IP_PROTO_TCP};

pub const TCP_FIN: u8 = 0x01;
pub const TCP_SYN: u8 = 0x02;
pub const TCP_RST: u8 = 0x04;
pub const TCP_PSH: u8 = 0x08;
pub const TCP_ACK: u8 = 0x10;
pub const TCP_URG: u8 = 0x20;

pub const TCP_HEADER_MIN_LEN: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TcpPacket {
    pub src_port: u16,
    pub dest_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_ptr: u16,
    pub payload: Vec<u8>,
}

impl TcpPacket {
    pub fn new(
        src_port: u16,
        dest_port: u16,
        seq_num: u32,
        ack_num: u32,
        flags: u8,
        src_ip: Ipv4Address,
        dest_ip: Ipv4Address,
        payload: &[u8],
    ) -> Self {
        let total_len = (TCP_HEADER_MIN_LEN + payload.len()) as u16;
        let mut pkt = Self {
            src_port,
            dest_port,
            seq_num,
            ack_num,
            data_offset: 5, // 5 * 4 = 20 bytes
            flags,
            window_size: 64240,
            checksum: 0,
            urgent_ptr: 0,
            payload: payload.to_vec(),
        };

        // Compute TCP pseudo-header checksum
        let mut pseudo = Vec::with_capacity(12 + TCP_HEADER_MIN_LEN + payload.len());
        pseudo.extend_from_slice(&src_ip.0);
        pseudo.extend_from_slice(&dest_ip.0);
        pseudo.push(0);
        pseudo.push(IP_PROTO_TCP);
        pseudo.push((total_len >> 8) as u8);
        pseudo.push((total_len & 0xFF) as u8);

        // TCP Header (unchecksummed)
        pseudo.extend_from_slice(&pkt.serialize_unchecksummed());
        pkt.checksum = calculate_checksum(&pseudo);
        pkt
    }

    fn serialize_unchecksummed(&self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(TCP_HEADER_MIN_LEN + self.payload.len());
        raw.push((self.src_port >> 8) as u8);
        raw.push((self.src_port & 0xFF) as u8);
        raw.push((self.dest_port >> 8) as u8);
        raw.push((self.dest_port & 0xFF) as u8);

        raw.push((self.seq_num >> 24) as u8);
        raw.push((self.seq_num >> 16) as u8);
        raw.push((self.seq_num >> 8) as u8);
        raw.push((self.seq_num & 0xFF) as u8);

        raw.push((self.ack_num >> 24) as u8);
        raw.push((self.ack_num >> 16) as u8);
        raw.push((self.ack_num >> 8) as u8);
        raw.push((self.ack_num & 0xFF) as u8);

        raw.push((self.data_offset << 4) & 0xF0);
        raw.push(self.flags);

        raw.push((self.window_size >> 8) as u8);
        raw.push((self.window_size & 0xFF) as u8);

        raw.push((self.checksum >> 8) as u8);
        raw.push((self.checksum & 0xFF) as u8);

        raw.push((self.urgent_ptr >> 8) as u8);
        raw.push((self.urgent_ptr & 0xFF) as u8);

        raw.extend_from_slice(&self.payload);
        raw
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut raw = self.serialize_unchecksummed();
        raw[16] = (self.checksum >> 8) as u8;
        raw[17] = (self.checksum & 0xFF) as u8;
        raw
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < TCP_HEADER_MIN_LEN {
            return None;
        }

        let src_port = ((buf[0] as u16) << 8) | (buf[1] as u16);
        let dest_port = ((buf[2] as u16) << 8) | (buf[3] as u16);

        let seq_num = ((buf[4] as u32) << 24)
            | ((buf[5] as u32) << 16)
            | ((buf[6] as u32) << 8)
            | (buf[7] as u32);

        let ack_num = ((buf[8] as u32) << 24)
            | ((buf[9] as u32) << 16)
            | ((buf[10] as u32) << 8)
            | (buf[11] as u32);

        let data_offset = buf[12] >> 4;
        let header_len = (data_offset as usize) * 4;
        if header_len < TCP_HEADER_MIN_LEN || buf.len() < header_len {
            return None;
        }

        let flags = buf[13];
        let window_size = ((buf[14] as u16) << 8) | (buf[15] as u16);
        let checksum = ((buf[16] as u16) << 8) | (buf[17] as u16);
        let urgent_ptr = ((buf[18] as u16) << 8) | (buf[19] as u16);

        let mut payload = Vec::with_capacity(buf.len() - header_len);
        payload.extend_from_slice(&buf[header_len..]);

        Some(Self {
            src_port,
            dest_port,
            seq_num,
            ack_num,
            data_offset,
            flags,
            window_size,
            checksum,
            urgent_ptr,
            payload,
        })
    }
}

pub struct TcpConnection {
    pub local_ip: Ipv4Address,
    pub local_port: u16,
    pub remote_ip: Ipv4Address,
    pub remote_port: u16,
    pub state: TcpState,
    pub local_seq: u32,
    pub remote_seq: u32,
    pub rx_buffer: Vec<u8>,
    pub tx_buffer: Vec<u8>,
}

impl TcpConnection {
    pub fn new(
        local_ip: Ipv4Address,
        local_port: u16,
        remote_ip: Ipv4Address,
        remote_port: u16,
    ) -> Self {
        Self {
            local_ip,
            local_port,
            remote_ip,
            remote_port,
            state: TcpState::Closed,
            local_seq: 1000,
            remote_seq: 0,
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
        }
    }

    /// Process incoming packet and return response packet if one should be transmitted
    pub fn handle_packet(&mut self, pkt: &TcpPacket) -> Option<TcpPacket> {
        match self.state {
            TcpState::Closed => {
                // If RST, ignore; otherwise send RST
                if (pkt.flags & TCP_RST) == 0 {
                    Some(TcpPacket::new(
                        self.local_port,
                        pkt.src_port,
                        0,
                        pkt.seq_num.wrapping_add(1),
                        TCP_RST | TCP_ACK,
                        self.local_ip,
                        self.remote_ip,
                        &[],
                    ))
                } else {
                    None
                }
            }

            TcpState::Listen => {
                if (pkt.flags & TCP_SYN) != 0 {
                    self.remote_seq = pkt.seq_num.wrapping_add(1);
                    self.remote_port = pkt.src_port;
                    self.state = TcpState::SynReceived;

                    // Send SYN-ACK
                    let syn_ack = TcpPacket::new(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                        TCP_SYN | TCP_ACK,
                        self.local_ip,
                        self.remote_ip,
                        &[],
                    );
                    self.local_seq = self.local_seq.wrapping_add(1);
                    Some(syn_ack)
                } else {
                    None
                }
            }

            TcpState::SynSent => {
                if (pkt.flags & (TCP_SYN | TCP_ACK)) == (TCP_SYN | TCP_ACK) {
                    self.remote_seq = pkt.seq_num.wrapping_add(1);
                    self.state = TcpState::Established;

                    // Send ACK to complete 3-way handshake
                    let ack = TcpPacket::new(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                        TCP_ACK,
                        self.local_ip,
                        self.remote_ip,
                        &[],
                    );
                    Some(ack)
                } else {
                    None
                }
            }

            TcpState::SynReceived => {
                if (pkt.flags & TCP_ACK) != 0 {
                    self.state = TcpState::Established;
                    None
                } else {
                    None
                }
            }

            TcpState::Established => {
                if (pkt.flags & TCP_FIN) != 0 {
                    self.remote_seq = pkt.seq_num.wrapping_add(1);
                    self.state = TcpState::CloseWait;

                    // ACK the FIN
                    let ack = TcpPacket::new(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                        TCP_ACK,
                        self.local_ip,
                        self.remote_ip,
                        &[],
                    );
                    Some(ack)
                } else if !pkt.payload.is_empty() {
                    // Receive payload and send ACK
                    self.rx_buffer.extend_from_slice(&pkt.payload);
                    self.remote_seq = pkt.seq_num.wrapping_add(pkt.payload.len() as u32);

                    let ack = TcpPacket::new(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                        TCP_ACK,
                        self.local_ip,
                        self.remote_ip,
                        &[],
                    );
                    Some(ack)
                } else {
                    None
                }
            }

            TcpState::CloseWait => {
                // Application sends FIN to complete teardown
                self.state = TcpState::LastAck;
                let fin = TcpPacket::new(
                    self.local_port,
                    self.remote_port,
                    self.local_seq,
                    self.remote_seq,
                    TCP_FIN | TCP_ACK,
                    self.local_ip,
                    self.remote_ip,
                    &[],
                );
                self.local_seq = self.local_seq.wrapping_add(1);
                Some(fin)
            }

            TcpState::FinWait1 => {
                if (pkt.flags & TCP_ACK) != 0 {
                    self.state = TcpState::FinWait2;
                }
                None
            }

            TcpState::FinWait2 => {
                if (pkt.flags & TCP_FIN) != 0 {
                    self.remote_seq = pkt.seq_num.wrapping_add(1);
                    self.state = TcpState::TimeWait;

                    let ack = TcpPacket::new(
                        self.local_port,
                        self.remote_port,
                        self.local_seq,
                        self.remote_seq,
                        TCP_ACK,
                        self.local_ip,
                        self.remote_ip,
                        &[],
                    );
                    Some(ack)
                } else {
                    None
                }
            }

            TcpState::LastAck => {
                if (pkt.flags & TCP_ACK) != 0 {
                    self.state = TcpState::Closed;
                }
                None
            }

            TcpState::TimeWait | TcpState::Closing => {
                self.state = TcpState::Closed;
                None
            }
        }
    }
}
