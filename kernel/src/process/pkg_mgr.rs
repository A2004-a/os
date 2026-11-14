//! NovaOS Zero — Application Package Manager & Sandboxed App Model
//! Implements the .npk (NovaApp Package) format, capability verification,
//! and the sandboxed package registry.

pub const NPKG_MAGIC: u32 = 0x4E50_4B47; // 'NPKG'
pub const NPKG_VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 64;

// Sandbox capability bitmask flags
pub const CAP_NONE: u32     = 0;
pub const CAP_NET: u32      = 1 << 0; // Network socket and packet access
pub const CAP_STORAGE: u32  = 1 << 1; // Direct disk / VFS write access
pub const CAP_IPC: u32      = 1 << 2; // Inter-process communication
pub const CAP_GRAPHICS: u32 = 1 << 3; // Direct framebuffer / Window Server access
pub const CAP_AUDIO: u32    = 1 << 4; // Audio hardware output
pub const CAP_ADMIN: u32    = 1 << 5; // Elevated microkernel privileges
pub const CAP_ALL: u32      = 0x3F;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageHeader {
    pub magic: u32,             // 'NPKG'
    pub format_version: u16,    // 1
    pub capabilities: u32,      // Bitmask of requested capabilities
    pub manifest_offset: u32,   // Offset to manifest
    pub manifest_size: u32,     // Manifest size in bytes
    pub payload_offset: u32,    // Offset to executable payload
    pub payload_size: u32,      // Payload size in bytes
    pub checksum: u32,          // Adler-32 checksum of manifest + payload
    pub reserved: [u8; 34],     // Padding to 64 bytes
}

impl PackageHeader {
    pub fn new(
        capabilities: u32,
        manifest_offset: u32,
        manifest_size: u32,
        payload_offset: u32,
        payload_size: u32,
        checksum: u32,
    ) -> Self {
        PackageHeader {
            magic: NPKG_MAGIC,
            format_version: NPKG_VERSION,
            capabilities,
            manifest_offset,
            manifest_size,
            payload_offset,
            payload_size,
            checksum,
            reserved: [0; 34],
        }
    }

