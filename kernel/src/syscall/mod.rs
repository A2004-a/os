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

#[no_mangle]
pub fn handle_syscall(ctx: &mut ExceptionContext) {
    let syscall_num = ctx.rax as usize;
    let arg1 = ctx.rdi;
    let arg2 = ctx.rsi;
    let arg3 = ctx.rdx;
    let _arg4 = ctx.r10;
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
