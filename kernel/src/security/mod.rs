//! NovaOS Apex — Security Subsystem
//! Implements user isolation, credentials, and access control.

pub mod user;

pub use user::{
    check_permission, Credentials, Gid, Uid, DEFAULT_USER_GID, DEFAULT_USER_UID, ROOT_GID,
    ROOT_UID, SYSTEM_GID, SYSTEM_UID, S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFMT, S_IFREG,
    S_IRGRP, S_IROTH, S_IRUSR, S_IRWXG, S_IRWXO, S_IRWXU, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP,
    S_IXOTH, S_IXUSR,
};
