use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion, SamplingMode,
};

const SAMPLE_SIZE: usize = 10;
const WARM_UP_TIME: Duration = Duration::from_secs(5);
const MEASURE_TIME: Duration = Duration::from_secs(10);

fn criterion_unidic_exact(c: &mut Criterion) {
    let mut group = c.benchmark_group("unidic/exact");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(WARM_UP_TIME);
    group.measurement_time(MEASURE_TIME);
    group.sampling_mode(SamplingMode::Flat);
    let mut keys = load_file("data/unidic/unidic");
    keys.sort_unstable();
    let queries = load_file("data/unidic/unidic.1k.queries");

    add_exact_match_benches(&mut group, &keys, &queries);
}

fn criterion_unidic_enumerate(c: &mut Criterion) {
    let mut group = c.benchmark_group("unidic/cps");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(WARM_UP_TIME);
    group.measurement_time(MEASURE_TIME);
    group.sampling_mode(SamplingMode::Flat);
    let mut keys = load_file("data/unidic/unidic");
    keys.sort_unstable();
    let texts = load_file("data/wagahaiwa_nekodearu.txt");

    add_enumerate_benches(&mut group, &keys, &texts);
}

fn add_exact_match_benches(
    group: &mut BenchmarkGroup<WallTime>,
    keys: &[String],
    queries: &[String],
) {
    group.bench_function("crawdad/trie", |b| {
        let trie = crawdad::Trie::from_keys(keys).unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += trie.exact_match(query.chars()).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("crawdad/mptrie", |b| {
        let trie = crawdad::MpTrie::from_keys(keys).unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += trie.exact_match(query.chars()).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("std/BTreeMap", |b| {
        let mut map = std::collections::BTreeMap::new();
        for (i, key) in keys.iter().enumerate() {
            map.insert(key.clone(), i as u32);
        }
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += map.get(query).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("std/HashMap", |b| {
        let mut map = std::collections::HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            map.insert(key.clone(), i as u32);
        }
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += map.get(query).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("yada", |b| {
        let data = yada::builder::DoubleArrayBuilder::build(
            &keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, i as u32))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let da = yada::DoubleArray::new(data);
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += da.exact_match_search(query).unwrap();
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("fst/map", |b| {
        let map = fst::raw::Fst::from_iter_map(
            keys.iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, i.try_into().unwrap())),
        )
        .unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for query in queries {
                dummy += map.get(query).unwrap().value() as u32;
            }
            if dummy == 0 {
                panic!();
            }
        });
    });
}

fn add_enumerate_benches(group: &mut BenchmarkGroup<WallTime>, keys: &[String], texts: &[String]) {
    group.bench_function("crawdad/trie", |b| {
        let trie = crawdad::Trie::from_keys(keys).unwrap();
        let mut searcher = trie.common_prefix_searcher();
        b.iter(|| {
            let mut dummy = 0;
            for text in texts {
                searcher.update_haystack(text.chars());
                for i in 0..searcher.len_chars() {
                    for m in searcher.search(i) {
                        dummy += m.end_bytes() + m.value() as usize;
                    }
                }
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("crawdad/mptrie", |b| {
        let trie = crawdad::MpTrie::from_keys(keys).unwrap();
        let mut searcher = trie.common_prefix_searcher();
        b.iter(|| {
            let mut dummy = 0;
            for text in texts {
                searcher.update_haystack(text.chars());
                for i in 0..searcher.len_chars() {
                    for m in searcher.search(i) {
                        dummy += m.end_bytes() + m.value() as usize;
                    }
                }
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("yada", |b| {
        let data = yada::builder::DoubleArrayBuilder::build(
            &keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, i as u32))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let da = yada::DoubleArray::new(data);
        b.iter(|| {
            let mut dummy = 0;
            for text in texts {
                let text_bytes = text.as_bytes();
                for i in 0..text_bytes.len() {
                    for (id, length) in da.common_prefix_search(&text_bytes[i..]) {
                        dummy += i + length + id as usize;
                    }
                }
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("fst/map", |b| {
        let map = fst::raw::Fst::from_iter_map(
            keys.iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, i.try_into().unwrap())),
        )
        .unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for text in texts {
                let text_bytes = text.as_bytes();
                for i in 0..text_bytes.len() {
                    for (id, length) in fst_common_prefix_search(&map, &text_bytes[i..]) {
                        dummy += i + length as usize + id as usize;
                    }
                }
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("daachorse/bytewise", |b| {
        let pma = daachorse::DoubleArrayAhoCorasick::new(keys).unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for text in texts {
                for m in pma.find_overlapping_iter(text) {
                    dummy += m.end() + m.value() as usize;
                }
            }
            if dummy == 0 {
                panic!();
            }
        });
    });

    group.bench_function("daachorse/charwise", |b| {
        let pma = daachorse::charwise::CharwiseDoubleArrayAhoCorasick::new(keys).unwrap();
        b.iter(|| {
            let mut dummy = 0;
            for text in texts {
                for m in pma.find_overlapping_iter(text) {
                    dummy += m.end() + m.value() as usize;
                }
            }
            if dummy == 0 {
                panic!();
            }
        });
    });
}

fn load_file<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}

fn fst_common_prefix_search<'a>(
    fst: &'a fst::raw::Fst<Vec<u8>>,
    text: &'a [u8],
) -> impl Iterator<Item = (u64, u64)> + 'a {
    text.iter()
        .scan(
            (0, fst.root(), fst::raw::Output::zero()),
            |(pattern_len, node, output), &byte| {
                node.find_input(byte).map(|b_index| {
                    let transition = node.transition(b_index);
                    *pattern_len += 1;
                    *output = output.cat(transition.out);
                    *node = fst.node(transition.addr);
                    (node.is_final(), *pattern_len, output.value())
                })
            },
        )
        .filter_map(|(is_final, pattern_len, pattern_id)| {
            is_final.then(|| (pattern_id, pattern_len))
        })
}

criterion_group!(benches, criterion_unidic_exact, criterion_unidic_enumerate);
criterion_main!(benches);
