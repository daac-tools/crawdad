pub const INVALID_CODE: u32 = u32::MAX;

#[derive(Default, Clone)]
pub struct CodeMapper {
    table: Vec<u32>,
    alphabet_size: u32,
}

impl CodeMapper {
    pub fn new(freqs: &[u32]) -> Self {
        let sorted = {
            let mut sorted = vec![];
            for (c, &f) in freqs.iter().enumerate().filter(|(_, &f)| f != 0) {
                sorted.push((c, f));
            }
            sorted.sort_unstable_by(|(_, f1), (_, f2)| f2.cmp(f1));
            sorted
        };
        let mut table = vec![INVALID_CODE; freqs.len()];
        for (i, &(c, _)) in sorted.iter().enumerate() {
            table[c] = i as u32;
        }
        Self {
            table,
            alphabet_size: sorted.len() as u32,
        }
    }

    #[inline]
    pub const fn alphabet_size(&self) -> u32 {
        self.alphabet_size
    }

    #[inline(always)]
    pub fn get(&self, c: char) -> Option<u32> {
        self.table
            .get(c as usize)
            .copied()
            .filter(|&code| code != INVALID_CODE)
    }

    #[inline]
    pub fn heap_bytes(&self) -> usize {
        self.table.len() * std::mem::size_of::<u32>()
    }
}
