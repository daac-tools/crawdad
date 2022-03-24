pub mod freqmap;
pub mod nomap;

use crate::OFFSET_MASK;

#[derive(Default, Clone)]
pub struct Node {
    pub(crate) base: u32,
    pub(crate) check: u32,
}

impl Node {
    #[inline(always)]
    pub const fn get_base(&self) -> u32 {
        self.base & OFFSET_MASK
    }

    #[inline(always)]
    pub const fn get_check(&self) -> u32 {
        self.check & OFFSET_MASK
    }

    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        self.base & !OFFSET_MASK != 0
    }

    #[inline(always)]
    pub const fn has_leaf(&self) -> bool {
        self.check & !OFFSET_MASK != 0
    }
}
