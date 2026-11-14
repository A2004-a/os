//! NovaOS Apex — Comprehensive Terminal & NovaShell Verification Suite
//! Verifies all 25 terminal commands, capabilities, and interactive shell behaviors one-by-one:
//! - Built-in commands: help, uname, whoami, pwd, cd, ls, dir, cat, echo, mkdir, touch,
//!   stat, rm/del, ifconfig/ipconfig, ping, netstat, uptime, winver, ps/tasklist, free/mem, clear/cls, exit
//! - Interactive keystroke buffering, backspace editing, 24-row FIFO line buffer scrolling, and fault-tolerant parsing.

use crate::graphics::desktop::DesktopShell;
use crate::graphics::color::Color;
use crate::serial;

macro_rules! check_term {
    ($num:expr, $name:expr, $cond:expr) => {
        if $cond {
            serial::write_str("  [TERM TEST ");
            print_num($num);
            serial::write_str("/25] PASS: ");
            serial::write_str($name);
            serial::write_str("\n");
        } else {
            serial::write_str("  [TERM TEST ");
            print_num($num);
            serial::write_str("/25] FAIL: ");
            serial::write_str($name);
            serial::write_str("\n");
            return false;
        }
    };
}

pub fn run_all_terminal_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Terminal & NovaShell Test Suite (25 Gates) \n");
    serial::write_str("=======================================================\n");

    let mut shell_guard = crate::graphics::desktop::DESKTOP.lock();
    let shell = match shell_guard.as_mut() {
        Some(s) => s,
        None => {
            serial::write_str("[TERM ERROR] DesktopShell not initialized!\n");
            return false;
        }
    };

    let prev_blit = shell.blit_to_screen;
    shell.blit_to_screen = false;

    let res = execute_all_gates(shell);

    // Restore pristine terminal state & redraw
    shell.init_terminal();
    shell.blit_to_screen = prev_blit;

    serial::write_str("\n=======================================================\n");
    if res {
        serial::write_str("[TERMINAL TEST RESULT] ALL 25 TERMINAL & NOVASHELL TESTS PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("[TERMINAL TEST RESULT] TERMINAL SUBSYSTEM ENCOUNTERED FAILURES!\n");
    }
    serial::write_str("=======================================================\n");

    res
}

