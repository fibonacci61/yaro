use core::{
    ops::{Add, Sub},
    ptr::NonNull,
};

pub static mut PHYS_OFFSET: PhysAddr = PhysAddr(0);

pub trait Addr: Copy + Eq + Ord {
    fn try_new(addr: usize) -> Option<Self>;
    fn as_usize(self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

impl Addr for VirtAddr {
    fn try_new(addr: usize) -> Option<Self> {
        Self::try_new(addr)
    }

    fn as_usize(self) -> usize {
        self.0
    }
}

impl Addr for PhysAddr {
    fn try_new(addr: usize) -> Option<Self> {
        Some(Self(addr))
    }

    fn as_usize(self) -> usize {
        self.0
    }
}

impl PhysAddr {
    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub const fn ppn(self) -> usize {
        self.0 >> 12 & 0xfffffffffff
    }

    pub const fn ppn0(self) -> usize {
        self.0 >> 12 & 0o777
    }

    pub const fn ppn1(self) -> usize {
        self.0 >> 21 & 0o777
    }

    pub const fn ppn2(self) -> usize {
        // mask is for first 26 bits
        self.0 >> 30 & 0x3ffffff
    }

    pub fn from_virt(virt: VirtAddr) -> Self {
        Self(virt.as_usize() - unsafe { PHYS_OFFSET }.as_usize())
    }
}

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.as_usize()
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.as_usize() + rhs)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.as_usize() - rhs)
    }
}

impl Sub for PhysAddr {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.as_usize() - rhs.as_usize()
    }
}

impl VirtAddr {
    const fn new_truncate(addr: usize) -> Self {
        Self(((addr << 25) as isize >> 25) as usize)
    }

    pub const fn try_new(addr: usize) -> Option<Self> {
        if Self::new_truncate(addr).0 == addr {
            Some(Self(addr))
        } else {
            None
        }
    }

    pub const fn new(addr: usize) -> Self {
        Self::try_new(addr).expect("non-canonical virtual address")
    }

    pub fn from_phys(phys: PhysAddr) -> Self {
        Self(unsafe { PHYS_OFFSET }.as_usize() + phys.as_usize())
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub const fn as_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    pub const fn as_non_null<T>(self) -> Option<NonNull<T>> {
        NonNull::new(self.0 as *mut T)
    }

    pub const fn vpn0(self) -> usize {
        self.0 >> 12 & 0o777
    }

    pub const fn vpn1(self) -> usize {
        self.0 >> 21 & 0o777
    }

    pub const fn vpn2(self) -> usize {
        self.0 >> 30 & 0o777
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}

// impl<T: Into<usize>> Add<T> for VirtAddr {
//     type Output = Self;

//     fn add(self, rhs: T) -> Self::Output {
//         Self(self.0 + )
//     }
// }
