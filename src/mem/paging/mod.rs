pub mod entry;
pub mod table;
use core::marker::PhantomData;

use super::addr::{Addr, VirtAddr};
use crate::mem::addr::PhysAddr;

pub struct Page<A: Addr> {
    index: usize,
    ty: PageType,
    _phantom: PhantomData<A>,
}

pub type PhysPage = Page<PhysAddr>;
pub type VirtPage = Page<VirtAddr>;

#[derive(Clone, Copy)]
pub enum PageType {
    /// 4 KiB page
    Base,
    /// 2 MiB page
    Mega,
    /// 1 GiB page
    Giga,
}

impl PageType {
    pub const fn size(self) -> usize {
        match self {
            Self::Base => 0x1000,
            Self::Mega => 0x200_0000,
            Self::Giga => 0x4000_0000,
        }
    }
}

impl<A: Addr> Page<A> {
    pub fn try_new(index: usize, ty: PageType) -> Option<Self> {
        if A::try_new(index * ty.size()).is_some() {
            Some(Self {
                index,
                ty,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn new(index: usize, ty: PageType) -> Self {
        Self::try_new(index, ty).unwrap()
    }

    pub fn containing_addr(addr: A, ty: PageType) -> Self {
        Page {
            index: addr.as_usize() / ty.size(),
            ty,
            _phantom: PhantomData,
        }
    }

    pub const fn index(&self) -> usize {
        self.index
    }

    pub fn start(&self) -> A {
        A::try_new(self.index * self.ty.size()).unwrap()
    }

    pub fn end(&self) -> A {
        A::try_new((self.index + 1) * self.ty.size()).unwrap()
    }
}
