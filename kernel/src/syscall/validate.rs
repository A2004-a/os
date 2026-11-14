//! NovaOS Apex — System Call Pointer & Parameter Validation Engine
//! Verifies that all userspace addresses, buffer lengths, and strings reside strictly in canonical user memory.

pub const USER_SPACE_LIMIT: u64 = 0x0000_8000_0000_0000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SyscallError {
    BadPointer,
    BufferOverflow,
    KernelTamperingAttempt,
    InvalidString,
    BufferTooLarge,
    BadAlignment,
    InvalidHandle,
    PermissionDenied,
    InvalidArgument,
}

/// Validates that a user-supplied buffer pointer and length reside strictly in userspace.
pub fn validate_user_buffer(ptr: u64, len: usize, _writable: bool) -> Result<(), SyscallError> {
    if ptr == 0 {
        return Err(SyscallError::BadPointer);
    }

    let end_addr = ptr.checked_add(len as u64).ok_or(SyscallError::BufferOverflow)?;

    if end_addr > USER_SPACE_LIMIT {
        return Err(SyscallError::KernelTamperingAttempt);
    }

    Ok(())
}

/// Validates that a user string pointer is valid, non-null, in userspace, and null-terminated within max_len.
pub fn validate_user_string(ptr: u64, max_len: usize) -> Result<&'static str, SyscallError> {
    if ptr == 0 {
        return Err(SyscallError::BadPointer);
    }

    let byte_ptr = ptr as *const u8;
    let mut len = 0;

    while len < max_len {
        let addr = ptr + (len as u64);
        if addr >= USER_SPACE_LIMIT {
            return Err(SyscallError::KernelTamperingAttempt);
        }

        let byte = unsafe { *byte_ptr.add(len) };
        if byte == 0 {
            let slice = unsafe { core::slice::from_raw_parts(byte_ptr, len) };
            return core::str::from_utf8(slice).map_err(|_| SyscallError::InvalidString);
        }
        len += 1;
    }

    Err(SyscallError::InvalidString)
}
