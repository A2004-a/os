//! NovaOS Zero — User Datagram Protocol (UDP)
//! RFC 768 Port Demultiplexing & Connectionless Datagram Encapsulation

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,    // Length in octets including header
    pub checksum: u16,
}

pub const UDP_HEADER_LEN: usize = core::mem::size_of::<UdpHeader>(); // 8 bytes

pub struct UdpDatagram<'a> {
    pub src_port: u16,
    pub dst_port: u16,
    pub payload: &'a [u8],
}

impl<'a> UdpDatagram<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self, &'static str> {
        if buf.len() < UDP_HEADER_LEN {
            return Err("Buffer smaller than UDP header");
        }

        let hdr = unsafe { &*(buf.as_ptr() as *const UdpHeader) };
        let length = u16::from_be(hdr.length) as usize;

        if length > buf.len() || length < UDP_HEADER_LEN {
            return Err("Invalid UDP datagram length field");
        }

        Ok(UdpDatagram {
            src_port: u16::from_be(hdr.src_port),
            dst_port: u16::from_be(hdr.dst_port),
            payload: &buf[UDP_HEADER_LEN..length],
        })
    }

    pub fn serialize(
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
        out: &mut [u8],
    ) -> Result<usize, &'static str> {
        let total_len = UDP_HEADER_LEN + payload.len();
        if out.len() < total_len {
            return Err("Output buffer too small for UDP datagram");
        }

        let hdr = UdpHeader {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            length: (total_len as u16).to_be(),
            checksum: 0, // Optional in IPv4, zeroed
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                &hdr as *const UdpHeader as *const u8,
                out.as_mut_ptr(),
                UDP_HEADER_LEN,
            );
        }

        out[UDP_HEADER_LEN..total_len].copy_from_slice(payload);
        Ok(total_len)
    }
}
