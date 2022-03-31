use crate::errors::{CrawdadError, Result};
use crate::mapper::CodeMapper;
use crate::{utils, MpTrie, MpfTrie, Node, Trie};
use crate::{END_CODE, END_MARKER, INVALID_IDX, MAX_VALUE, OFFSET_MASK};

use std::cmp::Ordering;

use sucds::RsBitVector;

#[derive(Default)]
struct Record {
    key: Vec<char>,
    value: u32,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Suffix {
    key: Vec<char>,
    value: u32,
}

#[derive(Default)]
pub struct Builder {
    records: Vec<Record>,
    mapper: CodeMapper,
    nodes: Vec<Node>,
    suffixes: Option<Vec<Suffix>>,
    labels: Vec<u32>,
    head_idx: u32,
    block_len: u32,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn minimal_prefix(mut self) -> Self {
        self.suffixes = Some(vec![]);
        self
    }

    pub fn build_from_keys<I, K>(self, keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.build_from_records(keys.into_iter().enumerate().map(|(i, k)| (k, i as u32)))
    }

    pub fn build_from_records<I, K>(mut self, records: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, u32)>,
        K: AsRef<str>,
    {
        self.records = records
            .into_iter()
            .map(|(k, v)| Record {
                key: k.as_ref().chars().collect(),
                value: v,
            })
            .collect();

        for &Record { key: _, value } in &self.records {
            if MAX_VALUE < value {
                return Err(CrawdadError::scale("input value", MAX_VALUE));
            }
        }

        self.mapper = CodeMapper::new(&make_freqs(&self.records)?);
        assert_eq!(self.mapper.get(END_MARKER).unwrap(), END_CODE);

        make_prefix_free(&mut self.records)?;

        self.block_len = get_block_len(self.mapper.alphabet_size());
        self.init_array();
        self.arrange_nodes(0, self.records.len(), 0, 0)?;
        self.finish();

        Ok(self)
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn release_trie(self) -> Result<Trie> {
        if self.suffixes.is_some() {
            Err(CrawdadError::setup("minimal_prefix must be disabled."))
        } else {
            let Builder { nodes, mapper, .. } = self;
            Ok(Trie { nodes, mapper })
        }
    }

    pub fn release_mptrie(self) -> Result<MpTrie> {
        if self.suffixes.is_none() {
            return Err(CrawdadError::setup("minimal_prefix must be enabled."));
        }

        let Builder {
            mapper,
            mut nodes,
            suffixes,
            ..
        } = self;

        let mut tails = vec![];
        let suffixes = suffixes.unwrap();

        let max_code = (mapper.alphabet_size() - 1) as u32;
        let code_size = utils::pack_size(max_code);

        let max_value = suffixes.iter().map(|s| s.value).max().unwrap();
        let value_size = utils::pack_size(max_value);

        for node_idx in 0..nodes.len() {
            if nodes[node_idx].is_vacant() {
                continue;
            }
            if !nodes[node_idx].is_leaf() {
                continue;
            }

            debug_assert_eq!(nodes[node_idx].check & !OFFSET_MASK, 0);
            let parent_idx = nodes[node_idx].check as usize;
            let suf_idx = (nodes[node_idx].base & OFFSET_MASK) as usize;
            let suffix = &suffixes[suf_idx];

            // HasLeaf?
            if nodes[parent_idx].has_leaf() {
                // `node_idx` is indicated from `parent_idx` with END_CODE?
                if nodes[parent_idx].base == node_idx as u32 {
                    assert!(suffix.key.is_empty());
                    nodes[node_idx].base = suffix.value | !OFFSET_MASK;
                    continue;
                }
            }

            nodes[node_idx].base = tails.len() as u32 | !OFFSET_MASK;
            tails.push(suffix.key.len() as u8);
            suffix
                .key
                .iter()
                .map(|&c| mapper.get(c).unwrap())
                .for_each(|c| utils::pack_u32(&mut tails, c, code_size));
            utils::pack_u32(&mut tails, suffix.value, value_size);
        }

        Ok(MpTrie {
            mapper,
            nodes,
            tails,
            code_size,
            value_size,
        })
    }

    pub fn release_mpftrie(self) -> Result<MpfTrie> {
        if self.suffixes.is_none() {
            return Err(CrawdadError::setup("minimal_prefix must be enabled."));
        }

        let Builder {
            mapper,
            mut nodes,
            suffixes,
            ..
        } = self;

        let mut ranks = vec![false; nodes.len()];
        let mut auxes = vec![];

        let suffixes = suffixes.unwrap();

        for node_idx in 0..nodes.len() {
            if nodes[node_idx].is_vacant() {
                continue;
            }
            if !nodes[node_idx].is_leaf() {
                continue;
            }

            debug_assert_eq!(nodes[node_idx].check & !OFFSET_MASK, 0);
            let parent_idx = nodes[node_idx].check as usize;
            let suf_idx = (nodes[node_idx].base & OFFSET_MASK) as usize;
            let suffix = &suffixes[suf_idx];

            // HasLeaf?
            if nodes[parent_idx].has_leaf() {
                // `node_idx` is indicated from `parent_idx` with END_CODE?
                if nodes[parent_idx].base == node_idx as u32 {
                    assert!(suffix.key.is_empty());
                    nodes[node_idx].base = suffix.value | !OFFSET_MASK;
                    continue;
                }
            }

            nodes[node_idx].base = suffix.value | !OFFSET_MASK;
            ranks[node_idx] = true;

            let tail: Vec<_> = suffix.key.iter().map(|&c| mapper.get(c)).collect();
            let tail_hash = utils::murmur_hash2(&tail).unwrap();
            auxes.push((tail.len() as u8, tail_hash as u8));
        }

        Ok(MpfTrie {
            mapper,
            nodes,
            ranks: RsBitVector::from_bits(ranks),
            auxes,
        })
    }

    #[inline(always)]
    fn num_nodes(&self) -> u32 {
        self.nodes.len() as u32
    }

    fn init_array(&mut self) {
        self.nodes.clear();
        self.nodes.resize(self.block_len as usize, Node::default());

        for i in 0..self.block_len {
            if i == 0 {
                self.set_prev(i, self.block_len - 1);
            } else {
                self.set_prev(i, i - 1);
            }
            if i == self.block_len - 1 {
                self.set_next(i, 0);
            } else {
                self.set_next(i, i + 1);
            }
        }

        self.head_idx = 0;
        self.fix_node(0);
    }

    fn arrange_nodes(
        &mut self,
        spos: usize,
        epos: usize,
        depth: usize,
        node_idx: u32,
    ) -> Result<()> {
        debug_assert!(self.is_fixed(node_idx));

        if let Some(suffixes) = self.suffixes.as_mut() {
            if spos + 1 == epos {
                let suffix_idx = suffixes.len() as u32;
                self.nodes[node_idx as usize].base = suffix_idx | !OFFSET_MASK;
                suffixes.push(Suffix {
                    key: pop_end_marker(&self.records[spos].key[depth..]),
                    value: self.records[spos].value,
                });
                return Ok(());
            }
        } else if self.records[spos].key.len() == depth {
            debug_assert_eq!(spos + 1, epos);
            debug_assert_eq!(self.records[spos].value & !OFFSET_MASK, 0);
            // Sets IsLeaf = True
            self.nodes[node_idx as usize].base = self.records[spos].value | !OFFSET_MASK;
            // Note: HasLeaf must not be set here and should be set in finish()
            // because MSB of check is used to indicate vacant element.
            return Ok(());
        }

        self.fetch_labels(spos, epos, depth);
        let base = self.define_nodes(node_idx)?;

        let mut i1 = spos;
        let mut c1 = self.records[i1].key[depth];
        for i2 in spos + 1..epos {
            let c2 = self.records[i2].key[depth];
            if c1 != c2 {
                debug_assert!(c1 < c2);
                let child_idx = base ^ self.mapper.get(c1).unwrap();
                self.arrange_nodes(i1, i2, depth + 1, child_idx)?;
                i1 = i2;
                c1 = c2;
            }
        }
        let child_idx = base ^ self.mapper.get(c1).unwrap();
        self.arrange_nodes(i1, epos, depth + 1, child_idx)
    }

    fn finish(&mut self) {
        self.nodes[0].check = OFFSET_MASK;
        if self.head_idx != INVALID_IDX {
            let mut node_idx = self.head_idx;
            loop {
                let next_idx = self.get_next(node_idx);
                self.nodes[node_idx as usize].base = OFFSET_MASK;
                self.nodes[node_idx as usize].check = OFFSET_MASK;
                node_idx = next_idx;
                if node_idx == self.head_idx {
                    break;
                }
            }
        }
        for node_idx in 0..self.nodes.len() {
            if self.nodes[node_idx].is_vacant() {
                continue;
            }
            if self.nodes[node_idx].is_leaf() {
                continue;
            }
            let end_idx = self.nodes[node_idx].base ^ END_CODE;
            if self.nodes[end_idx as usize].check as usize == node_idx {
                // Sets HasLeaf = True
                self.nodes[node_idx].check |= !OFFSET_MASK;
            }
        }
    }

    fn fetch_labels(&mut self, spos: usize, epos: usize, depth: usize) {
        self.labels.clear();
        let mut c1 = self.records[spos].key[depth];
        for i in spos + 1..epos {
            let c2 = self.records[i].key[depth];
            if c1 != c2 {
                debug_assert!(c1 < c2);
                self.labels.push(self.mapper.get(c1).unwrap());
                c1 = c2;
            }
        }
        self.labels.push(self.mapper.get(c1).unwrap());
    }

    fn define_nodes(&mut self, node_idx: u32) -> Result<u32> {
        let base = self.find_base(&self.labels);
        if base >= self.num_nodes() {
            self.enlarge()?;
        }

        self.nodes[node_idx as usize].base = base;
        for i in 0..self.labels.len() {
            let child_idx = base ^ self.labels[i];
            self.fix_node(child_idx);
            self.nodes[child_idx as usize].check = node_idx;
        }
        Ok(base)
    }

    fn find_base(&self, labels: &[u32]) -> u32 {
        debug_assert!(!labels.is_empty());

        if self.head_idx == INVALID_IDX {
            return self.num_nodes() ^ labels[0];
        }

        let mut node_idx = self.head_idx;
        loop {
            let base = node_idx ^ labels[0];
            if self.verify_base(base, labels) {
                return base;
            }
            node_idx = self.get_next(node_idx);
            if node_idx == self.head_idx {
                break;
            }
        }
        self.num_nodes() ^ labels[0]
    }

    #[inline(always)]
    fn verify_base(&self, base: u32, labels: &[u32]) -> bool {
        for &label in labels {
            let node_idx = base ^ label;
            if self.is_fixed(node_idx) {
                return false;
            }
        }
        true
    }

    #[inline(always)]
    fn fix_node(&mut self, node_idx: u32) {
        debug_assert!(!self.is_fixed(node_idx));

        let next = self.get_next(node_idx);
        let prev = self.get_prev(node_idx);

        self.set_next(prev, next);
        self.set_prev(next, prev);
        self.set_fixed(node_idx);

        if self.head_idx == node_idx {
            if next == node_idx {
                self.head_idx = INVALID_IDX;
            } else {
                self.head_idx = next;
            }
        }
    }

    fn enlarge(&mut self) -> Result<()> {
        let old_len = self.num_nodes();
        let new_len = old_len + self.block_len;

        if OFFSET_MASK < new_len {
            return Err(CrawdadError::scale("num_nodes", OFFSET_MASK));
        }

        for i in old_len..new_len {
            self.nodes.push(Node::default());
            self.set_next(i, i + 1);
            self.set_prev(i, i - 1);
        }

        if self.head_idx == INVALID_IDX {
            self.set_prev(old_len, new_len - 1);
            self.set_next(new_len - 1, old_len);
            self.head_idx = old_len;
        } else {
            let head_idx = self.head_idx;
            let tail_idx = self.get_prev(head_idx);
            self.set_prev(old_len, tail_idx);
            self.set_next(tail_idx, old_len);
            self.set_next(new_len - 1, head_idx);
            self.set_prev(head_idx, new_len - 1);
        }

        Ok(())
    }

    // If the most significant bit is unset, the state is fixed.
    #[inline(always)]
    fn is_fixed(&self, i: u32) -> bool {
        self.nodes[i as usize].check & !OFFSET_MASK == 0
    }

    // Unset the most significant bit.
    #[inline(always)]
    fn set_fixed(&mut self, i: u32) {
        debug_assert!(!self.is_fixed(i));
        self.nodes[i as usize].base = INVALID_IDX;
        self.nodes[i as usize].check &= OFFSET_MASK;
    }

    #[inline(always)]
    fn get_next(&self, i: u32) -> u32 {
        debug_assert_ne!(self.nodes[i as usize].base & !OFFSET_MASK, 0);
        self.nodes[i as usize].base & OFFSET_MASK
    }

    #[inline(always)]
    fn get_prev(&self, i: u32) -> u32 {
        debug_assert_ne!(self.nodes[i as usize].check & !OFFSET_MASK, 0);
        self.nodes[i as usize].check & OFFSET_MASK
    }

    #[inline(always)]
    fn set_next(&mut self, i: u32, x: u32) {
        debug_assert_eq!(x & !OFFSET_MASK, 0);
        self.nodes[i as usize].base = x | !OFFSET_MASK
    }

    #[inline(always)]
    fn set_prev(&mut self, i: u32, x: u32) {
        debug_assert_eq!(x & !OFFSET_MASK, 0);
        self.nodes[i as usize].check = x | !OFFSET_MASK
    }
}

fn make_freqs(records: &[Record]) -> Result<Vec<u32>> {
    let mut freqs = vec![];
    for rec in records {
        for &c in &rec.key {
            let c = c as usize;
            if freqs.len() <= c {
                freqs.resize(c + 1, 0);
            }
            freqs[c] += 1;
        }
    }
    if freqs[END_MARKER as usize] != 0 {
        Err(CrawdadError::input("END_MARKER must not be contained."))
    } else {
        freqs[END_MARKER as usize] = u32::MAX;
        Ok(freqs)
    }
}

fn make_prefix_free(records: &mut [Record]) -> Result<()> {
    if records.is_empty() {
        return Err(CrawdadError::input("records must not be empty."));
    }
    if records[0].key.is_empty() {
        return Err(CrawdadError::input(
            "records must not contain an empty key.",
        ));
    }
    for i in 1..records.len() {
        let (lcp, cmp) = utils::longest_common_prefix(&records[i - 1].key, &records[i].key);
        match cmp {
            Ordering::Less => {
                // Startswith?
                if lcp == records[i - 1].key.len() {
                    records[i - 1].key.push(END_MARKER);
                }
            }
            Ordering::Equal => {
                return Err(CrawdadError::input(
                    "records must not contain duplicated keys.",
                ));
            }
            Ordering::Greater => {
                return Err(CrawdadError::input("records must be sorted."));
            }
        }
    }
    Ok(())
}

fn pop_end_marker(x: &[char]) -> Vec<char> {
    let mut x = x.to_vec();
    if let Some(&c) = x.last() {
        if c == END_MARKER {
            x.pop();
        }
    }
    x
}

const fn get_block_len(alphabet_size: u32) -> u32 {
    let max_code = alphabet_size - 1;
    let mut shift = 1;
    while (max_code >> shift) != 0 {
        shift += 1;
    }
    1 << shift
}
