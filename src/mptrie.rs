//! A minimal-prefix trie form that is memory-efficient for long strings.
use crate::builder::Builder;
use crate::errors::Result;
use crate::mapper::CodeMapper;
use crate::{utils, Match, Node, Statistics};

use crate::END_CODE;

/// A minimal-prefix trie form that is memory-efficient for long strings.
pub struct MpTrie {
    pub(crate) mapper: CodeMapper,
    pub(crate) nodes: Vec<Node>,
    pub(crate) tails: Vec<u8>,
    pub(crate) code_size: u8,
    pub(crate) value_size: u8,
}

impl MpTrie {
    /// Creates a new [`MpTrie`] from input keys.
    ///
    /// # Arguments
    ///
    /// - `keys`: List of string keys.
    ///
    /// # Errors
    ///
    /// [`CrawdadError`](crate::errors::CrawdadError) will be returned when
    ///
    /// - `keys` is empty,
    /// - `keys` contains empty strings,
    /// - `keys` contains duplicate keys,
    /// - `keys` is not sorted,
    /// - the scale of `keys` exceeds the expected one, or
    /// - the scale of the resulting trie exceeds the expected one.
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::{MpTrie, Statistics};
    ///
    /// let keys = vec!["世界", "世界中", "世直し", "国民"];
    /// let trie = MpTrie::from_keys(keys).unwrap();
    ///
    /// assert_eq!(trie.num_elems(), 16);
    /// assert_eq!(trie.num_vacants(), 9);
    /// ```
    pub fn from_keys<I, K>(keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        Builder::new()
            .minimal_prefix()
            .build_from_keys(keys)?
            .release_mptrie()
    }

    /// Creates a new [`MpTrie`] from input records.
    ///
    /// # Arguments
    ///
    /// - `records`: List of pairs of a string key and an associated value.
    ///
    /// # Errors
    ///
    /// [`CrawdadError`](crate::errors::CrawdadError) will be returned when
    ///
    /// - `records` is empty,
    /// - `records` contains empty strings,
    /// - `records` contains duplicate keys,
    /// - keys in `records` are not sorted,
    /// - the scale of `keys` exceeds the expected one, or
    /// - the scale of the resulting trie exceeds the expected one.
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::{MpTrie, Statistics};
    ///
    /// let records = vec![("世界", 2), ("世界中", 3), ("世直し", 5), ("国民", 7)];
    /// let trie = MpTrie::from_records(records).unwrap();
    ///
    /// assert_eq!(trie.num_elems(), 16);
    /// assert_eq!(trie.num_vacants(), 9);
    /// ```
    pub fn from_records<I, K>(records: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, u32)>,
        K: AsRef<str>,
    {
        Builder::new()
            .minimal_prefix()
            .build_from_records(records)?
            .release_mptrie()
    }

