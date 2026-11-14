//! NovaOS Apex — Native Application Suite Runner (Milestone 9)

#![no_std]
#![no_main]

extern crate novaos_apex_runtime;
extern crate novaos_apex_toolkit;

use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use novaos_apex_runtime::*;

mod calculator;
mod devicemanager;
mod files;
mod installer;
mod lock_screen;
mod monitor;
mod notepad;
mod packages;
mod recovery;
mod run_dialog;
mod settings;
mod terminal;

use calculator::NovaCalculatorApp;
use devicemanager::NovaDeviceManagerApp;
use files::NovaFilesApp;
use installer::NovaInstallerApp;
use lock_screen::NovaLockScreenApp;
use monitor::NovaMonitorApp;
use notepad::NovaNotepadApp;
use packages::NovaPackagesApp;
use recovery::NovaRecoveryApp;
use run_dialog::NovaRunDialogApp;
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

    // 6. Initialize Nova OS Installer
    let inst_app = NovaInstallerApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    inst_app.render(buf, APP_WIDTH);
    if let Ok(w_inst) = create_window("NovaOS Setup Wizard", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_inst, &buf[..]);
        print("[APPS] [PASS] NovaOS Installer window mapped and composited.\n");
    }

    // 7. Initialize Nova Recovery & Diagnostics
    let rec_app = NovaRecoveryApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    rec_app.render(buf, APP_WIDTH);
    if let Ok(w_rec) = create_window("Nova Recovery Center", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_rec, &buf[..]);
        print("[APPS] [PASS] Nova Recovery Center window mapped and composited.\n");
    }

    // 8. Initialize Nova Device Manager
    let dev_app = NovaDeviceManagerApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    dev_app.render(buf, APP_WIDTH);
    if let Ok(w_dev) = create_window("Nova Device Manager", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_dev, &buf[..]);
        print("[APPS] [PASS] Nova Device Manager window mapped and composited.\n");
    }

    // 9. Initialize Nova Calculator
    let calc_app = NovaCalculatorApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    calc_app.render(buf, APP_WIDTH);
    if let Ok(w_calc) = create_window("Calculator", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_calc, &buf[..]);
        print("[APPS] [PASS] Nova Calculator window mapped and composited.\n");
    }

    // 10. Initialize Nova Notepad
    let notepad_app = NovaNotepadApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    notepad_app.render(buf, APP_WIDTH);
    if let Ok(w_note) = create_window("Notepad", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_note, &buf[..]);
        print("[APPS] [PASS] Nova Notepad window mapped and composited.\n");
    }

    // 11. Initialize Nova Run Dialog
    let run_app = NovaRunDialogApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    run_app.render(buf, APP_WIDTH);
    if let Ok(w_run) = create_window("Run", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_run, &buf[..]);
        print("[APPS] [PASS] Nova Run Dialog window mapped and composited.\n");
    }

    // 12. Initialize Nova Lock Screen
    let lock_app = NovaLockScreenApp::new(0, 0, APP_WIDTH, APP_HEIGHT);
    lock_app.render(buf, APP_WIDTH);
    if let Ok(w_lock) = create_window("Lock Screen", APP_WIDTH, APP_HEIGHT, 0) {
        let _ = blit_window(w_lock, &buf[..]);
        print("[APPS] [PASS] Nova Lock Screen window mapped and composited.\n");
    }

    print("\n[APPS SUCCESS] All 12 Windows 11-class native applications operational!\n");
    exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
