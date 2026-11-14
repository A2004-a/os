//! NovaOS Apex — Inter-Process Communication (IPC) Subsystem
//! Implements structured message passing between Ring 3 processes with length checks and mailbox queues.

pub mod clipboard;

extern crate alloc;
use alloc::vec::Vec;
use crate::scheduler::sync::SpinLock;

pub const MAX_IPC_PAYLOAD_SIZE: usize = 256;
pub const MAX_INBOX_MESSAGES: usize = 32;

#[derive(Clone, Copy, Debug)]
pub struct IpcMessage {
    pub sender_pid: u32,
    pub dest_pid: u32,
    pub msg_type: u32,
    pub payload: [u8; MAX_IPC_PAYLOAD_SIZE],
    pub len: usize,
}

impl IpcMessage {
    pub fn new(sender_pid: u32, dest_pid: u32, msg_type: u32, data: &[u8]) -> Option<Self> {
        if data.len() > MAX_IPC_PAYLOAD_SIZE {
            return None;
        }

        let mut payload = [0u8; MAX_IPC_PAYLOAD_SIZE];
        payload[..data.len()].copy_from_slice(data);

        Some(Self {
            sender_pid,
            dest_pid,
            msg_type,
            payload,
            len: data.len(),
        })
    }
}

pub struct ProcessMailbox {
    pub pid: u32,
    pub messages: Vec<IpcMessage>,
}

impl ProcessMailbox {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: IpcMessage) -> bool {
        if self.messages.len() >= MAX_INBOX_MESSAGES {
            return false; // Mailbox full
        }
        self.messages.push(msg);
        true
    }

    pub fn pop_from(&mut self, src_pid: u32) -> Option<IpcMessage> {
        if src_pid == 0 {
            // Any sender
            if !self.messages.is_empty() {
                Some(self.messages.remove(0))
            } else {
                None
            }
        } else {
            // Filter by sender
            if let Some(pos) = self.messages.iter().position(|m| m.sender_pid == src_pid) {
                Some(self.messages.remove(pos))
            } else {
                None
            }
        }
    }
}

static IPC_REGISTRY: SpinLock<Option<[Option<ProcessMailbox>; 32]>> = SpinLock::new("ipc_registry", None);

pub fn init() {
    let mut reg = IPC_REGISTRY.lock();
    let mut mailboxes: [Option<ProcessMailbox>; 32] = [const { None }; 32];
    for i in 0..32 {
        mailboxes[i] = Some(ProcessMailbox::new(i as u32));
    }
    *reg = Some(mailboxes);
}

pub fn send(sender_pid: u32, dest_pid: u32, data: &[u8]) -> Result<(), i32> {
    if dest_pid >= 32 {
        return Err(-1); // Invalid PID
    }
    let msg = IpcMessage::new(sender_pid, dest_pid, 0, data).ok_or(-2)?; // Payload too large

    let mut reg = IPC_REGISTRY.lock();
    if let Some(mailboxes) = reg.as_mut() {
        if let Some(inbox) = &mut mailboxes[dest_pid as usize] {
            if inbox.push(msg) {
                Ok(())
            } else {
                Err(-3) // Mailbox full
            }
        } else {
            Err(-1)
        }
    } else {
        Err(-4)
    }
}

pub fn receive(receiver_pid: u32, src_pid: u32, out_buf: &mut [u8]) -> Result<usize, i32> {
    if receiver_pid >= 32 {
        return Err(-1);
    }

    let mut reg = IPC_REGISTRY.lock();
    if let Some(mailboxes) = reg.as_mut() {
        if let Some(inbox) = &mut mailboxes[receiver_pid as usize] {
            if let Some(msg) = inbox.pop_from(src_pid) {
                let copy_len = msg.len.min(out_buf.len());
                out_buf[..copy_len].copy_from_slice(&msg.payload[..copy_len]);
                Ok(copy_len)
            } else {
                Err(-5) // No messages available (EAGAIN)
            }
        } else {
            Err(-1)
        }
    } else {
        Err(-4)
    }
}
