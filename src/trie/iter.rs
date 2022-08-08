//! Iterator of records in Trie.
#![cfg(feature = "record-iter")]

use alloc::vec::Vec;

use super::Trie;
use crate::utils::FromU32;
use crate::END_CODE;

/// Iterator of records stored in [`Trie`].
pub struct RecordIter<'t> {
    trie: &'t Trie,
    node_idx: u32,
}

impl Iterator for RecordIter<'_> {
    type Item = (Vec<char>, u32);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        while usize::from_u32(self.node_idx) < self.trie.num_elems() {
            if let Some((k, v)) = self.trie.extract_record(self.node_idx) {
                self.node_idx += 1;
                return Some((k, v));
            }
            self.node_idx += 1;
        }
        None
    }
}

impl Trie {
    /// Creates an iteratoer of records.
    #[inline(always)]
    pub fn record_iter(&self) -> RecordIter {
        RecordIter {
            trie: self,
            node_idx: 0,
        }
    }

    #[inline(always)]
    fn extract_record(&self, mut node_idx: u32) -> Option<(Vec<char>, u32)> {
        if !self.is_leaf(node_idx) {
            return None;
        }

        let value = self.get_value(node_idx);
        let mut key = vec![];

        while node_idx != 0 {
            let parent_idx = self.get_check(node_idx);
            let code = self.get_base(parent_idx) ^ node_idx;
            if code != END_CODE {
                key.push(self.mapper.get_inv(code));
            }
            node_idx = parent_idx;
        }
        key.reverse();
        Some((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_iter() {
        let keys = vec!["世界", "世界中", "世論調査", "統計調査"];
        let trie = Trie::from_keys(&keys).unwrap();
        let mut records: Vec<_> = trie.record_iter().collect();
        records.sort();

        assert_eq!(
            records,
            vec![
                (vec!['世', '界'], 0),
                (vec!['世', '界', '中'], 1),
                (vec!['世', '論', '調', '査'], 2),
                (vec!['統', '計', '調', '査'], 3)
            ]
        );
    }
}
