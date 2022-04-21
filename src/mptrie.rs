//! A minimal-prefix trie form that is memory-efficient for long strings.
use crate::builder::Builder;
use crate::errors::Result;
use crate::mapper::CodeMapper;
use crate::{utils, MappedChar, Match, Node};

use crate::END_CODE;

use alloc::vec::Vec;

use core::mem::size_of;

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
    /// use crawdad::MpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = MpTrie::from_keys(keys).unwrap();
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
            .release_mptrie()
    }

    /// Creates a new [`MpTrie`] from input records.
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
    /// use crawdad::MpTrie;
    ///
    /// let records = vec![("世界", 2), ("世界中", 3), ("国民", 2)];
    /// let trie = MpTrie::from_records(records).unwrap();
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
            .release_mptrie()
    }

    /// Serializes the data structure into a [`Vec`].
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::MpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = MpTrie::from_keys(&keys).unwrap();
    /// let bytes = trie.serialize_to_vec();
    /// ```
    pub fn serialize_to_vec(&self) -> Vec<u8> {
        let mut dest = Vec::with_capacity(self.io_bytes());
        self.mapper.serialize_into_vec(&mut dest);
        dest.extend_from_slice(&u32::try_from(self.nodes.len()).unwrap().to_le_bytes());
        for node in &self.nodes {
            dest.extend_from_slice(&node.serialize());
        }
        dest.extend_from_slice(&u32::try_from(self.tails.len()).unwrap().to_le_bytes());
        dest.extend_from_slice(&self.tails);
        dest.extend_from_slice(&[self.code_size]);
        dest.extend_from_slice(&[self.value_size]);
        dest
    }

    /// Deserializes the data structure from a given byte slice.
    ///
    /// # Arguments
    ///
    /// * `source` - A source byte slice.
    ///
    /// # Returns
    ///
    /// A tuple of the data structure and the slice not used for the deserialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use crawdad::MpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = MpTrie::from_keys(&keys).unwrap();
    ///
    /// let bytes = trie.serialize_to_vec();
    /// let (other, _) = MpTrie::deserialize_from_slice(&bytes);
    ///
    /// assert_eq!(trie.io_bytes(), other.io_bytes());
    /// ```
    pub fn deserialize_from_slice(source: &[u8]) -> (Self, &[u8]) {
        let (mapper, mut source) = CodeMapper::deserialize_from_slice(source);
        let nodes = {
            let len = u32::from_le_bytes(source[..4].try_into().unwrap()) as usize;
            source = &source[4..];
            let mut nodes = Vec::with_capacity(len);
            for _ in 0..len {
                nodes.push(Node::deserialize(
                    source[..Node::io_bytes()].try_into().unwrap(),
                ));
                source = &source[Node::io_bytes()..];
            }
            nodes
        };
        let tails = {
            let len = u32::from_le_bytes(source[..4].try_into().unwrap()) as usize;
            source = &source[4..];
            let tails = source[..len].to_vec();
            source = &source[len..];
            tails
        };
        let code_size = source[0];
        let value_size = source[1];
        (
            Self {
                mapper,
                nodes,
                tails,
                code_size,
                value_size,
            },
            &source[2..],
        )
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
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = MpTrie::from_keys(&keys).unwrap();
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
        let mut chars = key.into_iter();

        while !self.is_leaf(node_idx) {
            if let Some(c) = chars.next() {
                if let Some(child_idx) = self
                    .mapper
                    .get(c)
                    .and_then(|mc| self.get_child_idx(node_idx, mc))
                {
                    node_idx = child_idx;
                } else {
                    return None;
                }
            } else if self.has_leaf(node_idx) {
                return Some(self.get_value(self.get_leaf_idx(node_idx)));
            } else {
                return None;
            }
        }

        let tail_pos = usize::try_from(self.get_value(node_idx)).unwrap();
        let mut tail_iter = self.tail_iter(tail_pos);

        for tc in tail_iter.by_ref() {
            chars
                .next()
                .and_then(|c| self.mapper.get(c))
                .filter(|&mc| mc == tc)?;
        }

        if chars.next().is_some() {
            None
        } else {
            Some(tail_iter.value())
        }
    }

    /// Returns a common prefix searcher.
    ///
    /// The searcher finds all occurrences of keys starting from an input haystack, and
    /// the occurrences are reported as a sequence of [`Match`](crate::Match).
    ///
    /// # Arguments
    ///
    /// - `haystack`: Search haystack mapped by [`MpTrie::map_haystack`].
    ///
    /// # Examples
    ///
    /// You can find all occurrences  of keys in a haystack as follows.
    ///
    /// ```
    /// use crawdad::MpTrie;
    ///
    /// let keys = vec!["世界", "世界中", "国民"];
    /// let trie = MpTrie::from_keys(&keys).unwrap();
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
    fn tail_iter(&self, tail_pos: usize) -> TailIter {
        let tail_len = usize::try_from(self.tails[tail_pos]).unwrap();
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
    fn node_ref(&self, node_idx: u32) -> &Node {
        &self.nodes[usize::try_from(node_idx).unwrap()]
    }

    #[inline(always)]
    fn get_base(&self, node_idx: u32) -> u32 {
        self.node_ref(node_idx).get_base()
    }

    #[inline(always)]
    fn get_check(&self, node_idx: u32) -> u32 {
        self.node_ref(node_idx).get_check()
    }

    #[inline(always)]
    fn is_leaf(&self, node_idx: u32) -> bool {
        self.node_ref(node_idx).is_leaf()
    }

    #[inline(always)]
    fn has_leaf(&self, node_idx: u32) -> bool {
        self.node_ref(node_idx).has_leaf()
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
        self.node_ref(node_idx).get_base()
    }

    /// Returns the total amount of heap used by this automaton in bytes.
    pub fn heap_bytes(&self) -> usize {
        self.mapper.heap_bytes()
            + self.nodes.len() * size_of::<Node>()
            + self.tails.len() * size_of::<u8>()
    }

    /// Returns the total amount of bytes to serialize the data structure.
    pub fn io_bytes(&self) -> usize {
        self.mapper.io_bytes()
            + self.nodes.len() * Node::io_bytes()
            + size_of::<u32>()
            + self.tails.len() * size_of::<u8>()
            + size_of::<u32>()
            + size_of::<u8>() * 2
    }

    /// Returns the number of reserved elements.
    pub fn num_elems(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of vacant elements.
    pub fn num_vacants(&self) -> usize {
        self.nodes.iter().filter(|nd| nd.is_vacant()).count()
    }
}

/// Common prefix searcher created by [`MpTrie::common_prefix_searcher`].
pub struct CommonPrefixSearcher<'t> {
    trie: &'t MpTrie,
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
    trie: &'t MpTrie,
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
            if let Some(child_idx) = mc.c.and_then(|c| self.trie.get_child_idx(self.node_idx, c)) {
                self.node_idx = child_idx;
            } else {
                self.haystack_pos = self.haystack.len();
                return None;
            }

            self.haystack_pos += 1;

            if self.trie.is_leaf(self.node_idx) {
                let tail_pos = usize::try_from(self.trie.get_value(self.node_idx)).unwrap();
                let mut tail_iter = self.trie.tail_iter(tail_pos);

                for tc in tail_iter.by_ref() {
                    if self.haystack_pos == self.haystack.len() {
                        return None;
                    }
                    let mc = self.haystack[self.haystack_pos];
                    if mc.c.filter(|&c| c == tc).is_none() {
                        self.haystack_pos = self.haystack.len();
                        return None;
                    }
                    self.haystack_pos += 1;
                }

                let value = tail_iter.value();
                let end_chars = self.haystack_pos;
                let end_bytes = self.haystack[end_chars - 1].end_bytes;

                self.haystack_pos = self.haystack.len();

                return Some(Match {
                    value,
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
            self.pos += usize::try_from(self.trie.code_size).unwrap();
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
        let keys = vec!["世界", "世界中", "世論調査", "統計調査"];
        let trie = MpTrie::from_keys(&keys).unwrap();
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
        let trie = MpTrie::from_keys(&keys).unwrap();

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

    #[test]
    fn test_serialize() {
        let keys = vec!["世界", "世界中", "世論調査", "統計調査"];
        let trie = MpTrie::from_keys(&keys).unwrap();

        let bytes = trie.serialize_to_vec();
        assert_eq!(trie.io_bytes(), bytes.len());

        let (other, remain) = MpTrie::deserialize_from_slice(&bytes);
        assert!(remain.is_empty());

        assert_eq!(trie.mapper, other.mapper);
        assert_eq!(trie.nodes, other.nodes);
        assert_eq!(trie.tails, other.tails);
        assert_eq!(trie.code_size, other.code_size);
        assert_eq!(trie.value_size, other.value_size);
    }

    #[test]
    fn test_empty_set() {
        assert!(MpTrie::from_keys(&[""][0..0]).is_err());
    }

    #[test]
    fn test_empty_char() {
        assert!(MpTrie::from_keys([""]).is_err());
    }

    #[test]
    fn test_empty_key() {
        assert!(MpTrie::from_keys(["", "AAA"]).is_err());
    }

    #[test]
    fn test_unsorted_keys() {
        assert!(MpTrie::from_keys(["BB", "AA"]).is_err());
        assert!(MpTrie::from_keys(["AAA", "AA"]).is_err());
    }

    #[test]
    fn test_duplicate_keys() {
        assert!(MpTrie::from_keys(["AA", "AA"]).is_err());
    }
}
