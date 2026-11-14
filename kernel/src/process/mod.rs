//! NovaOS Apex — Process Management, Ring 3 Address Spaces & Lifecycle Accounting
//! Implements Process Control Blocks (PCB), parent-child relationships, per-process handle tables, and cleanups.

pub mod elf_loader;

extern crate alloc;
use alloc::vec::Vec;
use crate::memory::address::{PhysAddr, PHYSICAL_MEMORY_OFFSET};
use crate::memory::frame::{alloc_frame, free_frame};
use crate::memory::paging::{PageTable, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::objects::handle::HandleTable;
use crate::scheduler::thread::ThreadPriority;
use crate::serial;
use core::ptr::addr_of_mut;

pub const MAX_PROCESSES: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct ProcessId(pub u32);

impl ProcessId {
    pub const KERNEL: Self = Self(0);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessState {
    Starting,
    Running,
    Waiting(u32), // Waiting on child PID
    Zombie(i32),  // Exit code
    Terminated(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessError {
    ProcessTableFull,
    ProcessNotFound,
    AddressSpaceCreationFailed,
    ElfLoadFailed(elf_loader::ElfError),
    ThreadSpawnFailed,
    AccessDenied,
}

pub struct Process {
    pub id: ProcessId,
    pub parent_id: Option<ProcessId>,
    pub name: [u8; 32],
    pub state: ProcessState,
    pub pml4_phys: PhysAddr,
    pub handle_table: HandleTable,
    pub allocated_frames: Vec<PhysAddr>,
    pub user_heap_start: u64,
    pub user_heap_end: u64,
    pub cpu_ticks: u64,
    pub exit_code: Option<i32>,
    pub credentials: crate::security::Credentials,
    pub cwd: alloc::string::String,
    pub open_files: Vec<Option<crate::fs::vfs::OpenFile>>,
}

impl Process {
    pub fn new(
        id: ProcessId,
        parent_id: Option<ProcessId>,
        name: &str,
        pml4_phys: PhysAddr,
    ) -> Self {
        let mut name_buf = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(32);
        name_buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            id,
            parent_id,
            name: name_buf,
            state: ProcessState::Starting,
            pml4_phys,
            handle_table: HandleTable::new(),
            allocated_frames: Vec::new(),
            user_heap_start: 0x0000_1000_0000,
            user_heap_end:   0x0000_1000_0000,
            cpu_ticks: 0,
            exit_code: None,
            credentials: if id == ProcessId::KERNEL {
                crate::security::Credentials::root()
            } else {
                crate::security::Credentials::user(1000, 1000)
            },
            cwd: alloc::string::String::from("/"),
            open_files: alloc::vec![None, None, None],
        }
    }

    pub fn alloc_fd(&mut self, file: crate::fs::vfs::OpenFile) -> usize {
        for (i, slot) in self.open_files.iter_mut().enumerate() {
            if i >= 3 && slot.is_none() {
                *slot = Some(file);
                return i;
            }
        }
        self.open_files.push(Some(file));
        self.open_files.len() - 1
    }

    pub fn get_file(&self, fd: usize) -> Option<&crate::fs::vfs::OpenFile> {
        self.open_files.get(fd)?.as_ref()
    }

    pub fn get_file_mut(&mut self, fd: usize) -> Option<&mut crate::fs::vfs::OpenFile> {
        self.open_files.get_mut(fd)?.as_mut()
    }

    pub fn close_fd(&mut self, fd: usize) -> bool {
        if fd < self.open_files.len() && self.open_files[fd].is_some() {
            self.open_files[fd] = None;
            true
        } else {
            false
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("<unknown>")
    }
}

pub struct ProcessManager {
    processes: [Option<Process>; MAX_PROCESSES],
    next_pid: u32,
}

static mut PROCESS_MANAGER: ProcessManager = ProcessManager {
    processes: [const { None }; MAX_PROCESSES],
    next_pid: 1,
};

pub fn init() {
    unsafe {
        let pm = &mut *addr_of_mut!(PROCESS_MANAGER);

        // PID 0: Kernel Process
        let kernel_pml4 = PhysAddr::new(crate::arch::x86_64::instructions::read_cr3());
        let mut kernel_proc = Process::new(
            ProcessId::KERNEL,
            None,
            "kernel",
            kernel_pml4,
        );
        kernel_proc.state = ProcessState::Running;

        pm.processes[0] = Some(kernel_proc);
        pm.next_pid = 1;
    }

    serial::write_str("[PROCESS] Process manager initialized. PID 0 (Kernel) registered.\n");
}

pub fn create_user_address_space() -> Result<PhysAddr, ProcessError> {
    let pml4_frame = alloc_frame().ok_or(ProcessError::AddressSpaceCreationFailed)?;
    let pml4 = unsafe { &mut *(pml4_frame.as_u64() as *mut PageTable) };
    pml4.zero();

    // Allocate dedicated PDPT for lower half of user address space
    let user_pdpt_frame = alloc_frame().ok_or(ProcessError::AddressSpaceCreationFailed)?;
    let user_pdpt = unsafe { &mut *(user_pdpt_frame.as_u64() as *mut PageTable) };
    user_pdpt.zero();

    // Link user PDPT into user PML4 entry 0 with USER permissions
    pml4.entries[0] = user_pdpt_frame.as_u64() | PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER;

    // Clone kernel transition mapping: copy entry 0 of kernel PDPT (covers lower 1 GiB for kernel code/data)
    let kernel_cr3 = crate::arch::x86_64::instructions::read_cr3();
    let kernel_pml4 = unsafe { &*(kernel_cr3 as *const PageTable) };
    let kernel_pdpt_phys = kernel_pml4.entries[0] & 0x000F_FFFF_FFFF_F000;
    let kernel_pdpt = unsafe { &*(kernel_pdpt_phys as *const PageTable) };

    // user_pdpt entry 0 points to kernel's first 1 GiB PD so kernel execution during syscall/interrupt works
    user_pdpt.entries[0] = kernel_pdpt.entries[0];
    user_pdpt.entries[2] = kernel_pdpt.entries[2]; // 2 GiB - 3 GiB: GOP Framebuffer
    user_pdpt.entries[3] = kernel_pdpt.entries[3]; // 3 GiB - 4 GiB: PCI MMIO / APIC / BARs

    // Higher-half direct map for physical memory access (PML4 entry 256)
    let p4_high_idx = (PHYSICAL_MEMORY_OFFSET >> 39) & 0x1FF;
    pml4.entries[p4_high_idx as usize] = kernel_pml4.entries[p4_high_idx as usize];

    Ok(pml4_frame)
}

pub fn spawn_process(
    name: &str,
    elf_data: &[u8],
    parent_id: Option<ProcessId>,
) -> Result<ProcessId, ProcessError> {
    let pm = unsafe { &mut *addr_of_mut!(PROCESS_MANAGER) };

    let free_idx = pm.processes.iter().position(|p| p.is_none())
        .ok_or(ProcessError::ProcessTableFull)?;

    let pid = ProcessId(pm.next_pid);
    pm.next_pid += 1;

    // 1. Create clean user address space
    let pml4_phys = create_user_address_space()?;

    // 2. Load ELF64 binary into new address space
    let loaded = elf_loader::load_elf64(elf_data, pml4_phys)
        .map_err(ProcessError::ElfLoadFailed)?;

    // 3. Create PCB
    let mut proc = Process::new(pid, parent_id, name, pml4_phys);
    proc.allocated_frames = loaded.allocated_frames;
    proc.allocated_frames.push(pml4_phys); // Also track PML4 for cleanup
    proc.state = ProcessState::Running;

    // 4. Spawn user thread in scheduler
    let _tid = crate::scheduler::spawn_user_thread(
        name,
        pid.0,
        loaded.entry_point,
        loaded.user_stack_top,
        pml4_phys.as_u64(),
        ThreadPriority::Normal,
    ).map_err(|_| ProcessError::ThreadSpawnFailed)?;

    pm.processes[free_idx] = Some(proc);

    serial::write_str("[PROCESS] Spawned user process '");
    serial::write_str(name);
    serial::write_str("' (PID ");
    print_u32(pid.0);
    serial::write_str(") in Ring 3.\n");

    Ok(pid)
}

pub fn terminate_process(pid: ProcessId, exit_code: i32) {
    let pm = unsafe { &mut *addr_of_mut!(PROCESS_MANAGER) };

    if let Some(proc) = pm.processes.iter_mut().find_map(|p| p.as_mut().filter(|p| p.id == pid)) {
        proc.state = ProcessState::Terminated(exit_code);
        proc.exit_code = Some(exit_code);

        serial::write_str("[PROCESS] Process '");
        serial::write_str(proc.name_str());
        serial::write_str("' (PID ");
        print_u32(pid.0);
        serial::write_str(") terminated with exit code ");
        print_i32(exit_code);
        serial::write_str(".\n");

        // Clean up allocated physical frames
        for frame in proc.allocated_frames.drain(..) {
            let _ = free_frame(frame);
        }
    }
}

pub fn get_process_mut(pid: ProcessId) -> Option<&'static mut Process> {
    let pm = unsafe { &mut *addr_of_mut!(PROCESS_MANAGER) };
    pm.processes.iter_mut().find_map(|p| p.as_mut().filter(|p| p.id == pid))
}

pub fn get_current_process_mut() -> Option<&'static mut Process> {
    let pid = crate::scheduler::get_current_pid();
    get_process_mut(ProcessId(pid))
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u64,
    pub name: [u8; 32],
    pub state: u32,
    pub priority: u32,
    pub cpu_ticks: u64,
    pub mem_bytes: u64,
}

pub fn list_processes(out: &mut [ProcessInfo]) -> usize {
    let pm = unsafe { &*addr_of_mut!(PROCESS_MANAGER) };
    let mut count = 0;
    for proc_opt in pm.processes.iter() {
        if let Some(proc) = proc_opt {
            if count < out.len() {
                let state_code = match proc.state {
                    ProcessState::Starting => 0,
                    ProcessState::Running => 1,
                    ProcessState::Waiting(_) => 2,
                    ProcessState::Zombie(_) => 3,
                    ProcessState::Terminated(_) => 4,
                };
                let mem = proc.allocated_frames.len() as u64 * 4096;
                out[count] = ProcessInfo {
                    pid: proc.id.0 as u64,
                    name: proc.name,
                    state: state_code,
                    priority: 2,
                    cpu_ticks: proc.cpu_ticks,
                    mem_bytes: mem,
                };
                count += 1;
            }
        }
    }
    count
}

pub fn get_process_count() -> u32 {
    let pm = unsafe { &*addr_of_mut!(PROCESS_MANAGER) };
    pm.processes.iter().filter(|p| p.is_some()).count() as u32
}

pub fn create_test_process(name: &str) -> Result<ProcessId, ProcessError> {
    let pm = unsafe { &mut *addr_of_mut!(PROCESS_MANAGER) };
    let free_idx = pm.processes.iter().position(|p| p.is_none())
        .ok_or(ProcessError::ProcessTableFull)?;

    let pid = ProcessId(pm.next_pid);
    pm.next_pid += 1;

    let pml4_phys = create_user_address_space()?;
    let mut proc = Process::new(pid, Some(ProcessId::KERNEL), name, pml4_phys);
    proc.allocated_frames.push(pml4_phys);
    proc.state = ProcessState::Running;

    pm.processes[free_idx] = Some(proc);
    Ok(pid)
}

pub fn reap_terminated_processes() {
    let pm = unsafe { &mut *addr_of_mut!(PROCESS_MANAGER) };
    for slot in pm.processes.iter_mut() {
        if let Some(ref p) = slot {
            if matches!(p.state, ProcessState::Terminated(_)) {
                *slot = None;
            }
        }
    }
}

fn print_u32(mut n: u32) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}

fn print_i32(n: i32) {
    if n < 0 {
        serial::write_str("-");
        print_u32((-n) as u32);
    } else {
        print_u32(n as u32);
    }
}
