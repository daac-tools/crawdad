use super::Node;
use crate::END_CODE;

pub struct Trie {
    pub(crate) nodes: Vec<Node>,
    pub(crate) max_code: i32,
}

impl Trie {
    pub fn exact_match<K>(&self, key: K) -> Option<u32>
    where
        K: AsRef<str>,
    {
        let mut idx = 0;
        for c in key.as_ref().chars() {
            if let Some(child_id) = self.get_child_id(idx, c as u32) {
                idx = child_id;
            } else {
                return None;
            }
        }
        if self.nodes[idx as usize].is_leaf() {
            Some(self.nodes[idx as usize].get_base())
        } else if self.nodes[idx as usize].has_leaf() {
            let leaf_id = self.get_base(idx);
            debug_assert_eq!(self.nodes[leaf_id as usize].get_check(), idx);
            Some(self.nodes[leaf_id as usize].get_base())
        } else {
            None
        }
    }

    pub fn heap_bytes(&self) -> usize {
        self.nodes.len() * std::mem::size_of::<Node>()
    }

    #[inline(always)]
    fn get_child_id(&self, idx: u32, c: u32) -> Option<u32> {
        if self.nodes[idx as usize].is_leaf() {
            return None;
        }
        let child_idx = (self.get_base(idx) + c as i32) as u32;
        if let Some(check) = self.get_check(child_idx) {
            if check == idx {
                return Some(child_idx);
            }
        }
        None
    }

    #[inline(always)]
    fn get_base(&self, idx: u32) -> i32 {
        self.nodes[idx as usize].get_base() as i32 - self.max_code
    }

    #[inline(always)]
    fn get_check(&self, idx: u32) -> Option<u32> {
        if let Some(node) = self.nodes.get(idx as usize) {
            Some(node.get_check())
        } else {
            None
        }
    }

    #[inline(always)]
    fn is_leaf(&self, idx: u32) -> bool {
        self.nodes[idx as usize].is_leaf()
    }

    #[inline(always)]
    fn has_leaf(&self, idx: u32) -> bool {
        self.nodes[idx as usize].has_leaf()
    }

    #[inline(always)]
    fn get_leaf(&self, idx: u32) -> u32 {
        let leaf_id = self.nodes[idx as usize].get_base() ^ END_CODE;
        debug_assert_eq!(self.nodes[leaf_id as usize].get_check(), idx);
        leaf_id
    }

    #[inline(always)]
    fn get_value(&self, idx: u32) -> u32 {
        debug_assert!(self.is_leaf(idx));
        self.nodes[idx as usize].get_base()
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::plus::Builder;

    #[test]
    fn test_exact_match() {
        let keys = vec!["世界", "世界中", "世直し", "国民"];
        let trie = Builder::new().from_keys(&keys);
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(trie.exact_match(&key), Some(i as u32));
        }
    }
}
