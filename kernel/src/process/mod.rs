//! Process Control Block (PCB) & Address Space Isolation

pub mod elf_loader;

pub use elf_loader::{ElfError, ElfLoader, LoadedProgram};

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
