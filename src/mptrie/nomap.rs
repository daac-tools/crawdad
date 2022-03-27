use crate::Node;

pub struct MpTrie {
    pub(crate) nodes: Vec<Node>,
    pub(crate) suffixes: Vec<u32>,
    pub(crate) max_code: i32,
}

impl MpTrie {
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

        let suf_pos = self.get_value(node_idx) as usize;
        let suf_len = self.suffixes[suf_pos] as usize;

        for i in 1..=suf_len {
            if let Some(c) = chars.next() {
                if self.suffixes[suf_pos + i] != c as u32 {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(self.suffixes[suf_pos + suf_len + 1])
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
    trie: &'t MpTrie,
    node_idx: u32,
}

impl Iterator for CommonPrefixSearcher<'_, '_> {
    type Item = (u32, usize);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
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
                let mut matched_pos = self.text_pos;
                self.text_pos = self.text.len();
                let suf_pos = self.trie.get_value(self.node_idx) as usize;
                let suf_len = self.trie.suffixes[suf_pos] as usize;
                for i in 1..=suf_len {
                    if let Some(&mc) = self.text.get(matched_pos) {
                        if self.trie.suffixes[suf_pos + i] != mc {
                            return None;
                        }
                    } else {
                        return None;
                    }
                    matched_pos += 1;
                }
                return Some((self.trie.suffixes[suf_pos + suf_len + 1], matched_pos));
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
            .release_mptrie();
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
            .release_mptrie();
        assert_eq!(trie.exact_match("世界"), Some(0));
        assert_eq!(trie.exact_match("世界中"), Some(1));
        assert_eq!(trie.exact_match("世直し"), Some(2));
        assert_eq!(trie.exact_match("国民"), Some(3));
    }

    #[test]
    fn test_common_prefix_search_ja() {
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
