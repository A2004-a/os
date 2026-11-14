//! NovaOS Apex — Native Application Suite Library (Milestone 9)

#![no_std]

extern crate novaos_apex_runtime;
extern crate novaos_apex_toolkit;

pub mod devicemanager;
pub mod files;
pub mod installer;
pub mod monitor;
pub mod packages;
pub mod recovery;
pub mod settings;
pub mod terminal;

pub use devicemanager::NovaDeviceManagerApp;
pub use files::NovaFilesApp;
pub use installer::NovaInstallerApp;
pub use monitor::NovaMonitorApp;
pub use packages::NovaPackagesApp;
pub use recovery::NovaRecoveryApp;
pub use settings::NovaSettingsApp;
pub use terminal::NovaTerminalApp;
