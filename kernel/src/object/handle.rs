//! NovaOS Zero — Per-Process Typed Handle Table & Capability Check

use crate::security::CapabilityFlags;

pub type HandleId = u32;
pub const MAX_HANDLES: usize = 64;

#[derive(Clone, Copy)]
pub struct HandleEntry {
    pub object_id: u64,
    pub permissions: CapabilityFlags,
    pub valid: bool,
}

impl HandleEntry {
    pub const fn empty() -> Self {
        Self {
            object_id: 0,
            permissions: CapabilityFlags(0),
            valid: false,
        }
    }
}

pub struct HandleTable {
    entries: [HandleEntry; MAX_HANDLES],
    active_count: usize,
}

impl HandleTable {
    pub const fn new() -> Self {
        Self {
            entries: [HandleEntry::empty(); MAX_HANDLES],
            active_count: 0,
        }
    }

    /// Allocate a handle slot for a kernel object with granted capabilities
    pub fn allocate(&mut self, object_id: u64, permissions: CapabilityFlags) -> Result<HandleId, &'static str> {
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            if !entry.valid {
                entry.object_id = object_id;
                entry.permissions = permissions;
                entry.valid = true;
                self.active_count += 1;
                return Ok(idx as HandleId);
            }
        }
        Err("Process handle table full")
    }

    /// Retrieve handle entry and verify capability permission
    pub fn get(&self, handle: HandleId, required_perm: CapabilityFlags) -> Result<u64, &'static str> {
        let idx = handle as usize;
        if idx >= MAX_HANDLES {
            return Err("Invalid handle index");
        }

        let entry = &self.entries[idx];
        if !entry.valid {
            return Err("Handle is closed or invalid");
        }

        if !entry.permissions.contains(required_perm) {
            return Err("Permission denied for handle operation");
        }

        Ok(entry.object_id)
    }

    /// Close a handle and mark slot available
    pub fn close(&mut self, handle: HandleId) -> Result<u64, &'static str> {
        let idx = handle as usize;
        if idx >= MAX_HANDLES {
            return Err("Invalid handle index");
        }

        let entry = &mut self.entries[idx];
        if !entry.valid {
            return Err("Handle already closed");
        }

        let obj_id = entry.object_id;
        *entry = HandleEntry::empty();
        self.active_count -= 1;
        Ok(obj_id)
    }

    /// Close all handles during process termination
    pub fn close_all(&mut self) -> [u64; MAX_HANDLES] {
        let mut closed_objects = [0u64; MAX_HANDLES];
        let mut count = 0;

        for entry in self.entries.iter_mut() {
            if entry.valid {
                closed_objects[count] = entry.object_id;
                count += 1;
                *entry = HandleEntry::empty();
            }
        }
        self.active_count = 0;
        closed_objects
    }

    pub fn active_handles(&self) -> usize {
        self.active_count
    }
}
