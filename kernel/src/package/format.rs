//! NovaOS Apex — Signed Nova Package (.nvp) Binary Format
//! Defines the container header, capability permissions bitmask, payload structures,
//! builder utilities, and validation routines.

extern crate alloc;
use alloc::vec::Vec;
use core::mem::size_of;

use super::crypto::{sha256, sign_digest, verify_signature};

pub const NVP_MAGIC: &[u8; 8] = b"NOVA_NVP";
pub const NVP_FORMAT_VERSION: u32 = 1;
pub const NVP_HEADER_SIZE: u32 = 192;
pub const NVP_SIGNATURE_SIZE: u32 = 64;

// Capability Permissions Bitmask
pub const CAP_FS_READ: u32 = 1 << 0;
pub const CAP_FS_WRITE: u32 = 1 << 1;
pub const CAP_NET_SOCKET: u32 = 1 << 2;
pub const CAP_NET_BIND: u32 = 1 << 3;
pub const CAP_GUI_WINDOW: u32 = 1 << 4;
pub const CAP_PROC_SPAWN: u32 = 1 << 5;
pub const CAP_SYS_ADMIN: u32 = 1 << 6;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackageError {
    InvalidMagic,
    UnsupportedVersion,
    CorruptedHeader,
    ChecksumMismatch,
    SignatureInvalid,
    BufferTooSmall,
    FileNotFound,
    PermissionDenied,
    AlreadyInstalled,
    StagingFailed,
    RollbackFailed,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NvpHeader {
    pub magic: [u8; 8],
    pub format_version: u32,
    pub name: [u8; 32],
    pub version: [u8; 16],
    pub publisher: [u8; 32],
    pub capabilities: u32,
    pub header_size: u32,
    pub payload_offset: u32,
    pub payload_size: u32,
    pub signature_offset: u32,
    pub signature_size: u32,
    pub sha256_hash: [u8; 32],
    pub reserved: [u8; 44],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NvpFileEntry {
    pub path: [u8; 64],
    pub mode: u32,
    pub file_size: u32,
    pub data_offset: u32,
}

impl NvpHeader {
    pub fn name_as_str(&self) -> &str {
        let mut len = 0;
        while len < self.name.len() && self.name[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.name[..len]).unwrap_or("unknown")
    }

    pub fn version_as_str(&self) -> &str {
        let mut len = 0;
        while len < self.version.len() && self.version[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.version[..len]).unwrap_or("0.0.0")
    }

    pub fn publisher_as_str(&self) -> &str {
        let mut len = 0;
        while len < self.publisher.len() && self.publisher[len] != 0 {
            len += 1;
        }
        core::str::from_utf8(&self.publisher[..len]).unwrap_or("unknown")
    }

    pub fn has_capability(&self, cap: u32) -> bool {
        (self.capabilities & cap) == cap
    }
}

/// Builds an in-memory `.nvp` binary package from components, signing it with the secret key.
pub fn build_nvp(
    name: &str,
    version: &str,
    publisher: &str,
    capabilities: u32,
    files: &[(&str, u32, &[u8])],
    secret_key: &[u8; 32],
) -> Vec<u8> {
    // 1. Serialize payload: [num_files: u32] + [NvpFileEntry * num] + [file_data...]
    let mut payload = Vec::new();
    let num_files = files.len() as u32;
    payload.extend_from_slice(&num_files.to_le_bytes());

    let entry_table_size = files.len() * size_of::<NvpFileEntry>();
    let mut current_data_offset = 4 + entry_table_size as u32;

    let mut entries = Vec::with_capacity(files.len());
    for &(path, mode, data) in files {
        let mut entry = NvpFileEntry {
            path: [0u8; 64],
            mode,
            file_size: data.len() as u32,
            data_offset: current_data_offset,
        };
        let b = path.as_bytes();
        let copy_len = b.len().min(63);
        entry.path[..copy_len].copy_from_slice(&b[..copy_len]);
        entries.push(entry);
        current_data_offset += data.len() as u32;
    }

    // Write file entries
    for entry in &entries {
        let bytes: [u8; size_of::<NvpFileEntry>()] = unsafe { core::mem::transmute_copy(entry) };
        payload.extend_from_slice(&bytes);
    }

    // Write file data
    for &(_, _, data) in files {
        payload.extend_from_slice(data);
    }

    // 2. Compute payload SHA-256
    let payload_hash = sha256(&payload);

    // 3. Compute digital signature
    let signature = sign_digest(secret_key, &payload_hash);

    // 4. Construct Header
    let mut hdr = NvpHeader {
        magic: *NVP_MAGIC,
        format_version: NVP_FORMAT_VERSION,
        name: [0u8; 32],
        version: [0u8; 16],
        publisher: [0u8; 32],
        capabilities,
        header_size: NVP_HEADER_SIZE,
        payload_offset: NVP_HEADER_SIZE,
        payload_size: payload.len() as u32,
        signature_offset: NVP_HEADER_SIZE + payload.len() as u32,
        signature_size: NVP_SIGNATURE_SIZE,
        sha256_hash: payload_hash,
        reserved: [0u8; 44],
    };

    let name_b = name.as_bytes();
    let name_len = name_b.len().min(31);
    hdr.name[..name_len].copy_from_slice(&name_b[..name_len]);

    let ver_b = version.as_bytes();
    let ver_len = ver_b.len().min(15);
    hdr.version[..ver_len].copy_from_slice(&ver_b[..ver_len]);

    let pub_b = publisher.as_bytes();
    let pub_len = pub_b.len().min(31);
    hdr.publisher[..pub_len].copy_from_slice(&pub_b[..pub_len]);

    // 5. Assemble complete package
    let mut pkg_bytes = Vec::with_capacity((NVP_HEADER_SIZE as usize) + payload.len() + (NVP_SIGNATURE_SIZE as usize));
    let hdr_bytes: [u8; size_of::<NvpHeader>()] = unsafe { core::mem::transmute_copy(&hdr) };
    pkg_bytes.extend_from_slice(&hdr_bytes);
    pkg_bytes.extend_from_slice(&payload);
    pkg_bytes.extend_from_slice(&signature);

    pkg_bytes
}

/// Parses and validates `.nvp` header and digital signature.
pub fn verify_nvp(pkg_bytes: &[u8], public_key: &[u8; 32]) -> Result<NvpHeader, PackageError> {
    if pkg_bytes.len() < NVP_HEADER_SIZE as usize {
        return Err(PackageError::BufferTooSmall);
    }

    let hdr: NvpHeader = unsafe {
        core::ptr::read_unaligned(pkg_bytes.as_ptr() as *const NvpHeader)
    };

    if &hdr.magic != NVP_MAGIC {
        return Err(PackageError::InvalidMagic);
    }
    if hdr.format_version != NVP_FORMAT_VERSION {
        return Err(PackageError::UnsupportedVersion);
    }
    if hdr.header_size != NVP_HEADER_SIZE {
        return Err(PackageError::CorruptedHeader);
    }

    let payload_end = (hdr.payload_offset as usize) + (hdr.payload_size as usize);
    let sig_end = (hdr.signature_offset as usize) + (hdr.signature_size as usize);

    if pkg_bytes.len() < sig_end || payload_end > pkg_bytes.len() {
        return Err(PackageError::BufferTooSmall);
    }

    // 1. Verify payload hash
    let payload = &pkg_bytes[hdr.payload_offset as usize..payload_end];
    let computed_hash = sha256(payload);
    if computed_hash != hdr.sha256_hash {
        return Err(PackageError::ChecksumMismatch);
    }

    // 2. Verify digital signature
    let sig_bytes = &pkg_bytes[hdr.signature_offset as usize..sig_end];
    let mut sig = [0u8; 64];
    sig.copy_from_slice(sig_bytes);

    if !verify_signature(public_key, &hdr.sha256_hash, &sig) {
        return Err(PackageError::SignatureInvalid);
    }

    Ok(hdr)
}
