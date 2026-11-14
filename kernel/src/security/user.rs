//! NovaOS Zero — Multi-User Security Model & Account Management
//! Enforces UID/GID privilege isolation, capability boundaries, and user authentication.

use super::CapabilityFlags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserAccount {
    pub uid: u32,
    pub gid: u32,
    pub username: &'static str,
    pub password_hash: &'static str,
    pub home_dir: &'static str,
    pub shell: &'static str,
    pub capabilities: CapabilityFlags,
}

pub const ALL_CAPABILITIES: CapabilityFlags = CapabilityFlags(
    CapabilityFlags::READ.0
        | CapabilityFlags::WRITE.0
        | CapabilityFlags::EXEC.0
        | CapabilityFlags::SPAWN.0
        | CapabilityFlags::NETWORK.0
        | CapabilityFlags::STORAGE.0,
);

pub const STANDARD_CAPABILITIES: CapabilityFlags = CapabilityFlags(
    CapabilityFlags::READ.0
        | CapabilityFlags::WRITE.0
        | CapabilityFlags::EXEC.0
        | CapabilityFlags::SPAWN.0
        | CapabilityFlags::NETWORK.0,
);

pub const GUEST_CAPABILITIES: CapabilityFlags = CapabilityFlags(
    CapabilityFlags::READ.0 | CapabilityFlags::EXEC.0,
);

pub static PRECONFIGURED_USERS: [UserAccount; 3] = [
    UserAccount {
        uid: 0,
        gid: 0,
        username: "root",
        password_hash: "novaroot2026",
        home_dir: "/root",
        shell: "/bin/shell",
        capabilities: ALL_CAPABILITIES,
    },
    UserAccount {
        uid: 1000,
        gid: 1000,
        username: "nova",
        password_hash: "novapass",
        home_dir: "/home/nova",
        shell: "/bin/shell",
        capabilities: STANDARD_CAPABILITIES,
    },
    UserAccount {
        uid: 1001,
        gid: 1001,
        username: "guest",
        password_hash: "guest",
        home_dir: "/home/guest",
        shell: "/bin/shell",
        capabilities: GUEST_CAPABILITIES,
    },
];

pub struct UserManager;

impl UserManager {
    pub fn find_by_name(name: &str) -> Option<&'static UserAccount> {
        for user in PRECONFIGURED_USERS.iter() {
            if user.username == name {
                return Some(user);
            }
        }
        None
    }

    pub fn find_by_uid(uid: u32) -> Option<&'static UserAccount> {
        for user in PRECONFIGURED_USERS.iter() {
            if user.uid == uid {
                return Some(user);
            }
        }
        None
    }

    pub fn authenticate(username: &str, password: &str) -> Option<&'static UserAccount> {
        if let Some(user) = Self::find_by_name(username) {
            if user.password_hash == password {
                return Some(user);
            }
        }
        None
    }

    pub fn is_root(uid: u32) -> bool {
        uid == 0
    }
}
