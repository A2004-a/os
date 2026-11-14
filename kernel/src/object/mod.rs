//! NovaOS Zero — Kernel Object Model & Lifecycle Management
//! Every system resource has typed identity, reference-counted lifetime, and leak tracking.

pub mod handle;

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::serial_println;

pub use handle::{HandleId, HandleTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelObjectType {
    Process,
    Thread,
    File,
    Device,
    Socket,
    SharedMemory,
    Event,
    Channel,
    Window,
}

static TOTAL_OBJECTS_CREATED: AtomicU64 = AtomicU64::new(0);
static ACTIVE_OBJECTS: AtomicU64 = AtomicU64::new(0);

pub struct KernelObject {
    pub id: u64,
    pub object_type: KernelObjectType,
    ref_count: AtomicU32,
    pub payload: u64, // Generic resource descriptor or physical frame address
}

impl KernelObject {
    pub fn new(object_type: KernelObjectType, payload: u64) -> Self {
        let id = TOTAL_OBJECTS_CREATED.fetch_add(1, Ordering::Relaxed) + 1;
        ACTIVE_OBJECTS.fetch_add(1, Ordering::Relaxed);

        KernelObject {
            id,
            object_type,
            ref_count: AtomicU32::new(1),
            payload,
        }
    }

    /// Increment reference count (e.g. handle duplicated or passed across IPC)
    pub fn retain(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Decrement reference count; returns true if object should be destroyed
    pub fn release(&self) -> bool {
        let prev = self.ref_count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            ACTIVE_OBJECTS.fetch_sub(1, Ordering::Relaxed);
            self.destroy();
            true
        } else {
            false
        }
    }

    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Relaxed)
    }

    /// Trigger resource-specific cleanup when reference count drops to zero
    fn destroy(&self) {
        serial_println!(
            "[OBJECT] Destroyed {:?} (id: {}, payload: 0x{:X})",
            self.object_type, self.id, self.payload
        );
    }
}

pub fn active_object_count() -> u64 {
    ACTIVE_OBJECTS.load(Ordering::Relaxed)
}

pub fn total_objects_created() -> u64 {
    TOTAL_OBJECTS_CREATED.load(Ordering::Relaxed)
}
