pub mod freqmap;
pub mod nomap;

use crate::bytes;

#[derive(Clone, Copy)]
pub struct TailIter<'a> {
    data: &'a [u8],
    pos: usize,
    num: u8,
    hash_size: u8,
    value_size: u8,
}

impl<'a> TailIter<'a> {
    pub fn new(data: &'a [u8], hash_size: u8, value_size: u8) -> Self {
        Self {
            data,
            pos: 0,
            num: 0,
            hash_size,
            value_size,
        }
    }

    pub fn clear(mut self) -> Self {
        self.num = 0;
        self.pos = 0;
        self
    }

    pub fn set(mut self, pos: usize) -> Self {
        assert_ne!(self.data[pos], 0);
        self.num = self.data[pos];
        self.pos = pos + 1;
        self
    }

    pub fn is_valid(&self) -> bool {
        self.num != 0
    }
}

impl Iterator for TailIter<'_> {
    type Item = (usize, u32, u32); // Len, Hash, Val

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.num == 0 {
            return None;
        }

        let hash_pos = self.pos + 1;
        let value_pos = hash_pos + self.hash_size as usize;
        let next_pos = value_pos + self.value_size as usize;

        let len = self.data[self.pos] as usize;
        let hash = bytes::unpack_u32(&self.data[hash_pos..], self.hash_size);
        let value = bytes::unpack_u32(&self.data[value_pos..], self.value_size);

        self.pos = next_pos;
        self.num -= 1;

        Some((len, hash, value))
    }
}
