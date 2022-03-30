use crate::mapper::CodeMapper;
use crate::utils;
use crate::Node;

use crate::END_CODE;

pub struct MpTrie {
    pub(crate) mapper: CodeMapper,
    pub(crate) nodes: Vec<Node>,
    pub(crate) tails: Vec<u8>,
    pub(crate) code_size: u8,
    pub(crate) value_size: u8,
}

impl MpTrie {
    #[inline(always)]
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut node_idx = 0;
        let mut chars = key.as_ref().chars();

        while !self.is_leaf(node_idx) {
            if let Some(c) = chars.next() {
                if let Some(mc) = self.mapper.get(c as u32) {
                    if let Some(child_idx) = self.get_child_idx(node_idx, mc) {
                        node_idx = child_idx;
                    } else {
                        return None;
                    }
                }
            } else if self.has_leaf(node_idx) {
                return Some(self.get_value(self.get_leaf(node_idx)));
            } else {
                return None;
            }
        }

        let mut tail_pos = self.get_value(node_idx) as usize;
        let tail_len = self.tails[tail_pos] as usize;

        tail_pos += 1;
        for _ in 0..tail_len {
            if let Some(c) = chars.next() {
                if let Some(mc) = self.mapper.get(c as u32) {
                    if mc != utils::unpack_u32(&self.tails[tail_pos..], self.code_size) {
                        return None;
                    }
                }
            } else {
                return None;
            }
            tail_pos += self.code_size as usize;
        }

        if let Some(_) = chars.next() {
            None
        } else {
            Some(utils::unpack_u32(&self.tails[tail_pos..], self.value_size))
        }
    }

    pub fn common_prefix_searcher<'k, 't>(
        &'t self,
        text: &'k [Option<u32>],
    ) -> CommonPrefixSearcher<'k, 't> {
        CommonPrefixSearcher {
            text,
            text_pos: 0,
            trie: self,
            node_idx: 0,
        }
    }

    #[inline(always)]
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
        self.mapper.heap_bytes()
            + self.nodes.len() * std::mem::size_of::<Node>()
            + self.tails.len() * std::mem::size_of::<u8>()
    }

    pub fn num_elems(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_vacants(&self) -> usize {
        self.nodes.iter().filter(|nd| nd.is_vacant()).count()
    }

    pub fn vacant_ratio(&self) -> f64 {
        self.num_vacants() as f64 / self.num_elems() as f64
    }

    #[inline(always)]
    fn get_child_idx(&self, node_idx: u32, mc: u32) -> Option<u32> {
        if self.is_leaf(node_idx) {
            return None;
        }
        let child_idx = self.get_base(node_idx) ^ mc;
        if self.get_check(child_idx) == node_idx {
            return Some(child_idx);
        }
        None
    }

    #[inline(always)]
    fn get_base(&self, node_idx: u32) -> u32 {
        self.nodes[node_idx as usize].get_base()
    }

    #[inline(always)]
    fn get_check(&self, node_idx: u32) -> u32 {
        self.nodes[node_idx as usize].get_check()
    }

    #[inline(always)]
    fn is_leaf(&self, node_idx: u32) -> bool {
        self.nodes[node_idx as usize].is_leaf()
    }

    #[inline(always)]
    fn has_leaf(&self, node_idx: u32) -> bool {
        self.nodes[node_idx as usize].has_leaf()
    }

    #[inline(always)]
    fn get_leaf(&self, node_idx: u32) -> u32 {
        let leaf_idx = self.get_base(node_idx) ^ END_CODE;
        debug_assert_eq!(self.get_check(leaf_idx), node_idx);
        leaf_idx
    }

    #[inline(always)]
    fn get_value(&self, node_idx: u32) -> u32 {
        debug_assert!(self.is_leaf(node_idx));
        self.nodes[node_idx as usize].get_base()
    }
}

pub struct CommonPrefixSearcher<'k, 't> {
    text: &'k [Option<u32>],
    text_pos: usize,
    trie: &'t MpTrie,
    node_idx: u32,
}

impl Iterator for CommonPrefixSearcher<'_, '_> {
    type Item = (u32, usize);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        while self.text_pos < self.text.len() {
            if let Some(mc) = self.text[self.text_pos] {
                if let Some(child_idx) = self.trie.get_child_idx(self.node_idx, mc) {
                    self.node_idx = child_idx;
                } else {
                    self.text_pos = self.text.len();
                    return None;
                }
            } else {
                self.text_pos = self.text.len();
                return None;
            }

            self.text_pos += 1;
            if self.trie.is_leaf(self.node_idx) {
                let mut tail_pos = self.trie.get_value(self.node_idx) as usize;
                let tail_len = self.trie.tails[tail_pos] as usize;
                tail_pos += 1;
                for _ in 0..tail_len {
                    if self.text_pos == self.text.len() {
                        return None;
                    }
                    if let Some(mc) = self.text[self.text_pos] {
                        if mc
                            != utils::unpack_u32(&self.trie.tails[tail_pos..], self.trie.code_size)
                        {
                            self.text_pos = self.text.len();
                            return None;
                        }
                    } else {
                        self.text_pos = self.text.len();
                        return None;
                    }

                    tail_pos += self.trie.code_size as usize;
                    self.text_pos += 1;
                }
                let val = utils::unpack_u32(&self.trie.tails[tail_pos..], self.trie.value_size);
                let pos = self.text_pos;
                self.text_pos = self.text.len();
                return Some((val, pos));
            } else if self.trie.has_leaf(self.node_idx) {
                let leaf_idx = self.trie.get_leaf(self.node_idx);
                return Some((self.trie.get_value(leaf_idx), self.text_pos));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::Builder;

    #[test]
    fn test_exact_match() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new()
            .minimal_prefix()
            .from_keys(&keys)
            .release_mptrie();
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(trie.exact_match(&key), Some(i as u32));
        }
    }

    #[test]
    fn test_common_prefix_search() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new()
            .minimal_prefix()
            .from_keys(&keys)
            .release_mptrie();

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
