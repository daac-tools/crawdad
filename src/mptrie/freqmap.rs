use crate::mapper::CodeMapper;
use crate::Node;

use crate::END_CODE;

pub struct MpTrie {
    pub(crate) nodes: Vec<Node>,
    pub(crate) suffixes: Vec<u32>,
    pub(crate) mapper: CodeMapper,
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
                if let Some(mc) = self.mapper.get(c as u32) {
                    if let Some(child_idx) = self.get_child_idx(node_idx, mc) {
                        node_idx = child_idx;
                    } else {
                        return None;
                    }
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
                if let Some(mc) = self.mapper.get(c as u32) {
                    if self.suffixes[suf_pos + i] != mc as u32 {
                        return None;
                    }
                } else {
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
        text: &'k [Option<u32>],
    ) -> CommonPrefixSearcher<'k, 't> {
        CommonPrefixSearcher {
            text,
            text_pos: 0,
            trie: self,
            node_idx: 0,
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
                let mut matched_pos = self.text_pos;
                self.text_pos = self.text.len();
                let suf_pos = self.trie.get_value(self.node_idx) as usize;
                let suf_len = self.trie.suffixes[suf_pos] as usize;
                for i in 1..=suf_len {
                    if let Some(&mc) = self.text.get(matched_pos) {
                        if let Some(mc) = mc {
                            if self.trie.suffixes[suf_pos + i] != mc {
                                return None;
                            }
                        } else {
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
