const BASE: u32 = 7;

pub struct RollingHasher {
    vals: Vec<u32>,
    tmp: u32,
}

impl RollingHasher {
    pub fn new(len: usize) -> Self {
        Self {
            vals: Vec::with_capacity(len),
            tmp: 1,
        }
    }

    pub fn add(&mut self, x: u32) {
        self.vals.push(x.wrapping_mul(self.tmp));
        self.tmp *= BASE;
    }

    pub fn last(&self) -> u32 {
        *self.vals.last().unwrap_or(&0)
    }

    pub fn hash(seq: &[u32]) -> u32 {
        let mut h = Self::new(seq.len());
        seq.iter().for_each(|&x| h.add(x));
        h.last()
    }

    pub fn hash_with_option(seq: &[Option<u32>]) -> Option<u32> {
        let mut h = Self::new(seq.len());
        for &x in seq {
            if let Some(x) = x {
                h.add(x);
            } else {
                return None;
            }
        }
        Some(h.last())
    }
}
