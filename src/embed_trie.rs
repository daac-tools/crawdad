pub mod nomap;

use crate::{Node, OFFSET_MASK};
use seahash::SeaHasher;
use std::hash::{Hash, Hasher};

const HASH_CODE_MASK: u32 = (1 << 23) - 1;

#[derive(Clone)]
pub struct EmbedSuffix {
    node: Node,
}

impl EmbedSuffix {
    pub fn new(node: Node) -> Self {
        assert!(node.is_embedded());
        Self { node }
    }

    pub fn from_suffix(suf: &[u32], val: u32) -> Node {
        assert_eq!(suf.len() & !0xFF, 0);
        let hc = hash_slice(suf);
        Node {
            base: val | !OFFSET_MASK,
            check: suf.len() as u32 | hc << 8 | !OFFSET_MASK,
        }
    }

    pub fn value(&self) -> u32 {
        self.node.get_base()
    }

    pub fn len(&self) -> usize {
        (self.node.get_check() & 0xFF) as usize
    }

    pub fn hash_code(&self) -> u32 {
        self.node.get_check() >> 8
    }
}

pub fn hash_slice(slice: &[u32]) -> u32 {
    let mut hasher = SeaHasher::new();
    slice.hash(&mut hasher);
    hasher.finish() as u32 & HASH_CODE_MASK
}
