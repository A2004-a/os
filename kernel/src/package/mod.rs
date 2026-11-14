//! NovaOS Apex — Package Subsystem
//! Manages Signed Nova Packages (.nvp), digital signature trust, capability sandboxing,
//! transactional staging, registry database, and atomic rollback.

pub mod crypto;
pub mod format;
pub mod manager;
pub mod store_tests;

pub fn init() {
    manager::init();
}
