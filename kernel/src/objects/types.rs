//! NovaOS Apex — Kernel Object Types and Capability Rights
//! Defines strongly-typed categories for all kernel resources and access control permissions.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ObjectType {
    Process = 1,
    Thread = 2,
    File = 3,
    Socket = 4,
    Event = 5,
    Device = 6,
    Window = 7,
    SharedMemory = 8,
}

impl ObjectType {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Process => "Process",
            Self::Thread => "Thread",
            Self::File => "File",
            Self::Socket => "Socket",
            Self::Event => "Event",
            Self::Device => "Device",
            Self::Window => "Window",
            Self::SharedMemory => "SharedMemory",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct CapabilityRights(pub u32);

impl CapabilityRights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const TRANSFER: Self = Self(1 << 3);
    pub const DUPLICATE: Self = Self(1 << 4);
    pub const CLOSE: Self = Self(1 << 5);
    pub const MAP: Self = Self(1 << 6);
    pub const SIGNAL: Self = Self(1 << 7);
    pub const WAIT: Self = Self(1 << 8);

    pub const ALL: Self = Self(
        Self::READ.0
            | Self::WRITE.0
            | Self::EXECUTE.0
            | Self::TRANSFER.0
            | Self::DUPLICATE.0
            | Self::CLOSE.0
            | Self::MAP.0
            | Self::SIGNAL.0
            | Self::WAIT.0,
    );

    #[inline(always)]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub const fn union(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn intersect(&self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
