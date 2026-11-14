//! NovaOS Apex — Native Application Suite Library (Milestone 9)

#![no_std]

extern crate novaos_apex_runtime;
extern crate novaos_apex_toolkit;

pub mod calculator;
pub mod devicemanager;
pub mod files;
pub mod installer;
pub mod lock_screen;
pub mod monitor;
pub mod notepad;
pub mod packages;
pub mod recovery;
pub mod run_dialog;
pub mod settings;
pub mod terminal;

pub use calculator::NovaCalculatorApp;
pub use devicemanager::NovaDeviceManagerApp;
pub use files::NovaFilesApp;
pub use installer::NovaInstallerApp;
pub use lock_screen::NovaLockScreenApp;
pub use monitor::NovaMonitorApp;
pub use notepad::NovaNotepadApp;
pub use packages::NovaPackagesApp;
pub use recovery::NovaRecoveryApp;
pub use run_dialog::NovaRunDialogApp;
pub use settings::NovaSettingsApp;
pub use terminal::NovaTerminalApp;
