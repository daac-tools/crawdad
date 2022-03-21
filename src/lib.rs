pub mod builder;
mod mapper;
pub mod trie;

pub const OFFSET_MASK: u32 = 0x7fff_ffff;
pub const INVALID_IDX: u32 = 0xffff_ffff;
pub const END_MARKER: u32 = 0;
pub const END_CODE: u32 = 0;
