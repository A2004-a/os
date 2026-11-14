//! NovaOS Apex — Filesystem Subsystem
//! Combines VFS, RAMFS root, DevFS virtual devices, NovaFS persistent block store, and test suite.

extern crate alloc;
use alloc::sync::Arc;

pub mod devfs;
pub mod novafs;
pub mod path;
pub mod ramfs;
pub mod tests;
pub mod types;
pub mod vfs;

use crate::scheduler::sync::SpinLock;
use crate::serial;

pub fn init() {
    serial::write_str("[KERNEL] Initializing Virtual File System (VFS) and mounts...\n");

    // 1. Create RAMFS and populate hierarchy
    let mut ramfs = ramfs::RamFs::new();
    ramfs::populate_default_hierarchy(&mut ramfs);
    let ramfs_arc = Arc::new(SpinLock::new("ramfs", ramfs));

    // 2. Initialize VFS with RAMFS as root /
    vfs::init_vfs(ramfs_arc);

    // 3. Mount DevFS at /dev
    let devfs = devfs::DevFs::new();
    let devfs_arc = Arc::new(SpinLock::new("devfs", devfs));
    if let Some(mut vfs_guard) = vfs::get_vfs() {
        if let Some(vfs) = vfs_guard.as_mut() {
            let _ = vfs.mount("/dev", devfs_arc);
        }
    }

    // 4. Format and mount NovaFS at /storage
    if let Ok(()) = novafs::novafs_format() {
        if let Ok(novafs_inst) = novafs::NovaFs::mount() {
            let novafs_arc = Arc::new(SpinLock::new("novafs", novafs_inst));
            if let Some(mut vfs_guard) = vfs::get_vfs() {
                if let Some(vfs) = vfs_guard.as_mut() {
                    let _ = vfs.mount("/storage", novafs_arc);
                }
            }
            serial::write_str("[KERNEL] NovaFS persistent block filesystem mounted at /storage.\n");
        }
    }

    serial::write_str("[KERNEL] VFS root (/), /dev, and /storage mounted and ready.\n");
}