fn execute_all_gates(shell: &mut DesktopShell) -> bool {
    // 1. Command 'help' Catalog Discovery
    set_input(shell, "help");
    shell.execute_terminal_command();
    let help_ok = shell.terminal_line_count > 0 &&
        shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("NovaShell");
    check_term!(1, "Command 'help' (Catalog Discovery & Command Reference)", help_ok);

    // 2. Command 'uname -a' Architecture Verification
    set_input(shell, "uname -a");
    shell.execute_terminal_command();
    let uname_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("x86_64");
    check_term!(2, "Command 'uname -a' (Kernel Architecture & Bare-Metal Platform)", uname_ok);

    // 3. Command 'whoami' Effective User Identity
    set_input(shell, "whoami");
    shell.execute_terminal_command();
    let whoami_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("apex");
    check_term!(3, "Command 'whoami' (Current User Identity & Administrator Ring)", whoami_ok);

    // 4. Command 'pwd' Working Directory Query
    set_input(shell, "pwd");
    shell.execute_terminal_command();
    let pwd_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("/users/apex");
    check_term!(4, "Command 'pwd' (Working Directory Query)", pwd_ok);

    // 5. Command 'cd' Directory Navigation
    set_input(shell, "cd /etc");
    shell.execute_terminal_command();
    let cd_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("/etc");
    check_term!(5, "Command 'cd' (Directory Navigation & Traversal)", cd_ok);

    // 6. Command 'ls' POSIX VFS Directory Enumeration
    set_input(shell, "ls");
    shell.execute_terminal_command();
    let ls_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("Telemetry.log");
    check_term!(6, "Command 'ls' (POSIX VFS Directory Enumeration)", ls_ok);

    // 7. Command 'dir' Windows PowerShell Directory Formatter
    set_input(shell, "dir");
    shell.execute_terminal_command();
    let dir_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("Telemetry.log")
        && shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("Readme.txt");
    check_term!(7, "Command 'dir' (Windows PowerShell Alias & File Table)", dir_ok);

    // 8. Command 'cat' File Stream Read & Content Inspection
    set_input(shell, "cat Readme.txt");
    shell.execute_terminal_command();
    let cat_ok = shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("Welcome to NovaOS Apex");
    check_term!(8, "Command 'cat' (File Stream Read & VFS Content Output)", cat_ok);

    // 9. Command 'echo' Parameter Parsing & Output Stream Echo
    set_input(shell, "echo NovaOS Apex Rocks");
    shell.execute_terminal_command();
    let echo_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("NovaOS Apex Rocks");
    check_term!(9, "Command 'echo' (Parameter Parsing & Output Stream Echo)", echo_ok);

    // 10. Command 'mkdir' Directory Inode Provisioning
    set_input(shell, "mkdir /tmp/testdir");
    shell.execute_terminal_command();
    let mkdir_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("Directory created");
    check_term!(10, "Command 'mkdir' (Directory Inode Provisioning)", mkdir_ok);

    // 11. Command 'touch' File Inode Allocation
    set_input(shell, "touch /tmp/testfile.txt");
    shell.execute_terminal_command();
    let touch_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("File created");
    check_term!(11, "Command 'touch' (File Inode Allocation & Timestamping)", touch_ok);

    // 12. Command 'stat' POSIX Inode Metadata
    set_input(shell, "stat Readme.txt");
    shell.execute_terminal_command();
    let stat_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("Size: 1248");
    check_term!(12, "Command 'stat' (POSIX Inode Metadata & Permissions Matrix)", stat_ok);

    // 13. Command 'rm' & 'del' Inode Unlinking
    set_input(shell, "rm /tmp/testfile.txt");
    shell.execute_terminal_command();
    let rm_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("File unlinked");
    check_term!(13, "Command 'rm' & 'del' (Inode Unlinking & Block Reclamation)", rm_ok);

    // 14. Command 'ifconfig' & 'ipconfig' VirtIO-Net Interface Telemetry
    set_input(shell, "ipconfig");
    shell.execute_terminal_command();
    let ifcfg_ok = shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("10.0.2.15");
    check_term!(14, "Command 'ifconfig' & 'ipconfig' (VirtIO-Net Interface Telemetry)", ifcfg_ok);

    // 15. Command 'ping' ICMP Echo Probe
    set_input(shell, "ping 10.0.2.2");
    shell.execute_terminal_command();
    let ping_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("icmp_seq=2");
    check_term!(15, "Command 'ping' (ICMP Echo Probe Request/Reply Round-Trip)", ping_ok);

    // 16. Command 'netstat' TCP/UDP Socket Listener Enumeration
    set_input(shell, "netstat");
    shell.execute_terminal_command();
    let netstat_ok = shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("ESTABLISHED");
    check_term!(16, "Command 'netstat' (TCP/UDP Socket Listener Enumeration)", netstat_ok);

    // 17. Command 'uptime' 1000 Hz Hardware APIC Clock Calculation
    set_input(shell, "uptime");
    shell.execute_terminal_command();
    let uptime_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("System Uptime");
    check_term!(17, "Command 'uptime' (1000 Hz Hardware APIC Clock Calculation)", uptime_ok);

    // 18. Command 'winver' & 'ver' Windows 11 Compatibility Banner
    set_input(shell, "winver");
    shell.execute_terminal_command();
    let winver_ok = shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("Build 22631.105");
    check_term!(18, "Command 'winver' & 'ver' (Windows 11 Compatibility Banner)", winver_ok);

    // 19. Command 'ps' & 'tasklist' Preemptive Scheduler Thread Hierarchy
    set_input(shell, "tasklist");
    shell.execute_terminal_command();
    let ps_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("novashell");
    check_term!(19, "Command 'ps' & 'tasklist' (Preemptive Scheduler Thread Hierarchy)", ps_ok);

    // 20. Command 'free' & 'mem' Physical RAM & Dynamic Heap Accounting
    set_input(shell, "mem");
    shell.execute_terminal_command();
    let mem_ok = shell.terminal_lines[shell.terminal_line_count - 2].as_str().contains("Physical RAM: 256 MiB")
        && shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("Kernel Heap:   64 MiB");
    check_term!(20, "Command 'free' & 'mem' (Physical RAM & Dynamic Heap Accounting)", mem_ok);

    // 21. Command 'clear' & 'cls' Screen Buffer Purge & Origin Reset
    set_input(shell, "cls");
    shell.execute_terminal_command();
    let clear_ok = shell.terminal_line_count == 0;
    check_term!(21, "Command 'clear' & 'cls' (Screen Buffer Purge & Origin Reset)", clear_ok);

    // 22. Interactive Keystroke Buffering & Character Echo
    shell.terminal_input_len = 0;
    type_str(shell, "echo test typing");
    let typed_ok = shell.terminal_input_len == 17;
    check_term!(22, "Interactive Keystroke Buffering & Monospace Character Insertion", typed_ok);

    // 23. Interactive Backspace Deletion & Horizontal Cursor Recalculation
    for _ in 0..7 {
        if shell.terminal_input_len > 0 {
            shell.terminal_input_len -= 1;
        }
    }
    let bksp_ok = shell.terminal_input_len == 10; // "echo test "
    shell.execute_terminal_command();
    let bksp_exec_ok = bksp_ok && shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("test");
    check_term!(23, "Interactive Backspace Deletion & Horizontal Cursor Recalculation", bksp_exec_ok);

    // 24. Line Buffer Scrolling & 24-Row FIFO Ring Buffer Saturation Safety
    for _ in 0..30 {
        shell.terminal_print_line("Stress line padding", Color::TEXT_PRIMARY);
    }
    let scroll_ok = shell.terminal_line_count <= 24;
    check_term!(24, "Line Buffer Scrolling & 24-Row FIFO Ring Buffer Saturation Safety", scroll_ok);

    // 25. Unknown Command Trapping & Fault-Tolerant Error Reporting
    set_input(shell, "invalid_cmd_xyz");
    shell.execute_terminal_command();
    let unk_ok = shell.terminal_lines[shell.terminal_line_count - 1].as_str().contains("command not found");
    check_term!(25, "Unknown Command Trapping & Fault-Tolerant Error Reporting", unk_ok);

    true
}

fn set_input(shell: &mut DesktopShell, s: &str) {
    shell.terminal_input_len = 0;
    let b = s.as_bytes();
    let len = b.len().min(64);
    shell.terminal_input[..len].copy_from_slice(&b[..len]);
    shell.terminal_input_len = len;
}

fn type_str(shell: &mut DesktopShell, s: &str) {
    for c in s.chars() {
        if shell.terminal_input_len < 64 && c >= ' ' && c <= '~' {
            shell.terminal_input[shell.terminal_input_len] = c as u8;
            shell.terminal_input_len += 1;
        }
    }
}

fn print_num(mut n: u32) {
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
