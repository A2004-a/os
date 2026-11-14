//! NovaOS Apex — User Security and Credentials Model
//! Manages User IDs, Group IDs, process security credentials, and POSIX permission verification.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uid(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gid(pub u32);

pub const ROOT_UID: Uid = Uid(0);
pub const SYSTEM_UID: Uid = Uid(1);
pub const DEFAULT_USER_UID: Uid = Uid(1000);

pub const ROOT_GID: Gid = Gid(0);
pub const SYSTEM_GID: Gid = Gid(1);
pub const DEFAULT_USER_GID: Gid = Gid(1000);

pub const S_IRUSR: u32 = 0o400;
pub const S_IWUSR: u32 = 0o200;
pub const S_IXUSR: u32 = 0o100;
pub const S_IRWXU: u32 = 0o700;

pub const S_IRGRP: u32 = 0o040;
pub const S_IWGRP: u32 = 0o020;
pub const S_IXGRP: u32 = 0o010;
pub const S_IRWXG: u32 = 0o070;

pub const S_IROTH: u32 = 0o004;
pub const S_IWOTH: u32 = 0o002;
pub const S_IXOTH: u32 = 0o001;
pub const S_IRWXO: u32 = 0o007;

pub const S_IFMT: u32 = 0o170000;
pub const S_IFREG: u32 = 0o100000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFIFO: u32 = 0o010000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Credentials {
    pub uid: Uid,
    pub gid: Gid,
    pub euid: Uid,
    pub egid: Gid,
}

impl Credentials {
    pub const fn root() -> Self {
        Self {
            uid: ROOT_UID,
            gid: ROOT_GID,
            euid: ROOT_UID,
            egid: ROOT_GID,
        }
    }

    pub const fn user(uid: u32, gid: u32) -> Self {
        Self {
            uid: Uid(uid),
            gid: Gid(gid),
            euid: Uid(uid),
            egid: Gid(gid),
        }
    }

    pub fn is_root(&self) -> bool {
        self.euid == ROOT_UID
    }
}

/// Evaluates file permissions based on mode bits, ownership, and credentials.
pub fn check_permission(
    mode: u32,
    owner: Uid,
    group: Gid,
    creds: &Credentials,
    want_read: bool,
    want_write: bool,
    want_exec: bool,
) -> bool {
    // Root bypasses read and write permissions; execute requires at least one execute bit
    if creds.euid == ROOT_UID {
        if want_exec {
            return (mode & (S_IXUSR | S_IXGRP | S_IXOTH)) != 0;
        }
        return true;
    }

    let (r_bit, w_bit, x_bit) = if creds.euid == owner {
        (S_IRUSR, S_IWUSR, S_IXUSR)
    } else if creds.egid == group {
        (S_IRGRP, S_IWGRP, S_IXGRP)
    } else {
        (S_IROTH, S_IWOTH, S_IXOTH)
    };

    if want_read && (mode & r_bit) == 0 {
        return false;
    }
    if want_write && (mode & w_bit) == 0 {
        return false;
    }
    if want_exec && (mode & x_bit) == 0 {
        return false;
    }

    true
}
