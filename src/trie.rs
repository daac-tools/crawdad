//! A standard trie form that often provides the fastest queries.
use crate::builder::Builder;
use crate::errors::Result;
use crate::mapper::CodeMapper;
use crate::{MappedChar, Match, Node, Statistics};

use crate::END_CODE;

/// A standard trie form that often provides the fastest queries.
pub struct Trie {
    pub(crate) mapper: CodeMapper,
    pub(crate) nodes: Vec<Node>,
}

impl Trie {
    /// Creates a new [`Trie`] from input keys.
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
    /// use crawdad::{Trie, Statistics};
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = Trie::from_keys(keys).unwrap();
    ///
    /// assert_eq!(trie.num_elems(), 8);
    /// ```
    pub fn from_keys<I, K>(keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        Builder::new().build_from_keys(keys)?.release_trie()
    }

    /// Creates a new [`Trie`] from input records.
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
    /// use crawdad::{Trie, Statistics};
    ///
    /// let records = vec![("世界", 2), ("世界中", 3), ("国民", 2)];
    /// let trie = Trie::from_records(records).unwrap();
    ///
    /// assert_eq!(trie.num_elems(), 8);
    /// ```
    pub fn from_records<I, K>(records: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, u32)>,
        K: AsRef<str>,
    {
        Builder::new().build_from_records(records)?.release_trie()
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
    /// use crawdad::Trie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = Trie::from_keys(&keys).unwrap();
    ///
    /// assert_eq!(trie.exact_match("世界中".chars()), Some(1));
    /// assert_eq!(trie.exact_match("日本中".chars()), None);
    /// ```
    #[inline(always)]
    pub fn exact_match<I>(&self, key: I) -> Option<u32>
    where
        I: IntoIterator<Item = char>,
    {
        let mut node_idx = 0;
        for c in key {
            if let Some(mc) = self.mapper.get(c) {
                if let Some(child_idx) = self.get_child_idx(node_idx, mc) {
                    node_idx = child_idx;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        if self.is_leaf(node_idx) {
            Some(self.get_value(node_idx))
        } else if self.has_leaf(node_idx) {
            Some(self.get_value(self.get_leaf_idx(node_idx)))
        } else {
            None
        }
    }

    /// Returns a common prefix searcher.
    ///
    /// The searcher finds all occurrences of keys starting from an input haystack, and
    /// the occurrences are reported as a sequence of [`Match`](crate::Match).
    ///
    /// # Examples
    ///
    /// You can find all occurrences of keys in a haystack as follows.
    ///
    /// ```
    /// use crawdad::Trie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = Trie::from_keys(&keys).unwrap();
    ///
    /// let mut searcher = trie.common_prefix_searcher();
    /// searcher.update_haystack("国民が世界中にて".chars());
    ///
    /// let mut matches = vec![];
    /// for i in 0..searcher.len_chars() {
    ///     for m in searcher.search(i) {
    ///         matches.push((
    ///             m.value(),
    ///             m.start_chars(), m.end_chars(),
    ///             m.start_bytes(), m.end_bytes(),
    ///         ));
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     matches,
    ///     vec![(2, 0, 2, 0, 6), (0, 3, 5, 9, 15), (1, 3, 6, 9, 18)]
    /// );
    /// ```
    pub const fn common_prefix_searcher(&self) -> CommonPrefixSearcher {
        CommonPrefixSearcher {
            trie: self,
            haystack: vec![],
        }
    }

    /// Prepares a search haystack for common prefix search.
    ///
    /// # Arguments
    ///
    /// - `haystack`: Search haystack.
    /// - `mapped`: Mapped haystack.
    #[inline(always)]
    fn map_haystack<I>(&self, haystack: I, mapped: &mut Vec<MappedChar>)
    where
        I: IntoIterator<Item = char>,
    {
        mapped.clear();
        let mut end_bytes = 0;
        for c in haystack {
            end_bytes += c.len_utf8();
            mapped.push(MappedChar {
                c: self.mapper.get(c),
                end_bytes,
            });
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

impl Statistics for Trie {
    fn heap_bytes(&self) -> usize {
        self.mapper.heap_bytes() + self.nodes.len() * std::mem::size_of::<Node>()
    }

    fn num_elems(&self) -> usize {
        self.nodes.len()
    }

    fn num_vacants(&self) -> usize {
        self.nodes.iter().filter(|nd| nd.is_vacant()).count()
    }
}

/// Common prefix searcher created by [`Trie::common_prefix_searcher`].
pub struct CommonPrefixSearcher<'t> {
    trie: &'t Trie,
    haystack: Vec<MappedChar>,
}

impl CommonPrefixSearcher<'_> {
    /// Sets a search haystack.
    pub fn update_haystack<I>(&mut self, haystack: I)
    where
        I: IntoIterator<Item = char>,
    {
        self.trie.map_haystack(haystack, &mut self.haystack);
    }

    /// Gets the haystack length in characters.
    pub fn len_chars(&self) -> usize {
        self.haystack.len()
    }

    /// Creates an iterator to search for the haystack in the given range.
    pub fn search(&self, start: usize) -> CommonPrefixSearchIter {
        let start_chars = start;
        let start_bytes = if start_chars == 0 {
            0
        } else {
            self.haystack[start_chars - 1].end_bytes
        };
        CommonPrefixSearchIter {
            haystack: &self.haystack,
            haystack_pos: start_chars,
            trie: self.trie,
            node_idx: 0,
            start_chars,
            start_bytes,
        }
    }
}

/// Iterator for common prefix search.
pub struct CommonPrefixSearchIter<'k, 't> {
    haystack: &'k [MappedChar],
    haystack_pos: usize,
    trie: &'t Trie,
    node_idx: u32,
    start_chars: usize,
    start_bytes: usize,
}

impl Iterator for CommonPrefixSearchIter<'_, '_> {
    type Item = Match;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        while self.haystack_pos < self.haystack.len() {
            let mc = self.haystack[self.haystack_pos];
            if let Some(c) = mc.c {
                if let Some(child_idx) = self.trie.get_child_idx(self.node_idx, c) {
                    self.node_idx = child_idx;
                } else {
                    self.haystack_pos = self.haystack.len();
                    return None;
                }
            } else {
                self.haystack_pos = self.haystack.len();
                return None;
            }

            self.haystack_pos += 1;

            if self.trie.is_leaf(self.node_idx) {
                let end_chars = self.haystack_pos;
                let end_bytes = self.haystack[end_chars - 1].end_bytes;
                self.haystack_pos = self.haystack.len();
                return Some(Match {
                    value: self.trie.get_value(self.node_idx),
                    range_chars: self.start_chars..end_chars,
                    range_bytes: self.start_bytes..end_bytes,
                });
            } else if self.trie.has_leaf(self.node_idx) {
                let leaf_idx = self.trie.get_leaf_idx(self.node_idx);
                let end_chars = self.haystack_pos;
                let end_bytes = self.haystack[end_chars - 1].end_bytes;
                return Some(Match {
                    value: self.trie.get_value(leaf_idx),
                    range_chars: self.start_chars..end_chars,
                    range_bytes: self.start_bytes..end_bytes,
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
        let trie = Trie::from_keys(&keys).unwrap();
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(
                trie.exact_match(key.chars()),
                Some(u32::try_from(i).unwrap())
            );
        }
        assert_eq!(trie.exact_match("世".chars()), None);
        assert_eq!(trie.exact_match("世論".chars()), None);
        assert_eq!(trie.exact_match("世界中で".chars()), None);
        assert_eq!(trie.exact_match("統計".chars()), None);
        assert_eq!(trie.exact_match("統計調".chars()), None);
        assert_eq!(trie.exact_match("日本".chars()), None);
    }

    #[test]
    fn test_common_prefix_search() {
        let keys = vec!["世界", "世界中", "世論調査", "統計調査"];
        let trie = Trie::from_keys(&keys).unwrap();

        let mut searcher = trie.common_prefix_searcher();
        searcher.update_haystack("世界中の統計世論調査".chars());

        let mut matches = vec![];
        for i in 0..searcher.len_chars() {
            for m in searcher.search(i) {
                matches.push((
                    m.value(),
                    m.start_chars(),
                    m.end_chars(),
                    m.start_bytes(),
                    m.end_bytes(),
                ));
            }
        }

        assert_eq!(
            matches,
            vec![(0, 0, 2, 0, 6), (1, 0, 3, 0, 9), (2, 6, 10, 18, 30)]
        );
    }
}
