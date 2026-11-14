//! NovaOS Zero — Filesystem Subsystem (VFS, NovaFS, RamFS)

pub mod fs_test;
pub mod novafs;
pub mod ramfs;
pub mod vfs;

pub use novafs::NovaFs;
pub use ramfs::{RamFile, RamFs};
pub use vfs::{FileOperations, FsError, NodeType, VirtualFileSystem, VFS};