    pub fn get_magic(&self) -> u32 { self.magic }
    pub fn get_version(&self) -> u16 { self.format_version }
    pub fn get_capabilities(&self) -> u32 { self.capabilities }
    pub fn get_manifest_offset(&self) -> u32 { self.manifest_offset }
    pub fn get_manifest_size(&self) -> u32 { self.manifest_size }
    pub fn get_payload_offset(&self) -> u32 { self.payload_offset }
    pub fn get_payload_size(&self) -> u32 { self.payload_size }
    pub fn get_checksum(&self) -> u32 { self.checksum }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageError {
    InvalidMagic,
    UnsupportedVersion,
    BufferTooSmall,
    CorruptedChecksum,
    CapabilityViolation,
    PackageNotFound,
    RegistryFull,
    AlreadyInstalled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageManifest {
    pub app_id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub exec_entry: &'static str,
    pub required_caps: u32,
}

impl PackageManifest {
    pub const fn new(
        app_id: &'static str,
        name: &'static str,
        version: &'static str,
        author: &'static str,
        exec_entry: &'static str,
        required_caps: u32,
    ) -> Self {
        PackageManifest {
            app_id,
            name,
            version,
            author,
            exec_entry,
            required_caps,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstalledPackage {
    pub manifest: PackageManifest,
    pub payload_size: usize,
    pub is_active: bool,
    pub launch_count: u32,
}

pub const MAX_INSTALLED_PACKAGES: usize = 16;

/// Calculate Adler-32 checksum for package data integrity
pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Verify package binary bytes, validating header, offsets, and Adler-32 integrity
pub fn verify_package_bytes(bytes: &[u8]) -> Result<(PackageHeader, &[u8], &[u8]), PackageError> {
    if bytes.len() < HEADER_SIZE {
        return Err(PackageError::BufferTooSmall);
    }

    let header = unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const PackageHeader) };

    if header.get_magic() != NPKG_MAGIC {
        return Err(PackageError::InvalidMagic);
    }
    if header.get_version() != NPKG_VERSION {
        return Err(PackageError::UnsupportedVersion);
    }

    let m_off = header.get_manifest_offset() as usize;
    let m_sz = header.get_manifest_size() as usize;
    let p_off = header.get_payload_offset() as usize;
    let p_sz = header.get_payload_size() as usize;

    if m_off + m_sz > bytes.len() || p_off + p_sz > bytes.len() {
        return Err(PackageError::BufferTooSmall);
    }

    let manifest_slice = &bytes[m_off..m_off + m_sz];
    let payload_slice = &bytes[p_off..p_off + p_sz];

    // Check integrity: checksum covers manifest and payload sequentially
    let mut combined_a: u32 = 1;
    let mut combined_b: u32 = 0;
    for &byte in manifest_slice {
        combined_a = (combined_a + byte as u32) % 65521;
        combined_b = (combined_b + combined_a) % 65521;
    }
    for &byte in payload_slice {
        combined_a = (combined_a + byte as u32) % 65521;
        combined_b = (combined_b + combined_a) % 65521;
    }
    let computed_checksum = (combined_b << 16) | combined_a;

    if computed_checksum != header.get_checksum() {
        return Err(PackageError::CorruptedChecksum);
    }

    Ok((header, manifest_slice, payload_slice))
}

pub struct PackageManager {
    pub packages: [Option<InstalledPackage>; MAX_INSTALLED_PACKAGES],
    pub system_capability_ceiling: u32,
}

impl PackageManager {
    pub const fn new(capability_ceiling: u32) -> Self {
        const INIT_PKG: Option<InstalledPackage> = None;
        PackageManager {
            packages: [INIT_PKG; MAX_INSTALLED_PACKAGES],
            system_capability_ceiling: capability_ceiling,
        }
    }

    /// Install a package after validating requested capabilities against system ceiling
    pub fn install(&mut self, manifest: PackageManifest, payload_size: usize) -> Result<(), PackageError> {
        // Enforce capability ceiling
        if (manifest.required_caps & !self.system_capability_ceiling) != 0 {
            return Err(PackageError::CapabilityViolation);
        }

        // Check if already installed
        for slot in self.packages.iter() {
            if let Some(ref pkg) = slot {
                if pkg.manifest.app_id == manifest.app_id {
                    return Err(PackageError::AlreadyInstalled);
                }
            }
        }

        // Find free slot
        for slot in self.packages.iter_mut() {
            if slot.is_none() {
                *slot = Some(InstalledPackage {
                    manifest,
                    payload_size,
                    is_active: true,
                    launch_count: 0,
                });
                return Ok(());
            }
        }

        Err(PackageError::RegistryFull)
    }

    /// Uninstall a package by app_id
    pub fn uninstall(&mut self, app_id: &str) -> Result<(), PackageError> {
        for slot in self.packages.iter_mut() {
            if let Some(ref pkg) = slot {
                if pkg.manifest.app_id == app_id {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(PackageError::PackageNotFound)
    }

    /// Find an installed package by app_id
    pub fn find(&self, app_id: &str) -> Option<&InstalledPackage> {
        for slot in self.packages.iter() {
            if let Some(ref pkg) = slot {
                if pkg.manifest.app_id == app_id {
                    return Some(pkg);
                }
            }
        }
        None
    }

    /// Launch an application, incrementing its launch counter
    pub fn launch(&mut self, app_id: &str) -> Result<u32, PackageError> {
        for slot in self.packages.iter_mut() {
            if let Some(ref mut pkg) = slot {
                if pkg.manifest.app_id == app_id {
                    pkg.launch_count += 1;
                    return Ok(pkg.launch_count);
                }
            }
        }
        Err(PackageError::PackageNotFound)
    }

    /// Count active installed packages
    pub fn count(&self) -> usize {
        self.packages.iter().filter(|p| p.is_some()).count()
    }
}
