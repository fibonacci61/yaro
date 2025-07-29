use core::marker::PhantomData;

use super::entry::Entry;
use crate::mem::paging::{PageType, PhysPage};

pub const ENTRY_COUNT: usize = 512;

#[repr(C, align(4096))]
pub struct RawTable(pub [Entry; ENTRY_COUNT]);

pub trait Level {
    const PAGE_TYPE: PageType;
}

pub trait Superlevel: Level {
    type Sublevel: Level;
}

pub struct Level2;
pub struct Level1;
pub struct Level0;

impl Level for Level2 {
    const PAGE_TYPE: PageType = PageType::Giga;
}

impl Superlevel for Level2 {
    type Sublevel = Level1;
}

impl Level for Level1 {
    const PAGE_TYPE: PageType = PageType::Mega;
}

impl Superlevel for Level1 {
    type Sublevel = Level0;
}

impl Level for Level0 {
    const PAGE_TYPE: PageType = PageType::Base;
}

pub struct Table<L: Level> {
    inner: RawTable,
    _phantom: PhantomData<L>,
}

pub type P2Table = Table<Level2>;
pub type P1Table = Table<Level1>;
pub type P0Table = Table<Level0>;

pub enum TableEntry<'a, L: Superlevel> {
    Page(PhysPage),
    Table(&'a Table<L::Sublevel>),
}

pub enum TableEntryMut<'a, L: Superlevel> {
    Page(PhysPage),
    Table(&'a mut Table<L::Sublevel>),
}

impl<L: Superlevel> Table<L> {
    pub fn next(&self, index: usize) -> Option<TableEntry<L>> {
        self.get(index).map(|entry| {
            let ppn = entry.ppn();
            if entry.is_leaf() {
                TableEntry::Page(PhysPage::new(ppn, L::PAGE_TYPE))
            } else {
                // TODO: get virtual address, cast to table pointer and dereference
                todo!()
            }
        })
    }

    pub fn next_mut(&self, index: usize) -> Option<TableEntryMut<L>> {
        self.get(index).map(|entry| {
            let ppn = entry.ppn();
            if entry.is_leaf() {
                TableEntryMut::Page(PhysPage::new(ppn, L::PAGE_TYPE))
            } else {
                // TODO: get virtual address, cast to table pointer and dereference
                todo!()
            }
        })
    }
}

impl<L: Level> Table<L> {
    fn get(&self, index: usize) -> Option<&Entry> {
        self.inner.0.get(index).filter(|entry| entry.valid())
    }

    pub fn get_page(&self, index: usize) -> Option<PhysPage> {
        self.get(index)
            .filter(|entry| entry.is_leaf())
            .map(|entry| PhysPage::new(entry.ppn(), L::PAGE_TYPE))
    }
}
