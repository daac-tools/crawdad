use super::{make_prefix_free, Record};
use crate::trie::{nomap::Trie, Node};

use crate::{END_MARKER, INVALID_IDX, OFFSET_MASK};

#[derive(Default)]
pub struct Builder {
    records: Vec<Record>,
    nodes: Vec<Node>,
    labels: Vec<i32>,
    max_code: i32,
    head_idx: u32,
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

        self.max_code = {
            let mut max_code = 0;
            for rec in &self.records {
                for &c in &rec.key {
                    assert_ne!(c, END_MARKER);
                    max_code = max_code.max(c);
                }
            }
            max_code as i32
        };

        make_prefix_free(&mut self.records);

        self.init_array();
        self.arrange_nodes(0, self.records.len(), 0, 0);
        self.release();

        Trie {
            nodes: self.nodes,
            max_code: self.max_code,
        }
    }

    #[inline(always)]
    pub fn num_nodes(&self) -> u32 {
        self.nodes.len() as u32
    }

    fn init_array(&mut self) {
        let max_idx = self.max_code as u32;

        self.nodes.clear();
        self.nodes.resize(max_idx as usize + 1, Node::default());

        for i in 0..=max_idx {
            if i == 0 {
                self.set_prev(i, max_idx);
            } else {
                self.set_prev(i, i - 1);
            }
            if i == max_idx {
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

        self.fetch_labels(spos, epos, depth);
        let base = self.define_nodes(idx);

        let mut i1 = spos;
        let mut c1 = self.records[i1].key[depth];
        for i2 in spos + 1..epos {
            let c2 = self.records[i2].key[depth];
            if c1 != c2 {
                assert!(c1 < c2);
                let child_idx = base + c1 as i32;
                self.arrange_nodes(i1, i2, depth + 1, child_idx as u32);
                i1 = i2;
                c1 = c2;
            }
        }
        let child_idx = base + c1 as i32;
        self.arrange_nodes(i1, epos, depth + 1, child_idx as u32);
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
            let em_idx = self.nodes[idx].base as i32 - self.max_code;
            if 0 <= em_idx && em_idx < self.num_nodes() as i32 {
                if self.nodes[em_idx as usize].check as usize == idx {
                    // Sets HasLeaf = True
                    self.nodes[idx].check |= !OFFSET_MASK;
                }
            }
        }
    }

    fn fetch_labels(&mut self, spos: usize, epos: usize, depth: usize) {
        self.labels.clear();
        let mut c1 = self.records[spos].key[depth];
        for i in spos + 1..epos {
            let c2 = self.records[i].key[depth];
            if c1 != c2 {
                assert!(c1 < c2);
                self.labels.push(c1 as i32);
                c1 = c2;
            }
        }
        self.labels.push(c1 as i32);
    }

    fn define_nodes(&mut self, idx: u32) -> i32 {
        let base = self.find_base(&self.labels);
        let max_idx = (base + self.labels.last().unwrap()) as u32;

        if self.num_nodes() <= max_idx {
            self.enlarge(max_idx);
        }

        self.nodes[idx as usize].base = (base + self.max_code) as u32;
        for i in 0..self.labels.len() {
            let child_idx = base + self.labels[i];
            self.fix_node(child_idx as u32);
            self.nodes[child_idx as usize].check = idx;
        }
        base
    }

    /// Finds a valid BASE value in a simple manner.
    fn find_base(&self, codes: &[i32]) -> i32 {
        assert!(!codes.is_empty());

        let min_code = codes[0];
        if self.head_idx == INVALID_IDX {
            return self.num_nodes() as i32 - min_code;
        }

        let mut node_idx = self.head_idx;
        loop {
            debug_assert!(!self.is_fixed(node_idx));
            let base = node_idx as i32 - min_code;
            if self.verify_base(base, codes) {
                return base;
            }
            node_idx = self.get_next(node_idx);
            if node_idx == self.head_idx {
                break;
            }
        }
        self.num_nodes() as i32 - min_code
    }

    #[inline(always)]
    fn verify_base(&self, base: i32, codes: &[i32]) -> bool {
        for &code in codes {
            let idx = (base + code) as u32;
            if self.num_nodes() <= idx {
                return true;
            }
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

    fn enlarge(&mut self, max_idx: u32) {
        while self.num_nodes() <= max_idx {
            self.push_node();
        }
    }

    #[inline(always)]
    fn push_node(&mut self) {
        if self.head_idx == INVALID_IDX {
            let new_idx = self.num_nodes() as u32;
            self.nodes.push(Node::default());
            self.set_next(new_idx, new_idx);
            self.set_prev(new_idx, new_idx);
        } else {
            let head_idx = self.head_idx;
            let tail_idx = self.get_prev(head_idx);
            let new_idx = self.num_nodes() as u32;
            self.nodes.push(Node::default());
            self.set_next(new_idx, head_idx);
            self.set_prev(head_idx, new_idx);
            self.set_prev(new_idx, tail_idx);
            self.set_next(tail_idx, new_idx);
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
