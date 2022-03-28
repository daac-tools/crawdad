use crate::hasher::RollingHasher;
use crate::Node;

pub struct RhTrie {
    pub(crate) nodes: Vec<Node>,
    pub(crate) tails: Vec<u32>,
    pub(crate) max_code: i32,
}

impl RhTrie {
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut node_idx = 0;
        let mut chars = key.as_ref().chars();

        while !self.is_leaf(node_idx) {
            if let Some(c) = chars.next() {
                if let Some(child_idx) = self.get_child_idx(node_idx, c as u32) {
                    node_idx = child_idx;
                } else {
                    return None;
                }
            } else if self.has_leaf(node_idx) {
                return Some(self.get_value(self.get_leaf(node_idx)));
            } else {
                return None;
            }
        }

        let suffix: Vec<_> = chars.map(|c| c as u32).collect();
        let mut tail_pos = self.get_value(node_idx) as usize;
        let tail_num = self.tails[tail_pos] as usize;

        tail_pos += 1;
        for _ in 0..tail_num {
            let tail_len = self.tails[tail_pos] as usize;
            if tail_len > suffix.len() {
                return None;
            }
            if tail_len == suffix.len() {
                if self.tails[tail_pos + 1] == RollingHasher::hash(&suffix) {
                    return Some(self.tails[tail_pos + 2]);
                }
            }
            tail_pos += 3;
        }

        return None;
    }

    pub fn common_prefix_searcher<'k, 't>(
        &'t self,
        text: &'k [u32],
    ) -> CommonPrefixSearcher<'k, 't> {
        CommonPrefixSearcher {
            text,
            text_pos: 0,
            trie: self,
            node_idx: 0,
            tail_num: 0,
            tail_len: 0,
            tail_pos: 0,
        }
    }

    pub fn map_text<K>(&self, text: K, mapped: &mut Vec<u32>)
    where
        K: AsRef<str>,
    {
        mapped.clear();
        for c in text.as_ref().chars() {
            mapped.push(c as u32);
        }
    }

    pub fn heap_bytes(&self) -> usize {
        self.nodes.len() * std::mem::size_of::<Node>()
            + self.tails.len() * std::mem::size_of::<u32>()
    }

    #[inline(always)]
    fn get_child_idx(&self, node_idx: u32, c: u32) -> Option<u32> {
        if self.is_leaf(node_idx) {
            return None;
        }
        let child_idx = (self.get_base(node_idx) + c as i32) as u32;
        if let Some(check) = self.get_check(child_idx) {
            if check == node_idx {
                return Some(child_idx);
            }
        }
        None
    }

    #[inline(always)]
    fn get_base(&self, node_idx: u32) -> i32 {
        self.nodes[node_idx as usize].get_base() as i32 - self.max_code
    }

    #[inline(always)]
    fn get_check(&self, node_idx: u32) -> Option<u32> {
        if let Some(node) = self.nodes.get(node_idx as usize) {
            Some(node.get_check())
        } else {
            None
        }
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
        let leaf_idx = self.get_base(node_idx) as u32;
        debug_assert_eq!(self.get_check(leaf_idx), Some(node_idx));
        leaf_idx
    }

    #[inline(always)]
    fn get_value(&self, node_idx: u32) -> u32 {
        debug_assert!(self.is_leaf(node_idx));
        self.nodes[node_idx as usize].get_base()
    }
}

pub struct CommonPrefixSearcher<'k, 't> {
    text: &'k [u32],
    text_pos: usize,
    trie: &'t RhTrie,
    node_idx: u32,
    tail_num: usize,
    tail_len: usize,
    tail_pos: usize,
}

impl CommonPrefixSearcher<'_, '_> {
    fn next_suffix(&mut self) -> Option<(u32, usize)> {
        debug_assert_ne!(self.tail_num, 0);

        let tails = &self.trie.tails;
        while self.text_pos < self.text.len() && self.tail_num != 0 {
            let tail_len = tails[self.tail_pos] as usize;
            let hash_val = tails[self.tail_pos + 1];

            self.tail_num -= 1;
            self.tail_len = tail_len;
            self.tail_pos += 3;

            let text_epos = self.text_pos + self.tail_len;
            if let Some(suffix) = self.text.get(self.text_pos..text_epos) {
                let h = RollingHasher::hash(suffix);
                if h == hash_val {
                    return Some((tails[self.tail_pos - 1], self.text_pos + self.tail_len));
                }
            } else {
                self.tail_num = 0;
                self.text_pos = self.text.len();
                return None;
            }
        }

        if self.tail_num == 0 {
            self.text_pos = self.text.len();
        }
        None
    }
}

