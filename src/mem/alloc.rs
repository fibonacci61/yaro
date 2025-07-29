use core::{num::NonZeroUsize, ptr::NonNull};

use spin::Mutex;
use talc::{ErrOnOom, Talc, Talck};

use super::{PAGE_SIZE, addr::VirtAddr};
use crate::mem::addr::PhysAddr;

pub static PAGE_ALLOCATOR: Mutex<BiBuddy> = Mutex::new(BiBuddy::new());

#[global_allocator]
pub static HEAP_ALLOCATOR: Talck<Mutex<()>, ErrOnOom> = Talck::new(Talc::new(ErrOnOom));

pub const HIGHEST_ORDER: usize = 12;
pub const ORDER_COUNT: usize = HIGHEST_ORDER + 1;

#[repr(align(4096))]
pub struct BlockHeader {
    next: Option<Block>,
}

#[derive(Debug)]
pub struct Block {
    header: NonNull<BlockHeader>,
    order: usize,
}

pub struct FreeList {
    head: Option<Block>,
}

pub struct BiBuddy {
    free_lists: [FreeList; ORDER_COUNT],
}

// Used to verify that a block did indeed come from the allocator.
#[derive(Debug)]
pub struct Allocation(Block);

unsafe impl Send for Block {}
unsafe impl Sync for Block {}

fn order_size(order: usize) -> usize {
    PAGE_SIZE * (1 << order)
}

impl Block {
    pub unsafe fn new(addr: PhysAddr, order: usize) -> Option<Self> {
        assert!(addr.as_usize() != 0, "block start cannot be null");
        if addr.as_usize() % order_size(order) != 0 {
            return None;
        }

        unsafe { Self::from_virt(VirtAddr::from_phys(addr), order) }
    }

    unsafe fn from_virt(virt: VirtAddr, order: usize) -> Option<Self> {
        let header = virt
            .as_non_null()
            .expect("block maps to null virtual address");
        unsafe {
            header.write(BlockHeader { next: None });
        }
        Some(Self { header, order })
    }

    pub unsafe fn from_range(mut start: PhysAddr, end: PhysAddr) -> Option<Self> {
        assert!(start.as_usize() != 0, "block start cannot be null");
        start = PhysAddr(start.as_usize().next_multiple_of(PAGE_SIZE));
        if start >= end {
            return None;
        }

        let size_pages = (end - start) / PAGE_SIZE;
        let order = size_pages.ilog2() as usize;
        Some(unsafe { Self::new(start, order) }.unwrap())
    }

    pub fn addr(&self) -> NonZeroUsize {
        self.header.addr()
    }

    pub const fn order(&self) -> usize {
        self.order
    }

    fn header_mut(&mut self) -> &mut BlockHeader {
        unsafe { self.header.as_mut() }
    }

    pub fn buddy_addr(&self) -> usize {
        self.addr().get() ^ order_size(self.order)
    }

    pub fn merge(mut a: Block, mut b: Block) -> Self {
        assert!(a.order == b.order);
        assert!(a.buddy_addr() == b.addr().get());

        if a.addr() < b.addr() {
            a.order += 1;
            a
        } else {
            b.order += 1;
            b
        }
    }

    pub fn split(&mut self) -> Self {
        assert!(self.order > 0);
        self.order -= 1;
        let new_size = PAGE_SIZE * (1 << self.order);
        let right_block_addr = self.addr().get() + new_size;
        unsafe { Self::from_virt(VirtAddr::new(right_block_addr), self.order) }.unwrap()
    }
}

impl FreeList {
    pub const fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&mut self, block: Block) {
        let old_head = self.head.replace(block);
        self.head.as_mut().unwrap().header_mut().next = old_head;
    }

    pub fn pop(&mut self) -> Option<Block> {
        let mut head = self.head.take();
        if let Some(ref mut head) = head {
            self.head = head.header_mut().next.take();
        }
        head
    }

    pub fn peek(&self) -> Option<&Block> {
        self.head.as_ref()
    }

    pub fn peek_mut(&mut self) -> Option<&mut Block> {
        self.head.as_mut()
    }

    pub fn remove_if(&mut self, mut predicate: impl FnMut(&Block) -> bool) -> Option<Block> {
        let head = self.peek_mut()?;
        if predicate(head) {
            return Some(self.pop().unwrap());
        }

        let mut prev = head.header;
        let mut current = head;
        loop {
            current = current.header_mut().next.as_mut()?;

            if predicate(current) {
                return unsafe {
                    let ret = (*prev.as_ptr()).next.take().unwrap();
                    (*prev.as_ptr()).next = current.header_mut().next.take();
                    Some(ret)
                };
            }

            prev = current.header;
        }
    }
}

impl BiBuddy {
    pub const fn new() -> Self {
        Self {
            free_lists: [const { FreeList::new() }; ORDER_COUNT],
        }
    }

    pub unsafe fn claim(&mut self, mut block: Block) {
        if block.order > HIGHEST_ORDER {
            unsafe { self.claim(block.split()) };
        }
        self.free_lists[block.order].push(block);
    }

    pub fn alloc(&mut self, order: usize) -> Option<Allocation> {
        assert!(order < ORDER_COUNT, "order too high");
        let mut next_order = order;
        loop {
            if next_order > HIGHEST_ORDER {
                return None;
            }

            if self.free_lists[next_order].peek().is_some() {
                break;
            }

            next_order += 1;
        }

        let mut block = self.free_lists[next_order].pop().unwrap();
        while next_order > order {
            next_order -= 1;
            let right_half = block.split();
            self.free_lists[next_order].push(right_half);
        }

        block.header_mut().next = None;
        Some(Allocation(block))
    }

    pub fn free(&mut self, allocation: Allocation) {
        self.free_inner(allocation.0);
    }

    // Unfortunately, this does a linear search of free lists to find buddies.
    // It's also recursive, but I believe rustc should be able to optimize it.
    fn free_inner(&mut self, block: Block) {
        let buddy_addr = block.buddy_addr();
        if block.order() <= HIGHEST_ORDER
            && let Some(buddy) = self.free_lists[block.order()]
                .remove_if(|other_block| other_block.addr().get() == buddy_addr)
        {
            let merged = Block::merge(block, buddy);
            self.free_inner(merged);
        } else {
            self.free_lists[block.order()].push(block);
        }
    }
}

impl Allocation {
    pub fn start(&self) -> VirtAddr {
        VirtAddr::from_phys(PhysAddr(self.0.header.addr().get()))
    }

    pub const fn order(&self) -> usize {
        self.0.order()
    }

    pub const fn size(&self) -> usize {
        PAGE_SIZE * (1 << self.0.order)
    }

    pub fn end(&self) -> VirtAddr {
        VirtAddr::from_phys(PhysAddr(self.0.header.addr().get() + self.size()))
    }
}
