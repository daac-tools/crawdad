use super::{make_prefix_free, Record};
use crate::mapper::CodeMapper;
use crate::trie::{xor::Trie, Node};

use crate::{END_CODE, END_MARKER, INVALID_IDX, OFFSET_MASK};

#[derive(Default)]
pub struct Builder {
    records: Vec<Record>,
    nodes: Vec<Node>,
    mapper: CodeMapper,
    labels: Vec<u32>,
    head_idx: u32,
    block_len: u32,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_keys<I, K>(mut self, keys: I) -> Trie
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        self.records = {
            let mut records = vec![];
            for key in keys {
                records.push(Record {
                    key: key.as_ref().chars().map(|c| c as u32).collect(),
                    val: records.len() as u32,
                });
            }
            records
        };

        self.mapper = CodeMapper::new(&Self::make_freqs(&self.records));
        assert_eq!(self.mapper.get(END_MARKER).unwrap(), END_CODE);

        make_prefix_free(&mut self.records);

        self.block_len = Self::get_block_len(self.mapper.alphabet_size());
        self.init_array();
        self.arrange_nodes(0, self.records.len(), 0, 0);
        self.release();

        Trie {
            nodes: self.nodes,
            mapper: self.mapper,
        }
    }

    fn make_freqs(records: &[Record]) -> Vec<u32> {
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
        assert_eq!(freqs[END_MARKER as usize], 0);
        freqs[END_MARKER as usize] += u32::MAX;
        freqs
    }

    const fn get_block_len(alphabet_size: u32) -> u32 {
        let max_code = alphabet_size - 1;
        let mut shift = 1;
        while (max_code >> shift) != 0 {
            shift += 1;
        }
        1 << shift
    }

    #[inline(always)]
    pub fn num_nodes(&self) -> u32 {
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

    fn arrange_nodes(&mut self, spos: usize, epos: usize, depth: usize, idx: u32) {
        assert!(self.is_fixed(idx));

        if self.records[spos].key.len() == depth {
            assert_eq!(spos + 1, epos);
            assert_eq!(self.records[spos].val & !OFFSET_MASK, 0);
            // Sets IsLeaf = True
            self.nodes[idx as usize].base = self.records[spos].val | !OFFSET_MASK;
            // Note: HasLeaf must not be set here and should be set in release()
            // because MSB of check is used to indicate vacant element.
            return;
        }

        // Fetches and maps labels.
        {
            self.labels.clear();
            let mut c1 = self.records[spos].key[depth];
            for i in spos + 1..epos {
                let c2 = self.records[i].key[depth];
                if c1 != c2 {
                    assert!(c1 < c2);
                    self.labels.push(self.mapper.get(c1).unwrap());
                    c1 = c2;
                }
            }
            self.labels.push(self.mapper.get(c1).unwrap());
        }

        let base = self.find_base(&self.labels);
        if base >= self.num_nodes() {
            self.enlarge();
        }

        self.nodes[idx as usize].base = base;
        for i in 0..self.labels.len() {
            let child_idx = base ^ self.labels[i];
            self.fix_node(child_idx);
            self.nodes[child_idx as usize].check = idx;
        }

        let mut i1 = spos;
        let mut c1 = self.records[i1].key[depth];
        for i2 in spos + 1..epos {
            let c2 = self.records[i2].key[depth];
            if c1 != c2 {
                assert!(c1 < c2);
                let child_idx = base ^ self.mapper.get(c1).unwrap();
                self.arrange_nodes(i1, i2, depth + 1, child_idx);
                i1 = i2;
                c1 = c2;
            }
        }
        let child_idx = base ^ self.mapper.get(c1).unwrap();
        self.arrange_nodes(i1, epos, depth + 1, child_idx);
    }

    fn release(&mut self) {
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
        for idx in 0..self.nodes.len() {
            // Empty?
            if self.nodes[idx].base == OFFSET_MASK && self.nodes[idx].check == OFFSET_MASK {
                continue;
            }
            // Leaf?
            if self.nodes[idx].base & !OFFSET_MASK != 0 {
                continue;
            }
            let em_idx = self.nodes[idx].base ^ END_CODE;
            if self.nodes[em_idx as usize].check as usize == idx {
                // Sets HasLeaf = True
                self.nodes[idx].check |= !OFFSET_MASK;
            }
        }
    }

    /// Finds a valid BASE value in a simple manner.
    fn find_base(&self, labels: &[u32]) -> u32 {
        assert!(!labels.is_empty());
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
            let idx = base ^ label;
            if self.is_fixed(idx) {
                return false;
            }
        }
        true
    }

    fn fix_node(&mut self, node_idx: u32) {
        assert!(!self.is_fixed(node_idx));

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

    fn enlarge(&mut self) {
        let old_len = self.num_nodes();
        let new_len = old_len + self.block_len;

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
    }

    // If the most significant bit is unset, the state is fixed.
    #[inline(always)]
    fn is_fixed(&self, i: u32) -> bool {
        self.nodes[i as usize].check & !OFFSET_MASK == 0
    }

    // Unset the most significant bit.
    #[inline(always)]
    fn set_fixed(&mut self, i: u32) {
        assert!(!self.is_fixed(i));
        self.nodes[i as usize].base = INVALID_IDX;
        self.nodes[i as usize].check &= OFFSET_MASK;
    }

    #[inline(always)]
    fn get_next(&self, i: u32) -> u32 {
        assert_ne!(self.nodes[i as usize].base & !OFFSET_MASK, 0, "i={}", i);
        self.nodes[i as usize].base & OFFSET_MASK
    }

    #[inline(always)]
    fn get_prev(&self, i: u32) -> u32 {
        assert_ne!(self.nodes[i as usize].check & !OFFSET_MASK, 0, "i={}", i);
        self.nodes[i as usize].check & OFFSET_MASK
    }

    #[inline(always)]
    fn set_next(&mut self, i: u32, x: u32) {
        assert_eq!(x & !OFFSET_MASK, 0);
        self.nodes[i as usize].base = x | !OFFSET_MASK
    }

    #[inline(always)]
    fn set_prev(&mut self, i: u32, x: u32) {
        assert_eq!(x & !OFFSET_MASK, 0);
        self.nodes[i as usize].check = x | !OFFSET_MASK
    }
}
