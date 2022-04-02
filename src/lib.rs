//! 🦞 Crawdad: ChaRActer-Wise Double-Array Dictionary
//!
//! Crawdad is a library of natural language dictionaries using character-wise double-array tries.
//! The implementation is optimized for strings of multibyte-characters,
//! and you can enjoy fast text processing on such strings such as Japanese or Chinese.
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(target_pointer_width = "16")]
compile_error!("`target_pointer_width` must be larger than or equal to 32");

mod builder;
pub mod errors;
pub mod fmptrie;
mod mapper;
pub mod mptrie;
pub mod trie;
mod utils;

pub(crate) const OFFSET_MASK: u32 = 0x7fff_ffff;
pub(crate) const INVALID_IDX: u32 = 0xffff_ffff;
pub(crate) const MAX_VALUE: u32 = OFFSET_MASK;
pub(crate) const END_CODE: u32 = 0;

/// Special terminator, which must not be contained in keys.
pub const END_MARKER: char = '\u{0}';

pub use fmptrie::FmpTrie;
pub use mptrie::MpTrie;
pub use trie::Trie;

/// Basic statistics of trie.
pub trait Statistics {
    /// Returns the total amount of heap used by this automaton in bytes.
    fn heap_bytes(&self) -> usize;

    /// Returns the number of reserved elements.
    fn num_elems(&self) -> usize;

    /// Returns the number of vacant elements.
    fn num_vacants(&self) -> usize;

    /// Returns the ratio of vacant elements.
    fn vacant_ratio(&self) -> f64 {
        self.num_vacants() as f64 / self.num_elems() as f64
    }
}

/// Result of common prefix search.
#[derive(Default, Clone, Copy)]
pub struct Match {
    value: u32,
    end_in_chars: usize,
    end_in_bytes: usize,
}

impl Match {
    /// Value associated with the matched key.
    #[inline(always)]
    pub const fn value(&self) -> u32 {
        self.value
    }

    /// Ending position of the match in characters.
    #[inline(always)]
    pub const fn end_in_chars(&self) -> usize {
        self.end_in_chars
    }

    /// Ending position of the match in bytes if characters are encoded in UTF-8.
    #[inline(always)]
    pub const fn end_in_bytes(&self) -> usize {
        self.end_in_bytes
    }
}

/// Handler for a mapped character.
#[derive(Default, Clone, Copy)]
pub struct MappedChar {
    c: Option<u32>,
    len_utf8: usize,
}

impl MappedChar {
    /// Returns the number of bytes the original character needs if encoded in UTF-8.
    pub const fn len_utf8(&self) -> usize {
        self.len_utf8
    }
}

#[derive(Default, Clone, Copy)]
struct Node {
    base: u32,
    check: u32,
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

    #[inline(always)]
    pub const fn is_vacant(&self) -> bool {
        self.base == OFFSET_MASK && self.check == OFFSET_MASK
    }
}
