//! NovaOS Zero — Transmission Control Protocol (TCP)
//! RFC 793 Segment Framing, Sequence Control & Connection State Machine

pub const TCP_FIN: u8 = 1 << 0;
pub const TCP_SYN: u8 = 1 << 1;
pub const TCP_RST: u8 = 1 << 2;
pub const TCP_PSH: u8 = 1 << 3;
pub const TCP_ACK: u8 = 1 << 4;
pub const TCP_URG: u8 = 1 << 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_number: u32,
    pub ack_number: u32,
    pub data_offset_reserved: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}

pub const TCP_HEADER_MIN_LEN: usize = core::mem::size_of::<TcpHeader>(); // 20 bytes

pub struct TcpSegment<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_number: u32,
    pub ack_number: u32,
    pub flags: u8,
    pub window_size: u16,
    pub payload: &'a [u8],
}

impl<'a> TcpSegment<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self, &'static str> {
        if buf.len() < TCP_HEADER_MIN_LEN {
            return Err("Buffer smaller than minimum TCP header");
        }

        let hdr = unsafe { &*(buf.as_ptr() as *const TcpHeader) };
        let offset = ((hdr.data_offset_reserved >> 4) as usize) * 4;

        if offset < TCP_HEADER_MIN_LEN || offset > buf.len() {
            return Err("Invalid TCP data offset");
        }

        Ok(TcpSegment {
            src_port: u16::from_be(hdr.src_port),
            dst_port: u16::from_be(hdr.dst_port),
            seq_number: u32::from_be(hdr.seq_number),
            ack_number: u32::from_be(hdr.ack_number),
            flags: hdr.flags,
            window_size: u16::from_be(hdr.window_size),
            payload: &buf[offset..],
        })
    }

    pub fn serialize(
        src_port: u16,
        dst_port: u16,
        seq_num: u32,
        ack_num: u32,
        flags: u8,
        window: u16,
        payload: &[u8],
        out: &mut [u8],
    ) -> Result<usize, &'static str> {
        let total_len = TCP_HEADER_MIN_LEN + payload.len();
        if out.len() < total_len {
            return Err("Output buffer too small for TCP segment");
        }

        let hdr = TcpHeader {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            seq_number: seq_num.to_be(),
            ack_number: ack_num.to_be(),
            data_offset_reserved: (5 << 4), // 5 * 4 = 20 bytes
            flags,
            window_size: window.to_be(),
            checksum: 0,
            urgent_pointer: 0,
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                &hdr as *const TcpHeader as *const u8,
                out.as_mut_ptr(),
                TCP_HEADER_MIN_LEN,
            );
        }

        out[TCP_HEADER_MIN_LEN..total_len].copy_from_slice(payload);
        Ok(total_len)
    }
}
