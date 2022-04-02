use crate::MappedChar;

use std::cmp::Ordering;

/// pack_size returns the smallest number of bytes that can encode `n`.
#[inline(always)]
pub const fn pack_size(n: u32) -> u8 {
    if n < 1 << 8 {
        1
    } else if n < 1 << 16 {
        2
    } else if n < 1 << 24 {
        3
    } else {
        4
    }
}

#[inline(always)]
pub fn pack_u32(vec: &mut Vec<u8>, mut n: u32, nbytes: u8) {
    debug_assert!((1..=4).contains(&nbytes));

    for _ in 0..nbytes {
        vec.push(n as u8);
        n >>= 8;
    }
}

#[inline(always)]
pub fn unpack_u32(slice: &[u8], nbytes: u8) -> u32 {
    debug_assert!((1..=4).contains(&nbytes));

    let mut n = 0;
    for (i, &b) in slice[..nbytes as usize].iter().enumerate() {
        n |= (b as u32) << (8 * i);
    }
    n
}

// https://github.com/aappleby/smhasher/blob/master/src/MurmurHash2.cpp
#[inline(always)]
pub fn murmur_hash2(key: &[MappedChar]) -> Option<u32> {
    let seed = 0xbc9f1d34;

    // 'm' and 'r' are mixing constants generated offline.
    // They're not really 'magic', they just happen to work well.
    let m = 0x5bd1e995;
    let r = 24;

    // Initialize the hash to a 'random' value
    let mut h = seed ^ key.len() as u32;

    // Mix 4 bytes at a time into the hash
    for k in key {
        if let Some(mut c) = k.c {
            c = c.wrapping_mul(m);
            c ^= c >> r;
            c = c.wrapping_mul(m);

            h = h.wrapping_mul(m);
            h ^= c;
        } else {
            return None;
        }
    }

    // Do a few final mixes of the hash to ensure the last few
    // bytes are well-incorporated.
    h ^= h >> 13;
    h = h.wrapping_mul(m);
    h ^= h >> 15;

    Some(h)
}

/// Returns (lcp, cmp) such that
///  - lcp: Length of longest commom prefix of two strings.
///  - cmp: if a < b then positive, elif b < a then negative, else zero.
#[inline(always)]
pub fn longest_common_prefix(a: &[char], b: &[char]) -> (usize, Ordering) {
    let min_len = a.len().min(b.len());
    for i in 0..min_len {
        if a[i] != b[i] {
            return (i, a[i].cmp(&b[i]));
        }
    }
    (min_len, a.len().cmp(&b.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_common_prefix() {
        assert_eq!(
            longest_common_prefix(&['a', 'b'], &['a', 'b', 'c']),
            (2, Ordering::Less)
        );
        assert_eq!(
            longest_common_prefix(&['a', 'b'], &['a', 'b']),
            (2, Ordering::Equal)
        );
        assert_eq!(
            longest_common_prefix(&['a', 'b', 'c'], &['a', 'b']),
            (2, Ordering::Greater)
        );
    }
}
