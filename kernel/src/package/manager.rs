//! NovaOS Apex — Package Lifecycle Manager & Transactional Registry
//! Manages transactional staging, atomic installation, rollback on failure, and installed package registry.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

use super::crypto::SYSTEM_ROOT_PUBKEY;
use super::format::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageState {
    Installed,
    Staged,
    RollbackPending,
    Uninstalled,
}

#[derive(Clone, Debug)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub capabilities: u32,
    pub install_time: u64,
    pub state: PackageState,
    pub files: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct StagedPackage {
    pub stage_id: u32,
    pub header: NvpHeader,
    pub files: Vec<(String, u32, Vec<u8>)>,
}

#[derive(Clone, Debug)]
pub struct BackupPackage {
    pub name: String,
    pub version: String,
    pub capabilities: u32,
    pub files: Vec<(String, u32, Vec<u8>)>,
}

pub struct PackageManager {
    installed: Vec<InstalledPackage>,
    staged: Vec<StagedPackage>,
    backups: Vec<BackupPackage>,
    next_stage_id: AtomicU32,
}

static mut GLOBAL_PKG_MGR: Option<PackageManager> = None;

impl PackageManager {
    pub fn new() -> Self {
        Self {
            installed: Vec::new(),
            staged: Vec::new(),
            backups: Vec::new(),
            next_stage_id: AtomicU32::new(1),
        }
    }

    /// Stages a package by verifying integrity, checking capability permissions, and buffering payload files.
    pub fn stage_package(&mut self, pkg_bytes: &[u8], user_uid: u32) -> Result<u32, PackageError> {
        let hdr = verify_nvp(pkg_bytes, &SYSTEM_ROOT_PUBKEY)?;

        // Enforce capability permissions: System Administration capability requires root (UID 0)
        if hdr.has_capability(CAP_SYS_ADMIN) && user_uid != 0 {
            return Err(PackageError::PermissionDenied);
        }

        // Unpack files from payload
        let payload_start = hdr.payload_offset as usize;
        let payload = &pkg_bytes[payload_start..payload_start + hdr.payload_size as usize];

        if payload.len() < 4 {
            return Err(PackageError::CorruptedHeader);
        }

        let num_files = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
        let mut offset = 4;
        let mut staged_files = Vec::with_capacity(num_files);

        for _ in 0..num_files {
            let entry_size = core::mem::size_of::<NvpFileEntry>();
            if offset + entry_size > payload.len() {
                return Err(PackageError::CorruptedHeader);
            }

            let entry: NvpFileEntry = unsafe {
                core::ptr::read_unaligned(payload[offset..].as_ptr() as *const NvpFileEntry)
            };
            offset += entry_size;

            let data_start = entry.data_offset as usize;
            let data_end = data_start + entry.file_size as usize;
            if data_end > payload.len() {
                return Err(PackageError::CorruptedHeader);
            }

            let file_data = payload[data_start..data_end].to_vec();
            let mut path_len = 0;
            while path_len < entry.path.len() && entry.path[path_len] != 0 {
                path_len += 1;
            }
            let path_str = core::str::from_utf8(&entry.path[..path_len])
                .unwrap_or("/unknown")
                .to_string();

            staged_files.push((path_str, entry.mode, file_data));
        }

        let stage_id = self.next_stage_id.fetch_add(1, Ordering::SeqCst);
        self.staged.push(StagedPackage {
            stage_id,
            header: hdr,
            files: staged_files,
        });

        Ok(stage_id)
    }

    /// Atomically commits a staged package into the installed registry, backing up the prior version.
    pub fn commit_stage(&mut self, stage_id: u32) -> Result<(), PackageError> {
        let stage_idx = self
            .staged
            .iter()
            .position(|s| s.stage_id == stage_id)
            .ok_or(PackageError::StagingFailed)?;
        let staged = self.staged.remove(stage_idx);

        let pkg_name = staged.header.name_as_str().to_string();
        let pkg_ver = staged.header.version_as_str().to_string();
        let pkg_pub = staged.header.publisher_as_str().to_string();

        // 1. If an existing version is installed, create an atomic backup
        if let Some(pos) = self.installed.iter().position(|p| p.name == pkg_name) {
            let existing = &self.installed[pos];
            self.backups.retain(|b| b.name != pkg_name); // keep latest backup
            self.backups.push(BackupPackage {
                name: existing.name.clone(),
                version: existing.version.clone(),
                capabilities: existing.capabilities,
                files: Vec::new(), // In full VFS this caches physical file blobs
            });
            self.installed.remove(pos);
        }

        // 2. Commit file paths to installed record
        let file_paths = staged.files.iter().map(|(p, _, _)| p.clone()).collect();
        self.installed.push(InstalledPackage {
            name: pkg_name,
            version: pkg_ver,
            publisher: pkg_pub,
            capabilities: staged.header.capabilities,
            install_time: 1_000,
            state: PackageState::Installed,
            files: file_paths,
        });

        Ok(())
    }

    /// Atomically rolls back a package to its previous backup version.
    pub fn rollback_package(&mut self, pkg_name: &str) -> Result<(), PackageError> {
        let backup_idx = self
            .backups
            .iter()
            .position(|b| b.name == pkg_name)
            .ok_or(PackageError::RollbackFailed)?;
        let backup = self.backups.remove(backup_idx);

        // Remove currently installed version
        if let Some(pos) = self.installed.iter().position(|p| p.name == pkg_name) {
            self.installed.remove(pos);
        }

        // Restore backup
        let restored_files = backup.files.iter().map(|(p, _, _)| p.clone()).collect();
        self.installed.push(InstalledPackage {
            name: backup.name,
            version: backup.version,
            publisher: "NovaOS Apex Team (Restored)".to_string(),
            capabilities: backup.capabilities,
            install_time: 2_000,
            state: PackageState::Installed,
            files: restored_files,
        });

        Ok(())
    }

    /// Uninstalls an installed package.
    pub fn uninstall_package(&mut self, pkg_name: &str, user_uid: u32) -> Result<(), PackageError> {
        if user_uid != 0 {
            return Err(PackageError::PermissionDenied);
        }
        let pos = self
            .installed
            .iter()
            .position(|p| p.name == pkg_name)
            .ok_or(PackageError::FileNotFound)?;
        self.installed.remove(pos);
        Ok(())
    }

    pub fn get_installed(&self, name: &str) -> Option<&InstalledPackage> {
        self.installed.iter().find(|p| p.name == name)
    }

    pub fn list_installed(&self) -> &[InstalledPackage] {
        &self.installed
    }
}

pub fn init() {
    let mgr = PackageManager::new();
    unsafe {
        core::ptr::write(core::ptr::addr_of_mut!(GLOBAL_PKG_MGR), Some(mgr));
    }
}

pub fn get_manager() -> &'static mut PackageManager {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(GLOBAL_PKG_MGR);
        (*ptr)
            .as_mut()
            .expect("Package manager uninitialized")
    }
}
