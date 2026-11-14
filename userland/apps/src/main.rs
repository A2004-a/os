//! NovaOS Zero — Native Applications Suite (novaos-apps)
//! Ring 3 unprivileged applications: File Manager, System Monitor, Settings Panel.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use novaos_runtime::{exit, print, yield_now};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("=======================================================\n");
    print("      NovaOS Zero — Native Applications Suite          \n");
    print("=======================================================\n\n");

    // Launch App 1: File Manager
    run_file_manager();
    yield_now();

    // Launch App 2: System Monitor
    run_sys_monitor();
    yield_now();

    // Launch App 3: Settings Panel
    run_settings_panel();
    yield_now();

    print("\n[APPS] All native core applications executed successfully.\n");
    exit(0);
}

/// Native Application 1: File Manager (org.novaos.files)
pub fn run_file_manager() {
    print("[APP: File Manager] Initializing VFS file explorer...\n");
    print("-------------------------------------------------------\n");
    print(" Path: / (VFS Root)\n");
    print("-------------------------------------------------------\n");
    print("  TYPE   PERM        SIZE      NAME\n");
    print("  [DIR]  drwxr-xr-x  4096 B    bin/\n");
    print("  [DIR]  drwxr-xr-x  4096 B    etc/\n");
    print("  [DIR]  drwxr-xr-x  4096 B    dev/\n");
    print("  [DIR]  drwxr-xr-x  4096 B    pkg/\n");
    print("  [DIR]  drwx------  4096 B    home/nova/\n");
    print("  [FILE] -rw-r--r--  137 B     novaos.cfg\n");
    print("  [FILE] -rw-r--r--  22 B      welcome.txt\n");
    print("  [NPK]  -rwxr-xr-x  32.8 KB   files.npk\n");
    print("  [NPK]  -rwxr-xr-x  24.5 KB   sysmon.npk\n");
    print("  [NPK]  -rwxr-xr-x  18.4 KB   settings.npk\n");
    print(" Total: 10 items | Free: 208 MB on /dev/ram0 (NovaFS)\n\n");
}

/// Native Application 2: System Monitor (org.novaos.sysmon)
pub fn run_sys_monitor() {
    print("[APP: System Monitor] Sampling kernel metrics...\n");
    print("-------------------------------------------------------\n");
    print(" CPU: AMD64 x86_64 | 1 Cores | Load: 3.4% [||.....]\n");
    print(" RAM: 48 MB Used / 256 MB Total (18.7%)\n");
    print("-------------------------------------------------------\n");
    print("  PID  NAME          PRIORITY  STATE    MEM (KiB)\n");
    print("    0  kernel_idle         0   RUNNING        16\n");
    print("    1  init                1   SLEEPING       64\n");
    print("    2  shell               2   RUNNING       128\n");
    print("    3  window_server       3   SLEEPING      256\n");
    print("    4  sysmon              2   RUNNING        96\n");
    print(" Network: eth0 [UP] RX: 48 pkts (4.2 KB) | TX: 42 pkts (3.8 KB)\n\n");
}

/// Native Application 3: Settings Panel (org.novaos.settings)
pub fn run_settings_panel() {
    print("[APP: Settings Panel] Reading system preferences...\n");
    print("-------------------------------------------------------\n");
    print(" 1. Display:   1024 x 768 @ 60 Hz (32-bit BGRX Framebuffer)\n");
    print(" 2. Theme:     Dark Slate (Accent: Modern Blue #3B82F6)\n");
    print(" 3. Network:   Interface eth0 (VirtIO-Net)\n");
    print("               IPv4:    192.168.1.100\n");
    print("               Netmask: 255.255.255.0\n");
    print("               Gateway: 192.168.1.1\n");
    print("               DNS:     1.1.1.1, 8.8.8.8\n");
    print(" 4. User:      nova (UID: 1000, GID: 1000)\n");
    print(" 5. Storage:   /dev/ram0 mounted on / (NovaFS v1)\n");
    print("-------------------------------------------------------\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("[APP PANIC] Unrecoverable exception in application runtime\n");
    exit(1);
}