impl Iterator for CommonPrefixSearcher<'_, '_> {
    type Item = (u32, usize);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.tail_num != 0 {
            return self.next_suffix();
        }
        while self.text_pos < self.text.len() {
            if let Some(child_idx) = self
                .trie
                .get_child_idx(self.node_idx, self.text[self.text_pos])
            {
                self.node_idx = child_idx;
            } else {
                self.text_pos = self.text.len();
                return None;
            }
            self.text_pos += 1;
            if self.trie.is_leaf(self.node_idx) {
                let tail_pos = self.trie.get_value(self.node_idx) as usize;
                self.tail_num = self.trie.tails[tail_pos] as usize;
                self.tail_len = 0;
                self.tail_pos = tail_pos + 1;
                return self.next_suffix();
            } else if self.trie.has_leaf(self.node_idx) {
                return Some((
                    self.trie.get_value(self.trie.get_leaf(self.node_idx)),
                    self.text_pos,
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::nomap::Builder;

    #[test]
    fn test_exact_match_en() {
        let keys = vec!["ab", "abc", "adaab", "bbc"];
        let trie = Builder::new()
            .set_suffix_thr(1)
            .from_keys(&keys)
            .release_rhtrie();
        assert_eq!(trie.exact_match("ab"), Some(0));
        assert_eq!(trie.exact_match("abc"), Some(1));
        assert_eq!(trie.exact_match("adaab"), Some(2));
        assert_eq!(trie.exact_match("bbc"), Some(3));
    }

    #[test]
    fn test_exact_match_ja() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new()
            .set_suffix_thr(1)
            .from_keys(&keys)
            .release_rhtrie();
        assert_eq!(trie.exact_match("世界"), Some(0));
        assert_eq!(trie.exact_match("世界中"), Some(1));
        assert_eq!(trie.exact_match("世直し"), Some(2));
        assert_eq!(trie.exact_match("国民"), Some(3));
    }

    #[test]
    fn test_common_prefix_search_en_1() {
        let keys = vec!["ab", "abc", "adaab", "bbc"];
        let trie = Builder::new()
            .set_suffix_thr(1)
            .from_keys(&keys)
            .release_rhtrie();

        let mut mapped = vec![];
        trie.map_text("adaabcabbc", &mut mapped);

        let mut results = vec![];
        for i in 0..mapped.len() {
            for (val, pos) in trie.common_prefix_searcher(&mapped[i..]) {
                results.push((val, i + pos));
            }
        }
        assert_eq!(results, vec![(2, 5), (0, 5), (1, 6), (0, 8), (3, 10)]);
    }

    #[test]
    fn test_common_prefix_search_en_2() {
        let keys = vec!["ab", "abc", "adaab", "bbc"];
        let trie = Builder::new()
            .set_suffix_thr(2)
            .from_keys(&keys)
            .release_rhtrie();

        let mut mapped = vec![];
        trie.map_text("adaabcabbc", &mut mapped);

        dbg!(&trie.tails);

        let mut results = vec![];
        for i in 0..mapped.len() {
            for (val, pos) in trie.common_prefix_searcher(&mapped[i..]) {
                dbg!((val, pos));
                results.push((val, i + pos));
            }
        }
        assert_eq!(results, vec![(2, 5), (0, 5), (1, 6), (0, 8), (3, 10)]);
    }

    #[test]
    fn test_common_prefix_search_ja_1() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new()
            .set_suffix_thr(1)
            .from_keys(&keys)
            .release_rhtrie();

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

    #[test]
    fn test_common_prefix_search_ja_2() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new()
            .set_suffix_thr(2)
            .from_keys(&keys)
            .release_rhtrie();

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
