pub mod freqmap;
pub mod nomap;

use crate::END_MARKER;

#[derive(Default)]
struct Record {
    key: Vec<u32>,
    val: u32,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Suffix {
    key: Vec<u32>,
    val: u32,
}

fn make_prefix_free(records: &mut [Record]) {
    for i in 1..records.len() {
        if startswith(&records[i - 1].key, &records[i].key) {
            records[i - 1].key.push(END_MARKER);
        }
    }
}

fn startswith(a: &[u32], b: &[u32]) -> bool {
    if b.len() < a.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}
