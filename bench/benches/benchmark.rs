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

fn criterion_unidic_cps(c: &mut Criterion) {
    let mut group = c.benchmark_group("unidic/cps");
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(WARM_UP_TIME);
    group.measurement_time(MEASURE_TIME);
    group.sampling_mode(SamplingMode::Flat);
    let mut keys = load_file("data/unidic/unidic");
    keys.sort_unstable();
    let texts = load_file("data/wagahaiwa_nekodearu.txt");

    add_find_overlapping_benches(&mut group, &keys, &texts);
}

fn add_find_overlapping_benches(
    group: &mut BenchmarkGroup<WallTime>,
    keys: &[String],
    texts: &[String],
) {
    group.bench_function("crawdad", |b| {
        let trie = crawdad::builder_xor::Builder::new().from_keys(keys);
        let mut mapped = Vec::with_capacity(256);
        b.iter(|| {
            let mut sum = 0;
            for text in texts {
                trie.map_text(text, &mut mapped);
                for i in 0..mapped.len() {
                    for (val, len) in trie.common_prefix_searcher(&mapped[i..]) {
                        sum += i + len + val as usize;
                    }
                }
            }
            if sum == 0 {
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
            let mut sum = 0;
            for text in texts {
                let text_bytes = text.as_bytes();
                for i in 0..text_bytes.len() {
                    for (id, length) in da.common_prefix_search(&text_bytes[i..]) {
                        sum += i + length + id as usize;
                    }
                }
            }
            if sum == 0 {
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

criterion_group!(benches, criterion_unidic_cps);
criterion_main!(benches);
