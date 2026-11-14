//! NovaOS Apex — Native Application Suite Library (Milestone 9)

#![no_std]

extern crate novaos_apex_runtime;
extern crate novaos_apex_toolkit;

pub mod files;
pub mod monitor;
pub mod packages;
pub mod settings;
pub mod terminal;

pub use files::NovaFilesApp;
pub use monitor::NovaMonitorApp;
pub use packages::NovaPackagesApp;
pub use settings::NovaSettingsApp;
pub use terminal::NovaTerminalApp;
