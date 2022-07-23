use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use clap::Parser;

const TRIALS: usize = 10;
const QUERIES: usize = 1000;

#[derive(Parser, Debug)]
#[clap(name = "bench", about = "A program to measure the performance.")]
struct Args {
    #[clap(short = 'k', long)]
    keys_filename: String,

    #[clap(short = 't', long)]
    texts_filename: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("keys_filename: {}", &args.keys_filename);
    let keys = {
        let mut keys = load_file(&args.keys_filename);
        keys.sort_unstable();
        keys
    };
    let queries = random_sample(&keys);
    let texts = args.texts_filename.map(|texts_filename| {
        println!("texts_filename: {}", &texts_filename);
        load_file(&texts_filename)
    });

    println!("#keys: {}", keys.len());

    {
        println!("[crawdad/trie]");
        let start = Instant::now();
        let trie = crawdad::Trie::from_keys(&keys).unwrap();
        let duration = start.elapsed();
        print_heap_bytes(trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("num_vacants: {}", trie.num_vacants());
        println!(
            "vacant_ratio: {:.3}",
            trie.num_vacants() as f64 / trie.num_elems() as f64
        );
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for query in &queries {
                    dummy += trie.exact_match(query.chars()).unwrap();
                }
            });
            println!(
                "exact_match: {:.3} [ns/query]",
                to_ns(elapsed_sec) / queries.len() as f64
            );
            println!("dummy: {}", dummy);
        }

        if let Some(texts) = texts.as_ref() {
            // Warmup
            let mut dummy = 0;
            let mut haystack = vec![];
            let elapsed_sec = measure(TRIALS, || {
                for text in texts {
                    haystack.clear();
                    text.chars().for_each(|c| haystack.push(c));
                    for i in 0..haystack.len() {
                        for (v, j) in trie.common_prefix_search(haystack[i..].iter().cloned()) {
                            dummy += j + v as usize;
                        }
                    }
                }
            });
            println!(
                "enumeration: {:.3} [us/text]",
                to_us(elapsed_sec) / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[crawdad/mptrie]");
        let start = Instant::now();
        let trie = crawdad::MpTrie::from_keys(&keys).unwrap();
        let duration = start.elapsed();
        print_heap_bytes(trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("num_vacants: {}", trie.num_vacants());
        println!(
            "vacant_ratio: {:.3}",
            trie.num_vacants() as f64 / trie.num_elems() as f64
        );
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for query in &queries {
                    dummy += trie.exact_match(query.chars()).unwrap();
                }
            });
            println!(
                "exact_match: {:.3} [ns/query]",
                to_ns(elapsed_sec) / queries.len() as f64
            );
            println!("dummy: {}", dummy);
        }

        if let Some(texts) = texts.as_ref() {
            // Warmup
            let mut dummy = 0;
            let mut haystack = vec![];
            let elapsed_sec = measure(TRIALS, || {
                for text in texts {
                    haystack.clear();
                    text.chars().for_each(|c| haystack.push(c));
                    for i in 0..haystack.len() {
                        for (v, j) in trie.common_prefix_search(haystack[i..].iter().cloned()) {
                            dummy += j + v as usize;
                        }
                    }
                }
            });
            println!(
                "enumeration: {:.3} [us/text]",
                to_us(elapsed_sec) / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[std/BTreeMap]");
        let start = Instant::now();
        let mut map = std::collections::BTreeMap::new();
        for (i, key) in keys.iter().enumerate() {
            map.insert(key, i as u32);
        }
        let duration = start.elapsed();
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for query in &queries {
                    dummy += map.get(query).unwrap();
                }
            });
            println!(
                "exact_match: {:.3} [ns/query]",
                to_ns(elapsed_sec) / queries.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[std/HashMap]");
        let start = Instant::now();
        let mut map = std::collections::HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            map.insert(key, i as u32);
        }
        let duration = start.elapsed();
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for query in &queries {
                    dummy += map.get(query).unwrap();
                }
            });
            println!(
                "exact_match: {:.3} [ns/query]",
                to_ns(elapsed_sec) / queries.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[yada]");
        let start = Instant::now();
        let data = yada::builder::DoubleArrayBuilder::build(
            &keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, u32::try_from(i).unwrap()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let duration = start.elapsed();
        print_heap_bytes(data.len());
        println!("num_elems: {}", data.len() / 4);
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        let da = yada::DoubleArray::new(data);
        {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for query in &queries {
                    dummy += da.exact_match_search(query).unwrap();
                }
            });
            println!(
                "exact_match: {:.3} [ns/query]",
                to_ns(elapsed_sec) / queries.len() as f64
            );
            println!("dummy: {}", dummy);
        }

        if let Some(texts) = texts.as_ref() {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for text in texts {
                    let text_bytes = text.as_bytes();
                    for i in 0..text_bytes.len() {
                        for (id, length) in da.common_prefix_search(&text_bytes[i..]) {
                            dummy += i + length + id as usize;
                        }
                    }
                }
            });
            println!(
                "enumeration: {:.3} [us/text]",
                to_us(elapsed_sec) / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[fst/map]");
        let start = Instant::now();
        let map = fst::raw::Fst::from_iter_map(
            keys.iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, i.try_into().unwrap())),
        )
        .unwrap();
        let duration = start.elapsed();
        print_heap_bytes(map.as_bytes().len());
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for query in &queries {
                    dummy += map.get(query).unwrap().value() as u32;
                }
            });
            println!(
                "exact_match: {:.3} [ns/query]",
                to_ns(elapsed_sec) / queries.len() as f64
            );
            println!("dummy: {}", dummy);
        }

        if let Some(texts) = texts.as_ref() {
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for text in texts {
                    let text_bytes = text.as_bytes();
                    for i in 0..text_bytes.len() {
                        for (id, length) in fst_common_prefix_search(&map, &text_bytes[i..]) {
                            dummy += i + length as usize + id as usize;
                        }
                    }
                }
            });
            println!(
                "enumeration: {:.3} [us/text]",
                to_us(elapsed_sec) / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[daachorse/bytewise]");
        let start = Instant::now();
        let pma = daachorse::DoubleArrayAhoCorasick::new(&keys).unwrap();
        let duration = start.elapsed();
        print_heap_bytes(pma.heap_bytes());
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        if let Some(texts) = texts.as_ref() {
            // Warmup
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for text in texts {
                    for m in pma.find_overlapping_iter(text) {
                        dummy += m.end() + m.value() as usize;
                    }
                }
            });
            println!(
                "enumeration: {:.3} [us/text]",
                to_us(elapsed_sec) / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }

    {
        println!("[daachorse/charwise]");
        let start = Instant::now();
        let pma = daachorse::charwise::CharwiseDoubleArrayAhoCorasick::new(&keys).unwrap();
        let duration = start.elapsed();
        print_heap_bytes(pma.heap_bytes());
        println!("construction: {:.3} [sec]", duration.as_secs_f64());

        if let Some(texts) = texts.as_ref() {
            // Warmup
            let mut dummy = 0;
            let elapsed_sec = measure(TRIALS, || {
                for text in texts {
                    for m in pma.find_overlapping_iter(text) {
                        dummy += m.end() + m.value() as usize;
                    }
                }
            });
            println!(
                "enumeration: {:.3} [us/text]",
                to_us(elapsed_sec) / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    }
}

fn print_heap_bytes(bytes: usize) {
    println!(
        "heap_bytes: {} bytes, {:.3} MiB",
        bytes,
        bytes as f64 / (1024.0 * 1024.0)
    );
}

fn load_file<P>(path: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(path).unwrap();
    let buf = BufReader::new(file);
    buf.lines().map(|line| line.unwrap()).collect()
}

fn random_sample(keys: &[String]) -> Vec<String> {
    let mut rng = rand::thread_rng();
    rand::seq::sample_slice(&mut rng, keys, QUERIES)
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

fn measure<F>(num_trials: usize, mut func: F) -> f64
where
    F: FnMut(),
{
    // Warmup
    func();
    // Measure
    let start = Instant::now();
    for _ in 0..num_trials {
        func();
    }
    let duration = start.elapsed();
    duration.as_secs_f64() / num_trials as f64
}

#[allow(dead_code)]
fn to_ms(sec: f64) -> f64 {
    sec * 1_000.
}

#[allow(dead_code)]
fn to_us(sec: f64) -> f64 {
    sec * 1_000_000.
}

#[allow(dead_code)]
fn to_ns(sec: f64) -> f64 {
    sec * 1_000_000_000.
}
