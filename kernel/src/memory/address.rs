//! NovaOS Apex — Strongly-Typed Physical and Virtual Memory Addresses
//! Prevents address confusion between physical frames and virtual pages.

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xFFFF_8000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    #[inline(always)]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn as_ptr<T>(self) -> *const T {
        self.to_virt().as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.to_virt().as_mut_ptr()
    }

    #[inline(always)]
    pub const fn is_aligned(self, align: u64) -> bool {
        (self.0 & (align - 1)) == 0
    }

    #[inline(always)]
    pub const fn is_page_aligned(self) -> bool {
        self.is_aligned(PAGE_SIZE as u64)
    }

    #[inline(always)]
    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    #[inline(always)]
    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }

    #[inline(always)]
    pub const fn to_virt(self) -> VirtAddr {
        VirtAddr(self.0 + PHYSICAL_MEMORY_OFFSET)
    }
}

impl core::fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysAddr(0x{:016X})", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    #[inline(always)]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    #[inline(always)]
    pub const fn is_aligned(self, align: u64) -> bool {
        (self.0 & (align - 1)) == 0
    }

    #[inline(always)]
    pub const fn is_page_aligned(self) -> bool {
        self.is_aligned(PAGE_SIZE as u64)
    }

    #[inline(always)]
    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    #[inline(always)]
    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }

    #[inline(always)]
    pub const fn p4_index(self) -> usize {
        ((self.0 >> 39) & 0x1FF) as usize
    }

    #[inline(always)]
    pub const fn p3_index(self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }

    #[inline(always)]
    pub const fn p2_index(self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }

    #[inline(always)]
    pub const fn p1_index(self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }

    #[inline(always)]
    pub const fn page_offset(self) -> u64 {
        self.0 & 0xFFF
    }

    #[inline(always)]
    pub const fn to_phys(self) -> Option<PhysAddr> {
        if self.0 >= PHYSICAL_MEMORY_OFFSET {
            Some(PhysAddr(self.0 - PHYSICAL_MEMORY_OFFSET))
        } else {
            None
        }
    }
}

impl core::fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtAddr(0x{:016X})", self.0)
    }
}
