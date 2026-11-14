//! Process Control Block (PCB) & Address Space Isolation

pub mod elf_loader;
pub mod pkg_mgr;
pub mod pkg_test;

pub use elf_loader::{ElfError, ElfLoader, LoadedProgram};
pub use pkg_mgr::{
    InstalledPackage, PackageError, PackageHeader, PackageManifest, PackageManager,
    CAP_ADMIN, CAP_ALL, CAP_AUDIO, CAP_GRAPHICS, CAP_IPC, CAP_NET, CAP_NONE, CAP_STORAGE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Active,
    Sleeping,
    Zombie,
}

pub struct Process {
    pub pid: ProcessId,
    pub parent_pid: Option<ProcessId>,
    pub cr3_address: u64,
    pub status: ProcessStatus,
    pub exit_code: i32,
}

impl Process {
    pub fn new(pid: u64, parent: Option<ProcessId>, cr3_address: u64) -> Self {
        Process {
            pid: ProcessId(pid),
            parent_pid: parent,
            cr3_address,
            status: ProcessStatus::Active,
            exit_code: 0,
        }
    }
}
