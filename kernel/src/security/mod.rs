//! NovaOS Security Model & Capability Handles
//! Enforces least-privilege resource access and userland pointer validation

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityFlags(pub u32);

impl CapabilityFlags {
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXEC: Self = Self(1 << 2);
    pub const SPAWN: Self = Self(1 << 3);
    pub const NETWORK: Self = Self(1 << 4);
    pub const STORAGE: Self = Self(1 << 5);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for CapabilityFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for CapabilityFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
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
/// canonical lower-half user address space, rejecting null pointers and zero-page references.
pub fn validate_user_buffer(ptr: *const u8, len: usize) -> bool {
    let addr = ptr as u64;
    if addr < 0x1000 {
        return false; // Null pointer or zero-page protection
    }
    let end_addr = match addr.checked_add(len as u64) {
        Some(a) => a,
        None => return false, // Arithmetic overflow detected
    };

    // Userland canonical boundary: strictly <= 0x0000_7FFF_FFFF_FFFF
    end_addr <= 0x0000_7FFF_FFFF_FFFF
}
