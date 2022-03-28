use crate::Node;

pub struct Trie {
    pub(crate) nodes: Vec<Node>,
    pub(crate) max_code: i32,
}

impl Trie {
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut node_idx = 0;
        for c in key.as_ref().chars() {
            if let Some(child_idx) = self.get_child_idx(node_idx, c as u32) {
                node_idx = child_idx;
            } else {
                return None;
            }
        }
        if self.is_leaf(node_idx) {
            Some(self.get_value(node_idx))
        } else if self.has_leaf(node_idx) {
            Some(self.get_value(self.get_leaf(node_idx)))
        } else {
            None
        }
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
    trie: &'t Trie,
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
                let match_pos = self.text_pos;
                self.text_pos = self.text.len();
                return Some((self.trie.get_value(self.node_idx), match_pos));
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
    use crate::builder::nomap::Builder;

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
