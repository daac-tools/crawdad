//! A fuzzy minimal-prefix trie form that is more memory-efficient
//! and allows for some false positives (Experimental).
use crate::builder::Builder;
use crate::errors::Result;
use crate::mapper::CodeMapper;
use crate::{utils, Match, Node, Statistics};

use crate::END_CODE;

use sucds::RsBitVector;

/// A fuzzy minimal-prefix trie form that is more memory-efficient
/// and allows for some false positives (Experimental).
pub struct FmpTrie {
    pub(crate) mapper: CodeMapper,
    pub(crate) nodes: Vec<Node>,
    pub(crate) ranks: RsBitVector,
    pub(crate) auxes: Vec<(u8, u8)>,
}

impl FmpTrie {
    /// Creates a new [`FmpTrie`] from input keys.
    ///
    /// Values in `[0..n-1]` will be associated with keys in the lexicographical order,
    /// where `n` is the number of keys.
    ///
    /// # Arguments
    ///
    /// - `keys`: Sorted list of string keys.
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
    /// use crawdad::{FmpTrie, Statistics};
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = FmpTrie::from_keys(keys).unwrap();
    ///
    /// assert_eq!(trie.num_elems(), 8);
    /// ```
    pub fn from_keys<I, K>(keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        Builder::new()
            .minimal_prefix()
            .build_from_keys(keys)?
            .release_mpftrie()
    }

    /// Creates a new [`FmpTrie`] from input records.
    ///
    /// # Arguments
    ///
    /// - `records`: Sorted list of key-value pairs.
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
    /// use crawdad::{FmpTrie, Statistics};
    ///
    /// let records = vec![("世界", 2), ("世界中", 3), ("国民", 2)];
    /// let trie = FmpTrie::from_records(records).unwrap();
    ///
    /// assert_eq!(trie.num_elems(), 8);
    /// ```
    pub fn from_records<I, K>(records: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, u32)>,
        K: AsRef<str>,
    {
        Builder::new()
            .minimal_prefix()
            .build_from_records(records)?
            .release_mpftrie()
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
    /// use crawdad::FmpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = FmpTrie::from_keys(&keys).unwrap();
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

        let value = self.get_value(node_idx);
        let suffix: Vec<_> = chars.map(|c| self.mapper.get(c)).collect();

        assert!(self.ranks.get_bit(node_idx as usize));
        let aux_pos = self.ranks.rank1(node_idx as usize);
        let (tail_len, tail_hash) = self.auxes[aux_pos];

        if tail_len as usize != suffix.len() {
            return None;
        }
        if let Some(suf_hash) = utils::murmur_hash2(&suffix) {
            if tail_hash == suf_hash as u8 {
                return Some(value);
            }
        }
        None
    }

    /// Returns an iterator for common prefix search.
    ///
    /// This operation finds all occurrences of keys starting from a search text, and
    /// the occurrences are reported as a sequence of [`Match`](crate::Match).
    ///
    /// # Arguments
    ///
    /// - `text`: Search text mapped by [`FmpTrie::map_text`].
    ///
    /// # Examples
    ///
    /// You can find all occurrences  of keys in a text as follows.
    ///
    /// ```
    /// use crawdad::FmpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = FmpTrie::from_keys(&keys).unwrap();
    ///
    /// let mut mapped = vec![];
    /// trie.map_text("国民が世界中にて", &mut mapped);
    ///
    /// let mut matches = vec![];
    /// for i in 0..mapped.len() {
    ///     for m in trie.common_prefix_searcher(&mapped[i..]) {
    ///         matches.push((m.value(), i + m.end()));
    ///     }
    /// }
    /// assert_eq!(matches, vec![(2, 2), (0, 5), (1, 6)]);
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

impl Statistics for FmpTrie {
    fn heap_bytes(&self) -> usize {
        self.mapper.heap_bytes()
            + self.nodes.len() * std::mem::size_of::<Node>()
            + self.auxes.len() * std::mem::size_of::<(u8, u8)>()
            + self.ranks.size_in_bytes()
    }

    fn num_elems(&self) -> usize {
        self.nodes.len()
    }

    fn num_vacants(&self) -> usize {
        self.nodes.iter().filter(|nd| nd.is_vacant()).count()
    }
}

/// Iterator created by [`FmpTrie::common_prefix_searcher`].
pub struct CommonPrefixSearcher<'k, 't> {
    text: &'k [Option<u32>],
    text_pos: usize,
    trie: &'t FmpTrie,
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
                let value = self.trie.get_value(self.node_idx);

                assert!(self.trie.ranks.get_bit(self.node_idx as usize));
                let aux_pos = self.trie.ranks.rank1(self.node_idx as usize);
                let (tail_len, tail_hash) = self.trie.auxes[aux_pos];

                if let Some(suffix) = self
                    .text
                    .get(self.text_pos..self.text_pos + tail_len as usize)
                {
                    if let Some(suf_hash) = utils::murmur_hash2(suffix) {
                        if tail_hash == suf_hash as u8 {
                            let end = self.text_pos + suffix.len();
                            self.text_pos = self.text.len();
                            return Some(Match { end, value });
                        }
                    }
                }
                self.text_pos = self.text.len();
                return None;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let keys = vec!["世界", "世界中", "世論調査", "統計調査"];
        let trie = FmpTrie::from_keys(&keys).unwrap();
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(trie.exact_match(&key), Some(i as u32));
        }
        assert_eq!(trie.exact_match("世"), None);
        assert_eq!(trie.exact_match("世論"), None);
        assert_eq!(trie.exact_match("世界中で"), None);
        assert_eq!(trie.exact_match("統計"), None);
        assert_eq!(trie.exact_match("統計調"), None);
        assert_eq!(trie.exact_match("日本"), None);
    }

    #[test]
    fn test_common_prefix_search() {
        let keys = vec!["世界", "世界中", "世論調査", "統計調査"];
        let trie = FmpTrie::from_keys(&keys).unwrap();

        let mut mapped = vec![];
        trie.map_text("世界中の統計世論調査", &mut mapped);

        let mut matches = vec![];
        for i in 0..mapped.len() {
            for m in trie.common_prefix_searcher(&mapped[i..]) {
                matches.push((m.value(), i + m.end()));
            }
        }
        assert_eq!(matches, vec![(0, 2), (1, 3), (2, 10)]);
    }
}