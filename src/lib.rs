//! 🦞 Crawdad: ChaRActer-Wise Double-Array Dictionary
//!
//! Crawdad is a library of natural language dictionaries using character-wise double-array tries.
//! The implementation is optimized for strings of multibyte-characters,
//! and you can enjoy fast text processing on such strings such as Japanese or Chinese.
#![deny(missing_docs)]
#![no_std]

#[cfg(target_pointer_width = "16")]
compile_error!("`target_pointer_width` must be larger than or equal to 32");

#[cfg(not(feature = "alloc"))]
compile_error!("`alloc` feature is currently required to build this crate");

#[macro_use]
extern crate alloc;

mod builder;
pub mod errors;
mod mapper;
pub mod mptrie;
pub mod trie;
mod utils;

use alloc::vec::Vec;

use core::ops::Range;

pub(crate) const OFFSET_MASK: u32 = 0x7fff_ffff;
pub(crate) const INVALID_IDX: u32 = 0xffff_ffff;
pub(crate) const MAX_VALUE: u32 = OFFSET_MASK;
pub(crate) const END_CODE: u32 = 0;

/// Special terminator, which must not be contained in keys.
pub const END_MARKER: char = '\u{0}';

pub use mptrie::MpTrie;
pub use trie::Trie;

/// Basic statistics of trie.
pub trait Statistics {
    /// Returns the total amount of heap used by this automaton in bytes.
    fn heap_bytes(&self) -> usize;

    /// Returns the total amount of bytes to serialize the data structure.
    fn io_bytes(&self) -> usize;

    /// Returns the number of reserved elements.
    fn num_elems(&self) -> usize;

    /// Returns the number of vacant elements.
    fn num_vacants(&self) -> usize;

    /// Returns the ratio of vacant elements.
    fn vacant_ratio(&self) -> f64 {
        self.num_vacants() as f64 / self.num_elems() as f64
    }
}

/// Trait to serialize/deserialize the data structure.
pub trait Serializer {
    /// Serializes the data structure into a [`Vec`].
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::{Trie, Serializer};
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = Trie::from_keys(&keys).unwrap();
    /// let bytes = trie.serialize_to_vec();
    /// ```
    fn serialize_to_vec(&self) -> Vec<u8>;

    /// Deserializes the data structure from a given byte slice.
    ///
    /// # Arguments
    ///
    /// * `source` - A source byte slice.
    ///
    /// # Returns
    ///
    /// A tuple of the data structure and the slice not used for the deserialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::{Trie, Serializer, Statistics};
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = Trie::from_keys(&keys).unwrap();
    ///
    /// let bytes = trie.serialize_to_vec();
    /// let (other, _) = Trie::deserialize_from_slice(&bytes);
    ///
    /// assert_eq!(trie.io_bytes(), other.io_bytes());
    /// ```
    fn deserialize_from_slice(source: &[u8]) -> (Self, &[u8])
    where
        Self: core::marker::Sized;
}

/// Result of common prefix search.
#[derive(Default, Clone)]
pub struct Match {
    value: u32,
    range_chars: Range<usize>,
    range_bytes: Range<usize>,
}

impl Match {
    /// Value associated with the matched key.
    #[inline(always)]
    pub const fn value(&self) -> u32 {
        self.value
    }

    /// Starting position of the match in characters.
    #[inline(always)]
    pub const fn start_chars(&self) -> usize {
        self.range_chars.start
    }

    /// Ending position of the match in characters.
    #[inline(always)]
    pub const fn end_chars(&self) -> usize {
        self.range_chars.end
    }

    /// Starting position of the match in bytes if characters are encoded in UTF-8.
    #[inline(always)]
    pub const fn start_bytes(&self) -> usize {
        self.range_bytes.start
    }

    /// Ending position of the match in bytes if characters are encoded in UTF-8.
    #[inline(always)]
    pub const fn end_bytes(&self) -> usize {
        self.range_bytes.end
    }
}

/// Handler for a mapped character.
#[derive(Default, Clone, Copy)]
struct MappedChar {
    c: Option<u32>,
    end_bytes: usize,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
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

    #[inline(always)]
    fn serialize(&self) -> [u8; 8] {
        let mut bytes = [0; 8];
        bytes[0..4].copy_from_slice(&self.base.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.check.to_le_bytes());
        bytes
    }

    #[inline(always)]
    fn deserialize(bytes: [u8; 8]) -> Self {
        Self {
            base: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            check: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        }
    }
}