    /// Returns a value associated with an input key if exists.
    ///
    /// # Arguments
    ///
    /// - `key`: Search key.
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::MpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "世直し", "国民"];
    /// let trie = MpTrie::from_keys(&keys).unwrap();
    ///
    /// assert_eq!(trie.exact_match("世界中"), Some(1));
    /// assert_eq!(trie.exact_match("日本中"), None);
    /// ```
    #[inline(always)]
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut node_idx = 0;
        let mut chars = key.as_ref().chars();

        while !self.is_leaf(node_idx) {
            if let Some(c) = chars.next() {
                if let Some(mc) = self.mapper.get(c) {
                    if let Some(child_idx) = self.get_child_idx(node_idx, mc) {
                        node_idx = child_idx;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else if self.has_leaf(node_idx) {
                return Some(self.get_value(self.get_leaf_idx(node_idx)));
            } else {
                return None;
            }
        }

        let tail_pos = self.get_value(node_idx) as usize;
        let mut tail_iter = self.tail_iter(tail_pos);

        for tc in tail_iter.by_ref() {
            if let Some(c) = chars.next() {
                if let Some(mc) = self.mapper.get(c) {
                    if mc != tc {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        if chars.next().is_some() {
            None
        } else {
            Some(tail_iter.value())
        }
    }

    /// Returns an iterator for common prefix search.
    ///
    /// This operation finds all occurrences of keys starting from a search text, and
    /// the occurrences are reported as pairs of value and end position.
    ///
    /// # Arguments
    ///
    /// - `text`: Search text mapped by [`MpTrie::map_text`].
    ///
    /// # Examples
    ///
    /// You can find all occurrences  of keys in a text as follows.
    ///
    /// ```
    /// use crawdad::MpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "世直し", "国民"];
    /// let trie = MpTrie::from_keys(&keys).unwrap();
    ///
    /// let mut mapped = vec![];
    /// trie.map_text("国民が世界中で世直し", &mut mapped);
    ///
    /// let mut matches = vec![];
    /// for i in 0..mapped.len() {
    ///     for m in trie.common_prefix_searcher(&mapped[i..]) {
    ///         matches.push((m.value(), i + m.end()));
    ///     }
    /// }
    /// assert_eq!(matches, vec![(3, 2), (0, 5), (1, 6), (2, 10)]);
    /// ```
    pub const fn common_prefix_searcher<'k, 't>(
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

    /// Prepares a search text for common prefix search.
    ///
    /// # Arguments
    ///
    /// - `text`: Search text.
    /// - `mapped`: Mapped text.
    #[inline(always)]
    pub fn map_text<K>(&self, text: K, mapped: &mut Vec<Option<u32>>)
    where
        K: AsRef<str>,
    {
        mapped.clear();
        for c in text.as_ref().chars() {
            mapped.push(self.mapper.get(c));
        }
    }

    #[inline(always)]
    fn tail_iter(&self, tail_pos: usize) -> TailIter {
        let tail_len = self.tails[tail_pos] as usize;
        TailIter {
            trie: self,
            pos: tail_pos + 1,
            len: tail_len,
        }
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
    fn get_leaf_idx(&self, node_idx: u32) -> u32 {
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

impl Statistics for MpTrie {
    fn heap_bytes(&self) -> usize {
        self.mapper.heap_bytes()
            + self.nodes.len() * std::mem::size_of::<Node>()
            + self.tails.len() * std::mem::size_of::<u8>()
    }

    fn num_elems(&self) -> usize {
        self.nodes.len()
    }

    fn num_vacants(&self) -> usize {
        self.nodes.iter().filter(|nd| nd.is_vacant()).count()
    }
}

/// Iterator created by [`MpTrie::common_prefix_searcher`].
pub struct CommonPrefixSearcher<'k, 't> {
    text: &'k [Option<u32>],
    text_pos: usize,
    trie: &'t MpTrie,
    node_idx: u32,
}

impl Iterator for CommonPrefixSearcher<'_, '_> {
    type Item = Match;

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
                let tail_pos = self.trie.get_value(self.node_idx) as usize;
                let mut tail_iter = self.trie.tail_iter(tail_pos);
                for tc in tail_iter.by_ref() {
                    if self.text_pos == self.text.len() {
                        return None;
                    }
                    if let Some(mc) = self.text[self.text_pos] {
                        if mc != tc {
                            self.text_pos = self.text.len();
                            return None;
                        }
                    } else {
                        self.text_pos = self.text.len();
                        return None;
                    }
                    self.text_pos += 1;
                }
                let value = tail_iter.value();
                let end = self.text_pos;
                self.text_pos = self.text.len();
                return Some(Match { end, value });
            } else if self.trie.has_leaf(self.node_idx) {
                let leaf_idx = self.trie.get_leaf_idx(self.node_idx);
                return Some(Match {
                    end: self.text_pos,
                    value: self.trie.get_value(leaf_idx),
                });
            }
        }
        None
    }
}

struct TailIter<'a> {
    trie: &'a MpTrie,
    pos: usize,
    len: usize,
}

impl TailIter<'_> {
    #[inline(always)]
    fn value(&self) -> u32 {
        utils::unpack_u32(&self.trie.tails[self.pos..], self.trie.value_size)
    }
}

impl Iterator for TailIter<'_> {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            let c = utils::unpack_u32(&self.trie.tails[self.pos..], self.trie.code_size);
            self.pos += self.trie.code_size as usize;
            self.len -= 1;
            Some(c)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let keys = vec!["世界", "世界中", "世直し", "直し中"];
        let trie = MpTrie::from_keys(&keys).unwrap();
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(trie.exact_match(&key), Some(i as u32));
        }
        assert_eq!(trie.exact_match("世"), None);
        assert_eq!(trie.exact_match("日本"), None);
        assert_eq!(trie.exact_match("世界中で"), None);
        assert_eq!(trie.exact_match("直し"), None);
    }

    #[test]
    fn test_common_prefix_search() {
        let keys = vec!["世界", "世界中", "世直し", "直し中"];
        let trie = MpTrie::from_keys(&keys).unwrap();

        let mut mapped = vec![];
        trie.map_text("世界中で世直し中", &mut mapped);

        let mut matches = vec![];
        for i in 0..mapped.len() {
            for m in trie.common_prefix_searcher(&mapped[i..]) {
                matches.push((m.value(), i + m.end()));
            }
        }
        assert_eq!(matches, vec![(0, 2), (1, 3), (2, 7), (3, 8)]);
    }
}
