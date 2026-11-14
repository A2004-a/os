//! NovaOS Apex — Kernel Object Representation & Lifecycle Ownership
//! Every object carries explicit typed metadata, ownership, and reference counting.

use super::types::ObjectType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessObject {
    pub pid: u32,
    pub parent_pid: u32,
    pub name: [u8; 32],
    pub state: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadObject {
    pub tid: u32,
    pub pid: u32,
    pub priority: u8,
    pub state: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileObject {
    pub inode: u64,
    pub path: [u8; 64],
    pub size: u64,
    pub cursor: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SocketObject {
    pub domain: u16,
    pub sock_type: u16,
    pub port: u16,
    pub state: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventObject {
    pub event_id: u32,
    pub signaled: bool,
    pub auto_reset: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceObject {
    pub device_id: u32,
    pub vendor_id: u16,
    pub device_type: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowObject {
    pub window_id: u32,
    pub owner_pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedMemoryObject {
    pub shm_id: u32,
    pub size_bytes: usize,
    pub phys_base: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectPayload {
    Process(ProcessObject),
    Thread(ThreadObject),
    File(FileObject),
    Socket(SocketObject),
    Event(EventObject),
    Device(DeviceObject),
    Window(WindowObject),
    SharedMemory(SharedMemoryObject),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelObject {
    pub id: u64,
    pub obj_type: ObjectType,
    pub ref_count: u32,
    pub payload: ObjectPayload,
}

impl KernelObject {
    pub fn new(id: u64, payload: ObjectPayload) -> Self {
        let obj_type = match payload {
            ObjectPayload::Process(_) => ObjectType::Process,
            ObjectPayload::Thread(_) => ObjectType::Thread,
            ObjectPayload::File(_) => ObjectType::File,
            ObjectPayload::Socket(_) => ObjectType::Socket,
            ObjectPayload::Event(_) => ObjectType::Event,
            ObjectPayload::Device(_) => ObjectType::Device,
            ObjectPayload::Window(_) => ObjectType::Window,
            ObjectPayload::SharedMemory(_) => ObjectType::SharedMemory,
        };

        Self {
            id,
            obj_type,
            ref_count: 1,
            payload,
        }
    }
}
