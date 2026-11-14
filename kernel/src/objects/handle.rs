//! NovaOS Apex — Typed Handle Table & Capability-Based Access Control
//! Manages typed handles with fine-grained capability checks and reference counted lifecycles.

use super::object::{KernelObject, ObjectPayload};
use super::types::{CapabilityRights, ObjectType};
use core::sync::atomic::{AtomicU64, Ordering};

pub const MAX_HANDLES: usize = 256;
pub const MAX_OBJECTS: usize = 256;

static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Handle(pub u32);

impl Handle {
    pub const INVALID: Self = Self(0);
}

#[derive(Clone, Copy, Debug)]
pub struct HandleEntry {
    pub object_idx: usize,
    pub obj_type: ObjectType,
    pub rights: CapabilityRights,
    pub in_use: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HandleError {
    InvalidHandle,
    TypeMismatch,
    AccessDenied,
    TableFull,
    ObjectStorageFull,
}

pub struct HandleTable {
    handles: [HandleEntry; MAX_HANDLES],
    objects: [Option<KernelObject>; MAX_OBJECTS],
    active_handles_count: usize,
}

impl HandleTable {
    pub const fn new() -> Self {
        const EMPTY_ENTRY: HandleEntry = HandleEntry {
            object_idx: 0,
            obj_type: ObjectType::Process,
            rights: CapabilityRights::NONE,
            in_use: false,
        };

        const EMPTY_OBJ: Option<KernelObject> = None;

        Self {
            handles: [EMPTY_ENTRY; MAX_HANDLES],
            objects: [EMPTY_OBJ; MAX_OBJECTS],
            active_handles_count: 0,
        }
    }

    pub fn insert_object(
        &mut self,
        payload: ObjectPayload,
        rights: CapabilityRights,
    ) -> Result<Handle, HandleError> {
        // 1. Find free object slot
        let mut obj_slot = None;
        for i in 0..MAX_OBJECTS {
            if self.objects[i].is_none() {
                obj_slot = Some(i);
                break;
            }
        }
        let obj_idx = obj_slot.ok_or(HandleError::ObjectStorageFull)?;

        // 2. Find free handle slot (Handle 0 is reserved as INVALID, so 1..MAX_HANDLES)
        let mut handle_slot = None;
        for i in 1..MAX_HANDLES {
            if !self.handles[i].in_use {
                handle_slot = Some(i);
                break;
            }
        }
        let handle_idx = handle_slot.ok_or(HandleError::TableFull)?;

        let obj_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::SeqCst);
        let obj = KernelObject::new(obj_id, payload);
        let obj_type = obj.obj_type;

        self.objects[obj_idx] = Some(obj);
        self.handles[handle_idx] = HandleEntry {
            object_idx: obj_idx,
            obj_type,
            rights,
            in_use: true,
        };
        self.active_handles_count += 1;

        Ok(Handle(handle_idx as u32))
    }

    pub fn get(
        &self,
        handle: Handle,
        expected_type: ObjectType,
        required_rights: CapabilityRights,
    ) -> Result<&KernelObject, HandleError> {
        let idx = handle.0 as usize;
        if idx == 0 || idx >= MAX_HANDLES || !self.handles[idx].in_use {
            return Err(HandleError::InvalidHandle);
        }

        let entry = &self.handles[idx];
        if entry.obj_type != expected_type {
            return Err(HandleError::TypeMismatch);
        }

        if !entry.rights.contains(required_rights) {
            return Err(HandleError::AccessDenied);
        }

        self.objects[entry.object_idx]
            .as_ref()
            .ok_or(HandleError::InvalidHandle)
    }

    pub fn duplicate(
        &mut self,
        handle: Handle,
        rights_mask: CapabilityRights,
    ) -> Result<Handle, HandleError> {
        let src_idx = handle.0 as usize;
        if src_idx == 0 || src_idx >= MAX_HANDLES || !self.handles[src_idx].in_use {
            return Err(HandleError::InvalidHandle);
        }

        let entry = self.handles[src_idx];
        if !entry.rights.contains(CapabilityRights::DUPLICATE) {
            return Err(HandleError::AccessDenied);
        }

        // Find free handle slot
        let mut target_slot = None;
        for i in 1..MAX_HANDLES {
            if !self.handles[i].in_use {
                target_slot = Some(i);
                break;
            }
        }
        let target_idx = target_slot.ok_or(HandleError::TableFull)?;

        let obj = self.objects[entry.object_idx]
            .as_mut()
            .ok_or(HandleError::InvalidHandle)?;
        obj.ref_count += 1;

        let new_rights = entry.rights.intersect(rights_mask);
        self.handles[target_idx] = HandleEntry {
            object_idx: entry.object_idx,
            obj_type: entry.obj_type,
            rights: new_rights,
            in_use: true,
        };
        self.active_handles_count += 1;

        Ok(Handle(target_idx as u32))
    }

    pub fn close(&mut self, handle: Handle) -> Result<(), HandleError> {
        let idx = handle.0 as usize;
        if idx == 0 || idx >= MAX_HANDLES || !self.handles[idx].in_use {
            return Err(HandleError::InvalidHandle);
        }

        let obj_idx = self.handles[idx].object_idx;
        self.handles[idx].in_use = false;
        self.active_handles_count = self.active_handles_count.saturating_sub(1);

        if let Some(ref mut obj) = self.objects[obj_idx] {
            obj.ref_count = obj.ref_count.saturating_sub(1);
            if obj.ref_count == 0 {
                // All handles closed: reclaim object
                self.objects[obj_idx] = None;
            }
        }

        Ok(())
    }

    pub fn active_handles(&self) -> usize {
        self.active_handles_count
    }
}
