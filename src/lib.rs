pub mod builder;
mod mapper;

use mapper::CodeMapper;

pub const OFFSET_MASK: u32 = 0x7fff_ffff;
pub const INVALID_IDX: u32 = 0xffff_ffff;
pub const END_MARKER: u32 = 0;
pub const END_CODE: u32 = 0;

pub struct Trie {
    nodes: Vec<Node>,
    mapper: CodeMapper,
}

impl Trie {
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut node_id = 0;
        for c in key.as_ref().chars() {
            if let Some(child_id) = self.get_child_id(node_id, c) {
                node_id = child_id;
            } else {
                return None;
            }
        }
        if self.nodes[node_id as usize].is_leaf() {
            Some(self.nodes[node_id as usize].get_base())
        } else if self.nodes[node_id as usize].has_leaf() {
            let leaf_id = self.nodes[node_id as usize].get_base() ^ END_CODE;
            debug_assert_eq!(self.nodes[leaf_id as usize].get_check(), node_id);
            Some(self.nodes[leaf_id as usize].get_base())
        } else {
            None
        }
    }

    #[inline(always)]
    fn get_child_id(&self, node_id: u32, c: char) -> Option<u32> {
        if self.nodes[node_id as usize].is_leaf() {
            return None;
        }
        if let Some(mc) = self.mapper.get(c as u32) {
            let child_id = self.nodes[node_id as usize].get_base() ^ mc;
            if self.nodes[child_id as usize].get_check() == node_id {
                return Some(child_id);
            }
        }
        None
    }
}

#[derive(Default, Clone)]
pub struct Node {
    base: u32,
    check: u32,
}

impl Node {
    #[inline(always)]
    pub const fn get_base(&self) -> u32 {
        self.base & OFFSET_MASK
    }

    #[inline(always)]
    pub const fn get_check(&self) -> u32 {
        self.check & OFFSET_MASK
    }

    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        self.base & !OFFSET_MASK != 0
    }

    #[inline(always)]
    pub const fn has_leaf(&self) -> bool {
        self.check & !OFFSET_MASK != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = builder::Builder::new().from_keys(&keys);
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(trie.exact_match(&key), Some(i as u32));
        }
    }
}
