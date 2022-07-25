use alloc::vec::Vec;

use core::mem::size_of;

pub const INVALID_CODE: u16 = u16::MAX;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CodeMapper {
    table: Vec<u16>,
    alphabet_size: u32,
}

impl CodeMapper {
    pub fn new(freqs: &[u32]) -> Self {
        let sorted = {
            let mut sorted = vec![];
            for (c, &f) in freqs.iter().enumerate().filter(|(_, &f)| f != 0) {
                sorted.push((c, f));
            }
            sorted.sort_unstable_by(|(c1, f1), (c2, f2)| f2.cmp(f1).then_with(|| c1.cmp(c2)));
            sorted
        };

        let mut table = vec![INVALID_CODE; freqs.len()];
        for (i, &(c, _)) in sorted.iter().enumerate() {
            table[c] = i.try_into().unwrap();
        }
        Self {
            table,
            alphabet_size: sorted.len().try_into().unwrap(),
        }
    }

    #[inline]
    pub const fn alphabet_size(&self) -> u32 {
        self.alphabet_size
    }

    #[inline(always)]
    pub fn get(&self, c: char) -> Option<u32> {
        self.table
            .get(usize::try_from(u32::from(c)).unwrap())
            .copied()
            .filter(|&code| code != INVALID_CODE)
            .map(u32::from)
    }

    #[inline]
    pub fn heap_bytes(&self) -> usize {
        self.table.len() * size_of::<u16>()
    }

    #[inline]
    pub fn io_bytes(&self) -> usize {
        self.table.len() * size_of::<u16>() + size_of::<u32>() * 2
    }

    pub fn serialize_into_vec(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(&u32::try_from(self.table.len()).unwrap().to_le_bytes());
        for x in &self.table {
            dest.extend_from_slice(&x.to_le_bytes());
        }
        dest.extend_from_slice(&self.alphabet_size.to_le_bytes());
    }

    pub fn deserialize_from_slice(mut source: &[u8]) -> (Self, &[u8]) {
        let table = {
            let len = u32::from_le_bytes(source[..4].try_into().unwrap()) as usize;
            source = &source[4..];
            let mut table = Vec::with_capacity(len);
            for _ in 0..len {
                table.push(u16::from_le_bytes(source[..2].try_into().unwrap()));
                source = &source[2..];
            }
            table
        };
        let alphabet_size = u32::from_le_bytes(source[..4].try_into().unwrap());
        source = &source[4..];
        (
            Self {
                table,
                alphabet_size,
            },
            source,
        )
    }
}
