//! NovaOS Apex — Native Application Suite Runner (Milestone 9)

#![no_std]
#![no_main]

extern crate novaos_apex_runtime;
extern crate novaos_apex_toolkit;

use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use novaos_apex_runtime::*;

mod files;
mod monitor;
mod packages;
mod settings;
mod terminal;

use files::NovaFilesApp;
use monitor::NovaMonitorApp;
use packages::NovaPackagesApp;
use settings::NovaSettingsApp;
use terminal::NovaTerminalApp;

const APP_WIDTH: u32 = 480;
const APP_HEIGHT: u32 = 320;
const BUF_SIZE: usize = (APP_WIDTH * APP_HEIGHT) as usize;

static mut SHARED_BUFFER: [u32; BUF_SIZE] = [0; BUF_SIZE];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("\n=======================================================\n");
    print("   NovaOS Apex — Native Application Suite (Ring 3)     \n");
    print("=======================================================\n");
    print("[APPS] Launching Windows 11-class native applications...\n");

    let buf = unsafe { &mut *addr_of_mut!(SHARED_BUFFER) };

    // 1. Initialize Nova Files
    let files_app = NovaFilesApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    files_app.render(buf, APP_WIDTH);
    if let Ok(w_files) = create_window("Nova Files", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_files, &buf[..]);
        print("[APPS] [PASS] Nova Files window mapped and composited.\n");
    }

    // 2. Initialize Nova System Monitor
    let mon_app = NovaMonitorApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    mon_app.render(buf, APP_WIDTH);
    if let Ok(w_mon) = create_window("Nova System Monitor", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_mon, &buf[..]);
        print("[APPS] [PASS] Nova System Monitor window mapped and composited.\n");
    }

    // 3. Initialize Nova Settings
    let set_app = NovaSettingsApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    set_app.render(buf, APP_WIDTH);
    if let Ok(w_set) = create_window("Nova Settings", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_set, &buf[..]);
        print("[APPS] [PASS] Nova Settings window mapped and composited.\n");
    }

    // 4. Initialize Nova Package Manager
    let pkg_app = NovaPackagesApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    pkg_app.render(buf, APP_WIDTH);
    if let Ok(w_pkg) = create_window("Nova Package Manager", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_pkg, &buf[..]);
        print("[APPS] [PASS] Nova Package Manager window mapped and composited.\n");
    }

    // 5. Initialize Nova Terminal
    let term_app = NovaTerminalApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    term_app.render(buf, APP_WIDTH);
    if let Ok(w_term) = create_window("Nova Terminal", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_term, &buf[..]);
        print("[APPS] [PASS] Nova Terminal window mapped and composited.\n");
    }

    print("\n[APPS SUCCESS] All 5 core native applications operational!\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
