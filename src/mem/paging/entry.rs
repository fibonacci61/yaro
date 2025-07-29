pub const FLAG_BITS: usize = 0b1111111111;
pub const PPN_BITS: usize = !FLAG_BITS;

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct EntryFlags: usize {
        const VALID     = 0b0000000001;
        const READ      = 0b0000000010;
        const WRITE     = 0b0000000100;
        const EXECUTE   = 0b0000001000;
        const USER      = 0b0000010000;
        const GLOBAL    = 0b0000100000;
        const ACCESSED  = 0b0001000000;
        const DIRTY     = 0b0010000000;
        const SOFTWARE0 = 0b0100000000;
        const SOFTWARE1 = 0b1000000000;
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Entry(usize);

impl Entry {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_retain(self.0 & FLAG_BITS)
    }

    pub const fn ppn(&self) -> usize {
        self.0 & PPN_BITS >> 10
    }

    pub const fn valid(&self) -> bool {
        self.flags().contains(EntryFlags::VALID)
    }

    pub const fn is_leaf(&self) -> bool {
        const LEAF_FLAGS: EntryFlags = EntryFlags::READ
            .union(EntryFlags::WRITE)
            .union(EntryFlags::EXECUTE);
        self.flags().intersects(LEAF_FLAGS)
    }

    pub const fn with_flags(self, flags: EntryFlags) -> Self {
        Self(self.0 & !FLAG_BITS | flags.bits())
    }

    pub const fn with_ppn(self, ppn: usize) -> Self {
        Self(self.0 & !PPN_BITS | (ppn << 10 & PPN_BITS))
    }
}
