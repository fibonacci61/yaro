pub const ENTRY_COUNT: usize = 512;

#[repr(C, align(4096))]
pub struct RawTable(pub [usize; ENTRY_COUNT]);

// TODO: Add level-aware abstraction over `RawTable`
