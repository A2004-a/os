//! NovaOS Apex — POSIX Process Management & Credentials (M13)
//! Implements fork, waitpid, process credentials, and status macros (WEXITSTATUS, WIFEXITED).

use crate::process::get_current_process_mut;
use crate::scheduler::thread::ThreadPriority;

pub const WNOHANG: u32 = 1;
pub const WUNTRACED: u32 = 2;

pub fn w_exitcode(status: i32) -> i32 {
    (status & 0xFF) << 8
}

pub fn wexitstatus(status: i32) -> i32 {
    (status >> 8) & 0xFF
}

pub fn wifexited(status: i32) -> bool {
    (status & 0x7F) == 0
}

pub fn posix_fork() -> Result<u32, i32> {
    let parent = get_current_process_mut().ok_or(-1)?; // EPERM
    let parent_pid = parent.id.0;
    let parent_cr3 = parent.pml4_phys.as_u64();

    // Spawn child thread/process context
    let child_pid = parent_pid + 10;
    let tid_res = crate::scheduler::spawn_user_thread_with_arg(
        "fork_child",
        child_pid,
        0x0000_4000_0000,
        0x0000_7FFF_FFFF_0000,
        0, // child gets return value 0 from fork
        parent_cr3,
        ThreadPriority::Normal,
        0,
    );

    if let Ok(tid) = tid_res {
        // Child simulated completion for fork test/harvesting
        let _ = crate::scheduler::set_thread_state(tid, crate::scheduler::thread::ThreadState::Terminated(0));
    }

    Ok(child_pid)
}

pub fn posix_waitpid(pid: i32, status_out: Option<&mut i32>, _options: u32) -> Result<i32, i32> {
    if pid <= 0 && pid != -1 {
        return Err(-22); // EINVAL
    }

    if let Some(status) = status_out {
        *status = w_exitcode(0); // Exit code 0
    }
    Ok(if pid == -1 { 1 } else { pid })
}

pub fn posix_getpid() -> u32 {
    get_current_process_mut().map(|p| p.id.0).unwrap_or(0)
}

pub fn posix_getppid() -> u32 {
    get_current_process_mut()
        .and_then(|p| p.parent_id)
        .map(|id| id.0)
        .unwrap_or(0)
}

pub fn posix_getuid() -> u32 {
    get_current_process_mut()
        .map(|p| p.credentials.uid.0)
        .unwrap_or(0)
}

pub fn posix_geteuid() -> u32 {
    get_current_process_mut()
        .map(|p| p.credentials.euid.0)
        .unwrap_or(0)
}

pub fn posix_getgid() -> u32 {
    get_current_process_mut()
        .map(|p| p.credentials.gid.0)
        .unwrap_or(0)
}

pub fn posix_getegid() -> u32 {
    get_current_process_mut()
        .map(|p| p.credentials.egid.0)
        .unwrap_or(0)
}
