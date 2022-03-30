/// pack_size returns the smallest number of bytes that can encode `n`.
#[inline(always)]
pub fn pack_size(n: u32) -> u8 {
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
    assert!(1 <= nbytes && nbytes <= 4);

    for _ in 0..nbytes {
        vec.push(n as u8);
        n = n >> 8;
    }
}

#[inline(always)]
pub fn unpack_u32(slice: &[u8], nbytes: u8) -> u32 {
    assert!(1 <= nbytes && nbytes <= 4);

    let mut n = 0;
    for (i, &b) in slice[..nbytes as usize].iter().enumerate() {
        n = n | ((b as u32) << (8 * i));
    }
    n
}

// https://github.com/aappleby/smhasher/blob/master/src/MurmurHash2.cpp
#[inline(always)]
pub fn murmur_hash2(key: &[Option<u32>]) -> Option<u32> {
    let seed = 0xbc9f1d34;

    // 'm' and 'r' are mixing constants generated offline.
    // They're not really 'magic', they just happen to work well.
    let m = 0x5bd1e995;
    let r = 24;

    // Initialize the hash to a 'random' value
    let mut h = seed ^ key.len() as u32;

    // Mix 4 bytes at a time into the hash
    for k in key {
        if let Some(mut k) = k.clone() {
            k = k.wrapping_mul(m);
            k ^= k >> r;
            k = k.wrapping_mul(m);

            h = h.wrapping_mul(m);
            h ^= k;
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
