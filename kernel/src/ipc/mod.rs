//! Inter-Process Communication (IPC) Primitives

pub struct IpcMessage {
    pub sender_pid: u64,
    pub data_len: usize,
    pub payload: [u8; 64],
}

pub struct Channel {
    messages: [Option<IpcMessage>; 8],
    head: usize,
    tail: usize,
}

impl Channel {
    pub const fn new() -> Self {
        const INIT_MSG: Option<IpcMessage> = None;
        Channel {
            messages: [INIT_MSG; 8],
            head: 0,
            tail: 0,
        }
    }

    pub fn send(&mut self, msg: IpcMessage) -> Result<(), &'static str> {
        let next_head = (self.head + 1) % 8;
        if next_head == self.tail {
            return Err("Channel queue is full");
        }
        self.messages[self.head] = Some(msg);
        self.head = next_head;
        Ok(())
    }

    pub fn receive(&mut self) -> Option<IpcMessage> {
        if self.head == self.tail {
            return None;
        }
        let msg = self.messages[self.tail].take();
        self.tail = (self.tail + 1) % 8;
        msg
    }
}
