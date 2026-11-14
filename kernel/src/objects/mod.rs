//! NovaOS Apex — Kernel Object Subsystem

pub mod handle;
pub mod object;
pub mod tests;
pub mod types;

pub use handle::{Handle, HandleError, HandleTable};
pub use object::{KernelObject, ObjectPayload};
pub use types::{CapabilityRights, ObjectType};
