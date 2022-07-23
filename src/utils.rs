use core::cmp::Ordering;

/// Returns `(lcp, ord)` such that
///  - lcp: Length of longest commom prefix of `a` and `b`.
///  - ord: `Ordering` between `a` and `b`.
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
