use crate::mapper::CodeMapper;
use crate::Node;

use crate::END_CODE;

pub struct Trie {
    pub(crate) nodes: Vec<Node>,
    pub(crate) mapper: CodeMapper,
}

impl Trie {
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut idx = 0;
        for c in key.as_ref().chars() {
            if let Some(mc) = self.mapper.get(c as u32) {
                if let Some(child_id) = self.get_child_id(idx, mc) {
                    idx = child_id;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        if self.nodes[idx as usize].is_leaf() {
            Some(self.nodes[idx as usize].get_base())
        } else if self.nodes[idx as usize].has_leaf() {
            let leaf_id = self.nodes[idx as usize].get_base() ^ END_CODE;
            debug_assert_eq!(self.nodes[leaf_id as usize].get_check(), idx);
            Some(self.nodes[leaf_id as usize].get_base())
        } else {
            None
        }
    }

    pub fn common_prefix_searcher<'k, 't>(
        &'t self,
        text: &'k [Option<u32>],
    ) -> CommonPrefixSearcher<'k, 't> {
        CommonPrefixSearcher {
            text,
            pos: 0,
            trie: self,
            idx: 0,
        }
    }

    pub fn map_text<K>(&self, text: K, mapped: &mut Vec<Option<u32>>)
    where
        K: AsRef<str>,
    {
        mapped.clear();
        for c in text.as_ref().chars() {
            mapped.push(self.mapper.get(c as u32));
        }
    }

    pub fn heap_bytes(&self) -> usize {
        self.mapper.heap_bytes() + self.nodes.len() * std::mem::size_of::<Node>()
    }

    #[inline(always)]
    fn get_child_id(&self, idx: u32, mc: u32) -> Option<u32> {
        if self.nodes[idx as usize].is_leaf() {
            return None;
        }
        let child_idx = self.nodes[idx as usize].get_base() ^ mc;
        if self.nodes[child_idx as usize].get_check() == idx {
            return Some(child_idx);
        }
        None
    }

    #[inline(always)]
    fn is_leaf(&self, idx: u32) -> bool {
        self.nodes[idx as usize].is_leaf()
    }

    #[inline(always)]
    fn has_leaf(&self, idx: u32) -> bool {
        self.nodes[idx as usize].has_leaf()
    }

    #[inline(always)]
    fn get_leaf(&self, idx: u32) -> u32 {
        let leaf_id = self.nodes[idx as usize].get_base() ^ END_CODE;
        debug_assert_eq!(self.nodes[leaf_id as usize].get_check(), idx);
        leaf_id
    }

    #[inline(always)]
    fn get_value(&self, idx: u32) -> u32 {
        debug_assert!(self.is_leaf(idx));
        self.nodes[idx as usize].get_base()
    }
}

pub struct CommonPrefixSearcher<'k, 't> {
    text: &'k [Option<u32>],
    pos: usize,
    trie: &'t Trie,
    idx: u32,
}

impl Iterator for CommonPrefixSearcher<'_, '_> {
    type Item = (u32, usize);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.text.len() {
            if let Some(mc) = self.text[self.pos] {
                if let Some(child_idx) = self.trie.get_child_id(self.idx, mc) {
                    self.idx = child_idx;
                } else {
                    self.pos = self.text.len();
                    return None;
                }
            } else {
                self.pos = self.text.len();
                return None;
            }
            self.pos += 1;
            if self.trie.is_leaf(self.idx) {
                let matched_pos = self.pos;
                self.pos = self.text.len();
                return Some((self.trie.get_value(self.idx), matched_pos));
            } else if self.trie.has_leaf(self.idx) {
                let leaf_idx = self.trie.get_leaf(self.idx);
                return Some((self.trie.get_value(leaf_idx), self.pos));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::freqmap::Builder;

    #[test]
    fn test_exact_match() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new().from_keys(&keys).release_trie();
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(trie.exact_match(&key), Some(i as u32));
        }
    }

    #[test]
    fn test_common_prefix_search() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new().from_keys(&keys).release_trie();

        let mut mapped = vec![];
        trie.map_text("国民が世界中で世直し", &mut mapped);

        let mut results = vec![];
        for i in 0..mapped.len() {
            for (val, pos) in trie.common_prefix_searcher(&mapped[i..]) {
                results.push((val, i + pos));
            }
        }
        assert_eq!(results, vec![(3, 2), (0, 5), (1, 6), (2, 10)]);
    }
}
