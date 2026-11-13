//! NovaOS Security Model & Capability Handles
//! Enforces least-privilege resource access and userland pointer validation

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CapabilityFlags: u32 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXEC    = 1 << 2;
        const SPAWN   = 1 << 3;
        const NETWORK = 1 << 4;
        const STORAGE = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CapabilityHandle {
    pub id: u64,
    pub permissions: CapabilityFlags,
}

impl CapabilityHandle {
    pub fn new(id: u64, permissions: CapabilityFlags) -> Self {
        CapabilityHandle { id, permissions }
    }

    pub fn has_permission(&self, flag: CapabilityFlags) -> bool {
        self.permissions.contains(flag)
    }
}

/// Validates that a user-supplied pointer and buffer length reside strictly within
/// canonical lower-half user address space.
pub fn validate_user_buffer(ptr: *const u8, len: usize) -> bool {
    let addr = ptr as u64;
    let end_addr = match addr.checked_add(len as u64) {
        Some(a) => a,
        None => return false, // Overflow detected
    };

    // Userland boundary: 0x0000_7FFF_FFFF_FFFF
    end_addr <= 0x0000_7FFF_FFFF_FFFF
}
