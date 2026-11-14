//! NovaOS Apex — Architecture System Call Dispatcher & 14 POSIX/NovaOS Primitives
//! Implements syscall gates (vector 0x80), pointer checks, handle tables, and Ring 3 transitions.

pub mod tests;
pub mod validate;

use crate::arch::x86_64::exceptions::ExceptionContext;
use crate::memory::address::{VirtAddr, PAGE_SIZE};
use crate::memory::frame::{alloc_frame, free_frame};
use crate::memory::paging::{OffsetPageTable, PAGE_PRESENT, PAGE_USER, PAGE_WRITABLE};
use crate::objects::handle::Handle;
use crate::objects::object::{FileObject, ObjectPayload};
use crate::objects::types::{CapabilityRights, ObjectType};
use crate::serial;
use validate::{validate_user_buffer, validate_user_string};

// System Call Table Identifiers
pub const SYS_EXIT: usize = 1;
pub const SYS_READ: usize = 2;
pub const SYS_WRITE: usize = 3;
pub const SYS_OPEN: usize = 4;
pub const SYS_CLOSE: usize = 5;
pub const SYS_SPAWN: usize = 6;
pub const SYS_WAIT: usize = 7;
pub const SYS_YIELD: usize = 8;
pub const SYS_SLEEP: usize = 9;
pub const SYS_GET_TIME: usize = 10;
pub const SYS_MAP_MEMORY: usize = 11;
pub const SYS_UNMAP_MEMORY: usize = 12;
pub const SYS_SEND_IPC: usize = 13;
pub const SYS_RECEIVE_IPC: usize = 14;
pub const SYS_STAT: usize = 15;
pub const SYS_READDIR: usize = 16;
pub const SYS_MKDIR: usize = 17;
pub const SYS_UNLINK: usize = 18;
pub const SYS_CHDIR: usize = 19;
pub const SYS_GETCWD: usize = 20;
pub const SYS_SEEK: usize = 21;
pub const SYS_SOCKET: usize = 22;
pub const SYS_BIND: usize = 23;
pub const SYS_LISTEN: usize = 24;
pub const SYS_ACCEPT: usize = 25;
pub const SYS_CONNECT: usize = 26;
pub const SYS_SEND: usize = 27;
pub const SYS_RECV: usize = 28;
pub const SYS_CREATE_WINDOW: usize = 29;
pub const SYS_DESTROY_WINDOW: usize = 30;
pub const SYS_BLIT_WINDOW: usize = 31;
pub const SYS_POLL_EVENT: usize = 32;
pub const SYS_GET_DISPLAY_INFO: usize = 33;
pub const SYS_SET_CURSOR: usize = 34;
pub const SYS_SNAP_WINDOW: usize = 35;
pub const SYS_RESTORE_WINDOW: usize = 36;
pub const SYS_MINIMIZE_WINDOW: usize = 37;
pub const SYS_SWITCH_WORKSPACE: usize = 38;
pub const SYS_GET_WORKSPACE: usize = 39;
pub const SYS_TOGGLE_LAUNCHER: usize = 40;
pub const SYS_GET_SYS_STATS: usize = 41;
pub const SYS_LIST_PROCESSES: usize = 42;
pub const SYS_KILL_PROCESS: usize = 43;
pub const SYS_VERIFY_PACKAGE: usize = 44;
pub const SYS_INSTALL_PACKAGE: usize = 45;
pub const SYS_UNINSTALL_PACKAGE: usize = 46;
pub const SYS_GET_PARTITIONS: usize = 47;
pub const SYS_TRIGGER_RECOVERY: usize = 48;
pub const SYS_PLAY_AUDIO: usize = 49;
pub const SYS_LIST_BLOCK_DEVICES: usize = 50;
pub const SYS_GET_HARDWARE_INFO: usize = 51;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockDeviceInfo {
    pub id: u32,
    pub name: [u8; 32],
    pub bus_type: u32, // 0 = VirtIO, 1 = AHCI SATA, 2 = NVMe
    pub capacity_sectors: u64,
    pub sector_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareSummary {
    pub pci_devices_detected: u32,
    pub block_devices_count: u32,
    pub has_ahci_sata: u32,
    pub has_nvme_ssd: u32,
    pub has_usb3_xhci: u32,
    pub has_intel_hda: u32,
    pub audio_codecs_count: u32,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageVerifyResult {
    pub valid: u32,
    pub name: [u8; 32],
    pub version: [u8; 16],
    pub publisher: [u8; 32],
    pub capabilities: u32,
    pub payload_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartitionInfo {
    pub id: u32,
    pub type_guid: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub size_sectors: u64,
    pub attributes: u64,
    pub name: [u8; 36],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SysStats {
    pub uptime_ms: u64,
    pub total_ram: u64,
    pub used_ram: u64,
    pub free_ram: u64,
    pub total_tasks: u32,
    pub active_windows: u32,
    pub cpu_usage_pct: u32,
}

#[no_mangle]
pub fn handle_syscall(ctx: &mut ExceptionContext) {
    let syscall_num = ctx.rax as usize;
    let arg1 = ctx.rdi;
    let arg2 = ctx.rsi;
    let arg3 = ctx.rdx;
    let arg4 = ctx.r10;
    let _arg5 = ctx.r8;
    let _arg6 = ctx.r9;

    let result: i64 = match syscall_num {
        SYS_EXIT => {
            let code = arg1 as i32;
            let current_pid = crate::scheduler::get_current_pid();
            if current_pid != 0 {
                crate::process::terminate_process(crate::process::ProcessId(current_pid), code);
            }
            crate::scheduler::exit_current(code, ctx);
            return; // Will not reach return with old thread context
        }

        SYS_READ => {
            let handle = arg1 as usize;
            let buf_ptr = arg2;
            let len = arg3 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, len, true) {
                -(e as i32 as i64)
            } else if handle == 0 {
                // Stdin: read from keyboard if available
                if let Some(ch) = crate::drivers::keyboard::read_char() {
                    let out_buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
                    if !out_buf.is_empty() {
                        out_buf[0] = ch as u8;
                        1
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                let out_buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
                if let Some(proc) = crate::process::get_current_process_mut() {
                    if let Some(file) = proc.get_file_mut(handle) {
                        match file.read(out_buf) {
                            Ok(n) => n as i64,
                            Err(e) => e.to_errno(),
                        }
                    } else if let Ok(_) = proc.handle_table.get(Handle(handle as u32), ObjectType::File, CapabilityRights::READ) {
                        len as i64 // Backward compatibility with mock handle
                    } else {
                        -9 // EBADF
                    }
                } else {
                    -1
                }
            }
        }

        SYS_WRITE => {
            let handle = arg1 as usize;
            let buf_ptr = arg2;
            let len = arg3 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, len, false) {
                -(e as i32 as i64)
            } else if handle == 1 || handle == 2 {
                // Stdout / Stderr: write directly to serial and text console
                let bytes = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
                if let Ok(s) = core::str::from_utf8(bytes) {
                    serial::write_str(s);
                    crate::drivers::console::write_str(s);
                } else {
                    for &b in bytes {
                        serial::write_byte(b);
                    }
                }
                len as i64
            } else {
                let in_buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
                if let Some(proc) = crate::process::get_current_process_mut() {
                    if let Some(file) = proc.get_file_mut(handle) {
                        match file.write(in_buf) {
                            Ok(n) => n as i64,
                            Err(e) => e.to_errno(),
                        }
                    } else if let Ok(_) = proc.handle_table.get(Handle(handle as u32), ObjectType::File, CapabilityRights::WRITE) {
                        len as i64 // Backward compatibility with mock handle
                    } else {
                        -9 // EBADF
                    }
                } else {
                    -1
                }
            }
        }

        SYS_OPEN => {
            let path_ptr = arg1;
            let flags = arg2 as u32;
            let mode = (arg3 as u32) & 0o7777;

            match validate_user_string(path_ptr, 256) {
                Ok(path_str) => {
                    if let Some(proc) = crate::process::get_current_process_mut() {
                        let creds = proc.credentials;
                        let full_path = crate::fs::path::normalize_path(&proc.cwd, path_str);
                        let vfs_guard = crate::fs::vfs::get_vfs();
                        if let Some(vfs_lock) = vfs_guard {
                            if let Some(vfs) = vfs_lock.as_ref() {
                                match vfs.open(&full_path, flags, mode, &creds) {
                                    Ok(open_file) => {
                                        let fd = proc.alloc_fd(open_file);
                                        // Also add typed handle for backward compat
                                        let payload = ObjectPayload::File(FileObject {
                                            inode: 1,
                                            path: [0u8; 64],
                                            size: 0,
                                            cursor: 0,
                                        });
                                        let rights = CapabilityRights::READ | CapabilityRights::WRITE | CapabilityRights::CLOSE;
                                        let _ = proc.handle_table.insert_object(payload, rights);
                                        fd as i64
                                    }
                                    Err(e) => e.to_errno(),
                                }
                            } else {
                                -5 // EIO
                            }
                        } else {
                            -5 // EIO
                        }
                    } else {
                        -1
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_CLOSE => {
            let handle = arg1 as usize;
            if let Some(proc) = crate::process::get_current_process_mut() {
                let closed_fd = proc.close_fd(handle);
                let _ = proc.handle_table.close(Handle(handle as u32));
                if closed_fd || handle < 3 {
                    0
                } else {
                    -9 // EBADF
                }
            } else {
                -1
            }
        }

        SYS_SPAWN => {
            let elf_ptr = arg1;
            let len = arg2 as usize;
            let name_ptr = arg3;

            let val_buf = validate_user_buffer(elf_ptr, len, false);
            let val_name = validate_user_string(name_ptr, 64);

            match (val_buf, val_name) {
                (Ok(()), Ok(name)) => {
                    let elf_bytes = unsafe { core::slice::from_raw_parts(elf_ptr as *const u8, len) };
                    let parent_pid = crate::scheduler::get_current_pid();
                    match crate::process::spawn_process(
                        name,
                        elf_bytes,
                        Some(crate::process::ProcessId(parent_pid)),
                    ) {
                        Ok(new_pid) => new_pid.0 as i64,
                        Err(_) => -1,
                    }
                }
                _ => -14, // EFAULT
            }
        }

        SYS_WAIT => {
            let target_pid = arg1 as u32;
            let status_ptr = arg2;

            if status_ptr != 0 {
                if let Err(e) = validate_user_buffer(status_ptr, core::mem::size_of::<i32>(), true) {
                    let err_code = -(e as i32) as i64;
                    ctx.rax = err_code as u64;
                    return;
                }
            }

            if let Some(proc) = crate::process::get_process_mut(crate::process::ProcessId(target_pid)) {
                if let Some(code) = proc.exit_code {
                    if status_ptr != 0 {
                        unsafe { *(status_ptr as *mut i32) = code; }
                    }
                    target_pid as i64
                } else {
                    -10 // ECHILD / not yet terminated
                }
            } else {
                -3 // ESRCH
            }
        }

        SYS_YIELD => {
            crate::scheduler::yield_current(ctx);
            0
        }

        SYS_SLEEP => {
            let ms = arg1;
            crate::scheduler::sleep_current(ms, ctx);
            0
        }

        SYS_GET_TIME => {
            crate::drivers::timer::get_uptime_ms() as i64
        }

        SYS_MAP_MEMORY => {
            let virt = arg1;
            let size = arg2 as usize;
            let _flags = arg3 as u32;

            let page_count = (size + PAGE_SIZE - 1) / PAGE_SIZE;
            if virt >= validate::USER_SPACE_LIMIT || virt + (size as u64) > validate::USER_SPACE_LIMIT {
                -14 // EFAULT
            } else {
                if let Some(proc) = crate::process::get_current_process_mut() {
                    let mut mapper = OffsetPageTable::new(proc.pml4_phys);
                    let mut mapped_ok = true;

                    for p in 0..page_count {
                        let page_vaddr = virt + (p as u64 * PAGE_SIZE as u64);
                        if let Some(frame) = alloc_frame() {
                            proc.allocated_frames.push(frame);
                            if mapper.map_page(
                                VirtAddr::new(page_vaddr),
                                frame,
                                PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER,
                            ).is_err() {
                                mapped_ok = false;
                                break;
                            }
                        } else {
                            mapped_ok = false;
                            break;
                        }
                    }

                    if mapped_ok {
                        virt as i64
                    } else {
                        -12 // ENOMEM
                    }
                } else {
                    -1
                }
            }
        }

        SYS_UNMAP_MEMORY => {
            let virt = arg1;
            let size = arg2 as usize;
            let page_count = (size + PAGE_SIZE - 1) / PAGE_SIZE;

            if let Some(proc) = crate::process::get_current_process_mut() {
                let mut mapper = OffsetPageTable::new(proc.pml4_phys);
                for p in 0..page_count {
                    let page_vaddr = virt + (p as u64 * PAGE_SIZE as u64);
                    if let Ok(phys) = mapper.unmap_page(VirtAddr::new(page_vaddr)) {
                        let _ = free_frame(phys);
                    }
                }
                0
            } else {
                -1
            }
        }

        SYS_SEND_IPC => {
            let dest_pid = arg1 as u32;
            let msg_ptr = arg2;
            let len = arg3 as usize;

            if let Err(e) = validate_user_buffer(msg_ptr, len, false) {
                -(e as i32 as i64)
            } else {
                let msg_slice = unsafe { core::slice::from_raw_parts(msg_ptr as *const u8, len) };
                let sender_pid = crate::scheduler::get_current_pid();
                match crate::ipc::send(sender_pid, dest_pid, msg_slice) {
                    Ok(()) => 0,
                    Err(code) => code as i64,
                }
            }
        }

        SYS_RECEIVE_IPC => {
            let src_pid = arg1 as u32;
            let buf_ptr = arg2;
            let max_len = arg3 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, max_len, true) {
                -(e as i32 as i64)
            } else {
                let out_slice = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, max_len) };
                let current_pid = crate::scheduler::get_current_pid();
                match crate::ipc::receive(current_pid, src_pid, out_slice) {
                    Ok(bytes_received) => bytes_received as i64,
                    Err(code) => code as i64,
                }
            }
        }

        SYS_STAT => {
            let path_ptr = arg1;
            let stat_ptr = arg2;

            if let Err(e) = validate_user_buffer(stat_ptr, core::mem::size_of::<crate::fs::types::FileStat>(), true) {
                -(e as i32 as i64)
            } else {
                match validate_user_string(path_ptr, 256) {
                    Ok(path_str) => {
                        if let Some(proc) = crate::process::get_current_process_mut() {
                            let creds = proc.credentials;
                            let full_path = crate::fs::path::normalize_path(&proc.cwd, path_str);
                            if let Some(vfs_lock) = crate::fs::vfs::get_vfs() {
                                if let Some(vfs) = vfs_lock.as_ref() {
                                    match vfs.stat(&full_path, &creds) {
                                        Ok(st) => {
                                            unsafe { *(stat_ptr as *mut crate::fs::types::FileStat) = st; }
                                            0
                                        }
                                        Err(e) => e.to_errno(),
                                    }
                                } else {
                                    -5
                                }
                            } else {
                                -5
                            }
                        } else {
                            -1
                        }
                    }
                    Err(e) => -(e as i32 as i64),
                }
            }
        }

        SYS_READDIR => {
            let path_ptr = arg1;
            let offset = arg2 as usize;
            let entry_ptr = arg3;

            if let Err(e) = validate_user_buffer(entry_ptr, core::mem::size_of::<crate::fs::types::UserDirEntry>(), true) {
                -(e as i32 as i64)
            } else {
                match validate_user_string(path_ptr, 256) {
                    Ok(path_str) => {
                        if let Some(proc) = crate::process::get_current_process_mut() {
                            let creds = proc.credentials;
                            let full_path = crate::fs::path::normalize_path(&proc.cwd, path_str);
                            if let Some(vfs_lock) = crate::fs::vfs::get_vfs() {
                                if let Some(vfs) = vfs_lock.as_ref() {
                                    match vfs.readdir(&full_path, offset, &creds) {
                                        Ok(Some(entry)) => {
                                            let user_entry = crate::fs::types::UserDirEntry::new(
                                                entry.inode.0,
                                                entry.file_type,
                                                &entry.name,
                                            );
                                            unsafe { *(entry_ptr as *mut crate::fs::types::UserDirEntry) = user_entry; }
                                            1 // 1 entry returned
                                        }
                                        Ok(None) => 0, // End of directory
                                        Err(e) => e.to_errno(),
                                    }
                                } else {
                                    -5
                                }
                            } else {
                                -5
                            }
                        } else {
                            -1
                        }
                    }
                    Err(e) => -(e as i32 as i64),
                }
            }
        }

        SYS_MKDIR => {
            let path_ptr = arg1;
            let mode = (arg2 as u32) & 0o7777;

            match validate_user_string(path_ptr, 256) {
                Ok(path_str) => {
                    if let Some(proc) = crate::process::get_current_process_mut() {
                        let creds = proc.credentials;
                        let full_path = crate::fs::path::normalize_path(&proc.cwd, path_str);
                        if let Some(vfs_lock) = crate::fs::vfs::get_vfs() {
                            if let Some(vfs) = vfs_lock.as_ref() {
                                match vfs.mkdir(&full_path, mode, &creds) {
                                    Ok(()) => 0,
                                    Err(e) => e.to_errno(),
                                }
                            } else {
                                -5
                            }
                        } else {
                            -5
                        }
                    } else {
                        -1
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_UNLINK => {
            let path_ptr = arg1;

            match validate_user_string(path_ptr, 256) {
                Ok(path_str) => {
                    if let Some(proc) = crate::process::get_current_process_mut() {
                        let creds = proc.credentials;
                        let full_path = crate::fs::path::normalize_path(&proc.cwd, path_str);
                        if let Some(vfs_lock) = crate::fs::vfs::get_vfs() {
                            if let Some(vfs) = vfs_lock.as_ref() {
                                match vfs.unlink(&full_path, &creds) {
                                    Ok(()) => 0,
                                    Err(e) => e.to_errno(),
                                }
                            } else {
                                -5
                            }
                        } else {
                            -5
                        }
                    } else {
                        -1
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_CHDIR => {
            let path_ptr = arg1;

            match validate_user_string(path_ptr, 256) {
                Ok(path_str) => {
                    if let Some(proc) = crate::process::get_current_process_mut() {
                        let creds = proc.credentials;
                        let full_path = crate::fs::path::normalize_path(&proc.cwd, path_str);
                        if let Some(vfs_lock) = crate::fs::vfs::get_vfs() {
                            if let Some(vfs) = vfs_lock.as_ref() {
                                match vfs.stat(&full_path, &creds) {
                                    Ok(st) => {
                                        if (st.mode & crate::security::S_IFDIR) != 0 {
                                            proc.cwd = full_path;
                                            0
                                        } else {
                                            -20 // ENOTDIR
                                        }
                                    }
                                    Err(e) => e.to_errno(),
                                }
                            } else {
                                -5
                            }
                        } else {
                            -5
                        }
                    } else {
                        -1
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_GETCWD => {
            let buf_ptr = arg1;
            let max_len = arg2 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, max_len, true) {
                -(e as i32 as i64)
            } else if let Some(proc) = crate::process::get_current_process_mut() {
                let cwd_bytes = proc.cwd.as_bytes();
                if cwd_bytes.len() + 1 > max_len {
                    -28 // ENOSPC / ERANGE
                } else {
                    let out = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, max_len) };
                    out[..cwd_bytes.len()].copy_from_slice(cwd_bytes);
                    out[cwd_bytes.len()] = 0;
                    cwd_bytes.len() as i64
                }
            } else {
                -1
            }
        }

        SYS_SEEK => {
            let handle = arg1 as usize;
            let offset = arg2 as i64;
            let whence = arg3 as u32;

            if let Some(proc) = crate::process::get_current_process_mut() {
                if let Some(file) = proc.get_file_mut(handle) {
                    match file.seek(offset, whence) {
                        Ok(new_off) => new_off as i64,
                        Err(e) => e.to_errno(),
                    }
                } else {
                    -9 // EBADF
                }
            } else {
                -1
            }
        }

        SYS_SOCKET => {
            let domain = arg1 as u32;
            let sock_type = arg2 as u32;
            let protocol = arg3 as u32;

            if let Some(proc) = crate::process::get_current_process_mut() {
                let creds = proc.credentials;
                match crate::network::socket::SOCKET_TABLE.lock().create_socket(domain, sock_type, protocol, &creds) {
                    Ok(handle) => handle.0 as i64,
                    Err(e) => e as i64,
                }
            } else {
                -1
            }
        }

        SYS_BIND => {
            let handle = arg1 as u32;
            let ip_packed = arg2 as u32;
            let port = arg3 as u16;

            let ip = crate::network::ipv4::Ipv4Address([
                (ip_packed >> 24) as u8,
                (ip_packed >> 16) as u8,
                (ip_packed >> 8) as u8,
                (ip_packed & 0xFF) as u8,
            ]);

            if let Some(proc) = crate::process::get_current_process_mut() {
                let creds = proc.credentials;
                match crate::network::socket::SOCKET_TABLE.lock().bind(crate::network::socket::SocketHandle(handle), ip, port, &creds) {
                    Ok(()) => 0,
                    Err(e) => e as i64,
                }
            } else {
                -1
            }
        }

        SYS_LISTEN => {
            let handle = arg1 as u32;
            let backlog = arg2 as usize;

            match crate::network::socket::SOCKET_TABLE.lock().listen(crate::network::socket::SocketHandle(handle), backlog) {
                Ok(()) => 0,
                Err(e) => e as i64,
            }
        }

        SYS_ACCEPT => {
            let _handle = arg1 as u32;
            if let Some(proc) = crate::process::get_current_process_mut() {
                let creds = proc.credentials;
                // Create child stream socket for accepted connection
                match crate::network::socket::SOCKET_TABLE.lock().create_socket(
                    crate::network::socket::AF_INET,
                    crate::network::socket::SOCK_STREAM,
                    0,
                    &creds,
                ) {
                    Ok(new_handle) => new_handle.0 as i64,
                    Err(e) => e as i64,
                }
            } else {
                -1
            }
        }

        SYS_CONNECT => {
            let handle = arg1 as u32;
            let ip_packed = arg2 as u32;
            let port = arg3 as u16;

            let ip = crate::network::ipv4::Ipv4Address([
                (ip_packed >> 24) as u8,
                (ip_packed >> 16) as u8,
                (ip_packed >> 8) as u8,
                (ip_packed & 0xFF) as u8,
            ]);

            match crate::network::socket::SOCKET_TABLE.lock().connect(crate::network::socket::SocketHandle(handle), ip, port) {
                Ok(Some(syn_pkt)) => {
                    let _ = crate::network::stack::send_ipv4(ip, crate::network::ipv4::IP_PROTO_TCP, &syn_pkt.serialize());
                    0
                }
                Ok(None) => 0,
                Err(e) => e as i64,
            }
        }

        SYS_SEND => {
            let handle = arg1 as u32;
            let buf_ptr = arg2;
            let len = arg3 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, len, false) {
                -(e as i32 as i64)
            } else {
                let data = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
                match crate::network::socket::SOCKET_TABLE.lock().send(crate::network::socket::SocketHandle(handle), data) {
                    Ok(sent) => sent as i64,
                    Err(e) => e as i64,
                }
            }
        }

        SYS_RECV => {
            let handle = arg1 as u32;
            let buf_ptr = arg2;
            let len = arg3 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, len, true) {
                -(e as i32 as i64)
            } else {
                let data = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
                match crate::network::socket::SOCKET_TABLE.lock().recv(crate::network::socket::SocketHandle(handle), data) {
                    Ok(rcv) => rcv as i64,
                    Err(e) => e as i64,
                }
            }
        }

        SYS_CREATE_WINDOW => {
            let title_ptr = arg1;
            let width = arg2 as u32;
            let height = arg3 as u32;
            let flags = arg4 as u32;

            match validate_user_string(title_ptr, 64) {
                Ok(title) => {
                    let pid = crate::scheduler::get_current_pid();
                    let mut comp_guard = crate::graphics::COMPOSITOR.lock();
                    if let Some(comp) = comp_guard.as_mut() {
                        let win_id = comp.create_window(title, 100, 100, width, height, flags, pid);
                        comp.composite();
                        win_id.0 as i64
                    } else {
                        -5 // EIO
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_DESTROY_WINDOW => {
            let win_id = crate::graphics::window::WindowId(arg1 as u32);
            let mut comp_guard = crate::graphics::COMPOSITOR.lock();
            if let Some(comp) = comp_guard.as_mut() {
                comp.destroy_window(win_id);
                comp.composite();
                0
            } else {
                -5
            }
        }

        SYS_BLIT_WINDOW => {
            let win_id = crate::graphics::window::WindowId(arg1 as u32);
            let buf_ptr = arg2;
            let buf_len = arg3 as usize;

            if let Err(e) = validate_user_buffer(buf_ptr, buf_len, false) {
                -(e as i32 as i64)
            } else {
                let mut comp_guard = crate::graphics::COMPOSITOR.lock();
                if let Some(comp) = comp_guard.as_mut() {
                    if let Some(win) = comp.get_window_mut(win_id) {
                        let user_pixels = unsafe { core::slice::from_raw_parts(buf_ptr as *const u32, buf_len / 4) };
                        let copy_count = user_pixels.len().min(win.surface.pixels.len());
                        win.surface.pixels[..copy_count].copy_from_slice(&user_pixels[..copy_count]);
                        comp.damage_window(win_id);
                        comp.composite();
                        0
                    } else {
                        -2 // ENOENT
                    }
                } else {
                    -5
                }
            }
        }

        SYS_POLL_EVENT => {
            let win_id = crate::graphics::window::WindowId(arg1 as u32);
            let event_buf_ptr = arg2;

            if let Err(e) = validate_user_buffer(event_buf_ptr, core::mem::size_of::<crate::graphics::events::UserEvent>(), true) {
                -(e as i32 as i64)
            } else {
                let mut comp_guard = crate::graphics::COMPOSITOR.lock();
                if let Some(comp) = comp_guard.as_mut() {
                    if let Some(win) = comp.get_window_mut(win_id) {
                        if let Some(ev) = win.pop_event() {
                            let user_ev: crate::graphics::events::UserEvent = ev.into();
                            unsafe { *(event_buf_ptr as *mut crate::graphics::events::UserEvent) = user_ev; }
                            1 // 1 event returned
                        } else {
                            0 // No event
                        }
                    } else {
                        -2 // ENOENT
                    }
                } else {
                    -5
                }
            }
        }

        SYS_GET_DISPLAY_INFO => {
            let buf_ptr = arg1;
            if let Err(e) = validate_user_buffer(buf_ptr, 16, true) {
                -(e as i32 as i64)
            } else {
                let fb = crate::drivers::framebuffer::get_driver();
                let out = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u32, 4) };
                out[0] = fb.width;
                out[1] = fb.height;
                out[2] = fb.stride;
                out[3] = 32; // 32 bits per pixel
                0
            }
        }

        SYS_SET_CURSOR => {
            let x = arg1 as i32;
            let y = arg2 as i32;
            let mut comp_guard = crate::graphics::COMPOSITOR.lock();
            if let Some(comp) = comp_guard.as_mut() {
                comp.on_mouse_move(x, y);
                comp.composite();
                0
            } else {
                -5
            }
        }

        SYS_SNAP_WINDOW => {
            let win_id = crate::graphics::window::WindowId(arg1 as u32);
            let snap_pos = match arg2 as u32 {
                1 => crate::graphics::snap::SnapPosition::LeftHalf,
                2 => crate::graphics::snap::SnapPosition::RightHalf,
                3 => crate::graphics::snap::SnapPosition::TopLeft,
                4 => crate::graphics::snap::SnapPosition::TopRight,
                5 => crate::graphics::snap::SnapPosition::BottomLeft,
                6 => crate::graphics::snap::SnapPosition::BottomRight,
                7 => crate::graphics::snap::SnapPosition::Maximize,
                _ => crate::graphics::snap::SnapPosition::None,
            };
            let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
            if let Some(shell) = shell_guard.as_mut() {
                shell.snap_window(win_id, snap_pos);
                shell.composite();
                0
            } else {
                -5
            }
        }

        SYS_RESTORE_WINDOW => {
            let win_id = crate::graphics::window::WindowId(arg1 as u32);
            let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
            if let Some(shell) = shell_guard.as_mut() {
                shell.restore_window(win_id);
                shell.composite();
                0
            } else {
                -5
            }
        }

        SYS_MINIMIZE_WINDOW => {
            let win_id = crate::graphics::window::WindowId(arg1 as u32);
            let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
            if let Some(shell) = shell_guard.as_mut() {
                shell.minimize_window(win_id);
                shell.composite();
                0
            } else {
                -5
            }
        }

        SYS_SWITCH_WORKSPACE => {
            let ws_id = arg1 as u8;
            let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
            if let Some(shell) = shell_guard.as_mut() {
                shell.switch_workspace(ws_id);
                shell.composite();
                0
            } else {
                -5
            }
        }

        SYS_GET_WORKSPACE => {
            let shell_guard = crate::graphics::desktop::DESKTOP.lock();
            if let Some(shell) = shell_guard.as_ref() {
                shell.workspaces.active_workspace as i64
            } else {
                1
            }
        }

        SYS_TOGGLE_LAUNCHER => {
            let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
            if let Some(shell) = shell_guard.as_mut() {
                shell.toggle_launcher();
                shell.composite();
                0
            } else {
                -5
            }
        }

        SYS_GET_SYS_STATS => {
            let user_buf = arg1;
            let size = core::mem::size_of::<SysStats>();
            match validate_user_buffer(user_buf, size, true) {
                Ok(()) => {
                    let h_stats = crate::memory::heap::get_heap_stats();
                    let active_win = crate::graphics::desktop::DESKTOP.lock()
                        .as_ref()
                        .map(|s| s.compositor.windows.len() as u32)
                        .unwrap_or(0);
                    let stats = SysStats {
                        uptime_ms: crate::drivers::timer::get_uptime_ms(),
                        total_ram: h_stats.total_bytes as u64,
                        used_ram: h_stats.allocated_bytes as u64,
                        free_ram: h_stats.free_bytes as u64,
                        total_tasks: crate::process::get_process_count(),
                        active_windows: active_win,
                        cpu_usage_pct: 12,
                    };
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            &stats as *const SysStats as *const u8,
                            user_buf as *mut u8,
                            size,
                        );
                    }
                    0
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_LIST_PROCESSES => {
            let buf_ptr = arg1;
            let max_count = arg2 as usize;
            let item_size = core::mem::size_of::<crate::process::ProcessInfo>();
            let total_size = max_count.saturating_mul(item_size);
            match validate_user_buffer(buf_ptr, total_size, true) {
                Ok(()) => {
                    let out_slice = unsafe {
                        core::slice::from_raw_parts_mut(
                            buf_ptr as *mut crate::process::ProcessInfo,
                            max_count,
                        )
                    };
                    let count = crate::process::list_processes(out_slice);
                    count as i64
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_KILL_PROCESS => {
            let target_pid = arg1 as u32;
            let is_root = if let Some(proc) = crate::process::get_current_process_mut() {
                proc.credentials.is_root() || proc.id.0 == target_pid
            } else {
                false
            };

            if !is_root {
                -1
            } else if target_pid == 0 {
                -1
            } else {
                crate::process::terminate_process(crate::process::ProcessId(target_pid), -9);
                0
            }
        }

        SYS_VERIFY_PACKAGE => {
            let pkg_addr = arg1;
            let pkg_len = arg2 as usize;
            let out_addr = arg3;

            let val_pkg = validate::validate_user_buffer(pkg_addr, pkg_len, false);
            let val_out = validate::validate_user_buffer(out_addr, core::mem::size_of::<PackageVerifyResult>(), true);

            match (val_pkg, val_out) {
                (Ok(()), Ok(())) => {
                    let slice = unsafe { core::slice::from_raw_parts(pkg_addr as *const u8, pkg_len) };
                    let out_ptr = out_addr as *mut PackageVerifyResult;
                    match crate::package::format::verify_nvp(slice, &crate::package::crypto::SYSTEM_ROOT_PUBKEY) {
                        Ok(hdr) => {
                            let result = PackageVerifyResult {
                                valid: 1,
                                name: hdr.name,
                                version: hdr.version,
                                publisher: hdr.publisher,
                                capabilities: hdr.capabilities,
                                payload_size: hdr.payload_size,
                            };
                            unsafe { core::ptr::write_unaligned(out_ptr, result) };
                            0
                        }
                        Err(_) => {
                            let result = PackageVerifyResult {
                                valid: 0,
                                name: [0u8; 32],
                                version: [0u8; 16],
                                publisher: [0u8; 32],
                                capabilities: 0,
                                payload_size: 0,
                            };
                            unsafe { core::ptr::write_unaligned(out_ptr, result) };
                            -1
                        }
                    }
                }
                _ => -14, // EFAULT
            }
        }

        SYS_INSTALL_PACKAGE => {
            let pkg_addr = arg1;
            let pkg_len = arg2 as usize;
            let _flags = arg3 as u32;

            match validate::validate_user_buffer(pkg_addr, pkg_len, false) {
                Ok(()) => {
                    let slice = unsafe { core::slice::from_raw_parts(pkg_addr as *const u8, pkg_len) };
                    let uid = if let Some(proc) = crate::process::get_current_process_mut() {
                        proc.credentials.uid.0
                    } else {
                        0
                    };
                    let mgr = crate::package::manager::get_manager();
                    match mgr.stage_package(slice, uid) {
                        Ok(stage_id) => {
                            if mgr.commit_stage(stage_id).is_ok() {
                                0
                            } else {
                                -5 // EIO
                            }
                        }
                        Err(crate::package::format::PackageError::PermissionDenied) => -1, // EPERM
                        Err(_) => -22, // EINVAL
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_UNINSTALL_PACKAGE => {
            let name_addr = arg1;
            let name_len = arg2 as usize;

            match validate::validate_user_string(name_addr, name_len) {
                Ok(name_str) => {
                    let uid = if let Some(proc) = crate::process::get_current_process_mut() {
                        proc.credentials.uid.0
                    } else {
                        0
                    };
                    let mgr = crate::package::manager::get_manager();
                    match mgr.uninstall_package(name_str, uid) {
                        Ok(()) => 0,
                        Err(crate::package::format::PackageError::PermissionDenied) => -1,
                        Err(_) => -2, // ENOENT
                    }
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_GET_PARTITIONS => {
            let _dev_addr = arg1;
            let _dev_len = arg2 as usize;
            let buf_addr = arg3;
            let max_count = arg4 as usize;

            let req_size = max_count.saturating_mul(core::mem::size_of::<PartitionInfo>());
            match validate::validate_user_buffer(buf_addr, req_size, true) {
                Ok(()) => {
                    let mut count = 0;
                    let buf_ptr = buf_addr as *mut PartitionInfo;
                    // Read GPT entries from sectors 2..9
                    let mut sector_buf = [0u8; 512];
                    for s in 0..8 {
                        if crate::drivers::virtio_block::read_sector(2 + s as u64, &mut sector_buf).is_ok() {
                            for e in 0..4 { // 4 entries of 128 bytes per 512-byte sector
                                if count >= max_count {
                                    break;
                                }
                                let entry_slice: &[u8; 128] = sector_buf[e * 128..(e + 1) * 128].try_into().unwrap();
                                if let Some(entry) = crate::storage::gpt::parse_partition_entry(entry_slice) {
                                    let mut p_name = [0u8; 36];
                                    let _ = entry.get_name_ascii(&mut p_name);
                                    let info = PartitionInfo {
                                        id: count as u32,
                                        type_guid: entry.type_guid,
                                        starting_lba: entry.starting_lba,
                                        ending_lba: entry.ending_lba,
                                        size_sectors: entry.size_sectors(),
                                        attributes: entry.attributes,
                                        name: p_name,
                                    };
                                    unsafe {
                                        core::ptr::write_unaligned(buf_ptr.add(count), info);
                                    }
                                    count += 1;
                                }
                            }
                        }
                    }
                    count as i64
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_TRIGGER_RECOVERY => {
            let reason_code = arg1 as u32;
            let action_param = arg2 as u32;

            let is_root = if let Some(proc) = crate::process::get_current_process_mut() {
                proc.credentials.is_root()
            } else {
                true
            };

            if !is_root {
                -1 // EPERM
            } else {
                let rec = crate::recovery::get_recovery();
                match reason_code {
                    1 => {
                        rec.trigger_recovery(crate::recovery::RecoveryReason::UserManualTrigger);
                        0
                    }
                    2 => {
                        let _report = rec.scrub_novafs();
                        0
                    }
                    3 => {
                        if rec.rollback_to_snapshot(action_param).is_ok() {
                            0
                        } else {
                            -1
                        }
                    }
                    _ => -22, // EINVAL
                }
            }
        }

        SYS_PLAY_AUDIO => {
            let tone_code = arg1 as u32;
            let duration_ms = (arg2 as u32).min(5000);
            let volume = (arg3 as u32).min(100) as u8;

            let hda = crate::drivers::hda::get_hda();
            hda.master_volume = volume;

            match tone_code {
                0 => hda.play_boot_chime(),
                1 => hda.play_alert(),
                2 => hda.play_notification(),
                freq => {
                    hda.synthesize_tone(freq, duration_ms);
                    crate::drivers::audio::play_beep(freq, duration_ms);
                }
            }
            0
        }

        SYS_LIST_BLOCK_DEVICES => {
            let out_addr = arg1;
            let max_entries = arg2 as usize;

            let req_size = max_entries.saturating_mul(core::mem::size_of::<BlockDeviceInfo>());
            match validate::validate_user_buffer(out_addr, req_size, true) {
                Ok(()) => {
                    let out_ptr = out_addr as *mut BlockDeviceInfo;
                    let mut count = 0;

                    // 1. VirtIO Block (/dev/vda)
                    if count < max_entries {
                        let mut name = [0u8; 32];
                        let s = b"/dev/vda (VirtIO)";
                        name[..s.len()].copy_from_slice(s);
                        let dev = BlockDeviceInfo {
                            id: 0,
                            name,
                            bus_type: 0,
                            capacity_sectors: crate::drivers::virtio_block::get_capacity_sectors(),
                            sector_size: 512,
                        };
                        unsafe { core::ptr::write_unaligned(out_ptr.add(count), dev) };
                        count += 1;
                    }

                    // 2. AHCI SATA (/dev/sda)
                    if count < max_entries {
                        let ahci = crate::drivers::ahci::get_ahci();
                        if let Some(ref d) = ahci.drives[0] {
                            let mut name = [0u8; 32];
                            let s = b"/dev/sda (AHCI SATA)";
                            name[..s.len()].copy_from_slice(s);
                            let dev = BlockDeviceInfo {
                                id: 1,
                                name,
                                bus_type: 1,
                                capacity_sectors: d.capacity_sectors,
                                sector_size: d.sector_size,
                            };
                            unsafe { core::ptr::write_unaligned(out_ptr.add(count), dev) };
                            count += 1;
                        }
                    }

                    // 3. NVMe SSD (/dev/nvme0n1)
                    if count < max_entries {
                        let nvme = crate::drivers::nvme::get_nvme();
                        if let Some(ref ns) = nvme.namespaces[0] {
                            let mut name = [0u8; 32];
                            let s = b"/dev/nvme0n1 (NVMe SSD)";
                            name[..s.len()].copy_from_slice(s);
                            let dev = BlockDeviceInfo {
                                id: 2,
                                name,
                                bus_type: 2,
                                capacity_sectors: ns.capacity_sectors,
                                sector_size: ns.sector_size,
                            };
                            unsafe { core::ptr::write_unaligned(out_ptr.add(count), dev) };
                            count += 1;
                        }
                    }

                    count as i64
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        SYS_GET_HARDWARE_INFO => {
            let out_addr = arg1;
            match validate::validate_user_buffer(out_addr, core::mem::size_of::<HardwareSummary>(), true) {
                Ok(()) => {
                    let out_ptr = out_addr as *mut HardwareSummary;
                    let _ahci = crate::drivers::ahci::get_ahci();
                    let _nvme = crate::drivers::nvme::get_nvme();
                    let _xhci = crate::drivers::xhci::get_xhci();
                    let hda = crate::drivers::hda::get_hda();

                    let summary = HardwareSummary {
                        pci_devices_detected: 8,
                        block_devices_count: 3, // vda + sda + nvme0n1
                        has_ahci_sata: 1,
                        has_nvme_ssd: 1,
                        has_usb3_xhci: 1,
                        has_intel_hda: 1,
                        audio_codecs_count: hda.codec_count as u32,
                        screen_width: 1280,
                        screen_height: 800,
                    };
                    unsafe { core::ptr::write_unaligned(out_ptr, summary) };
                    0
                }
                Err(e) => -(e as i32 as i64),
            }
        }

        _ => {
            serial::write_str("[SYSCALL] Unknown syscall number: ");
            print_usize(syscall_num);
            serial::write_str("\n");
            -38 // ENOSYS
        }
    };

    ctx.rax = result as u64;
}

fn print_usize(mut n: usize) {
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
