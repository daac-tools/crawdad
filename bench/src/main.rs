use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use crawdad::Statistics;

use clap::Parser;

const TRIALS: usize = 10;
const SAMPLES: usize = 1000;

#[derive(Parser, Debug)]
#[clap(name = "bench", about = "A program to measure the performance.")]
struct Args {
    #[clap(short = 'k', long)]
    keys_filename: String,

    #[clap(short = 't', long)]
    texts_filename: Option<String>,
}

macro_rules! crawdad_common {
    ($trie:ident, $keys:ident, $queries:ident, $texts:ident) => {
        let start = Instant::now();
        let trie = crawdad::$trie::from_keys(&$keys).unwrap();
        let duration = start.elapsed();
        print_heap_bytes(trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());

        {
            // Warmup
            let mut dummy = 0;
            for q in &$queries {
                dummy += trie.exact_match(q).unwrap();
            }
            // Measure
            let start = Instant::now();
            for _ in 0..TRIALS {
                for q in &$queries {
                    dummy += trie.exact_match(q).unwrap();
                }
            }
            let duration = start.elapsed();
            println!(
                "exact_match: {:.3} [us/query]",
                duration.as_secs_f64() * 1000000. / TRIALS as f64 / SAMPLES as f64
            );
            println!("dummy: {}", dummy);
        }

        if let Some(texts) = $texts.as_ref() {
            // Warmup
            let mut dummy = 0;
            let mut mapped = Vec::with_capacity(256);
            for text in texts {
                trie.map_text(text, &mut mapped);
                for i in 0..mapped.len() {
                    for m in trie.common_prefix_searcher(&mapped[i..]) {
                        dummy += i + m.end() + m.value() as usize;
                    }
                }
            }
            // Measure
            let start = Instant::now();
            for _ in 0..TRIALS {
                for text in texts {
                    trie.map_text(text, &mut mapped);
                    for i in 0..mapped.len() {
                        for m in trie.common_prefix_searcher(&mapped[i..]) {
                            dummy += i + m.end() + m.value() as usize;
                        }
                    }
                }
            }
            let duration = start.elapsed();
            println!(
                "common_prefix_search: {:.3} [us/text]",
                duration.as_secs_f64() * 1000000. / TRIALS as f64 / texts.len() as f64
            );
            println!("dummy: {}", dummy);
        }
    };
}

fn main() {
    let args = Args::parse();

    println!("keys_filename: {}", &args.keys_filename);
    let keys = load_file(&args.keys_filename);
    let queries = random_sample(&keys);
    let texts = if let Some(texts_filename) = args.texts_filename {
        println!("texts_filename: {}", &texts_filename);
        Some(load_file(&texts_filename))
    } else {
        None
    };

    println!("#keys: {}", keys.len());
    {
        println!("[crawdad/trie]");
        crawdad_common!(Trie, keys, queries, texts);
    }
    {
        println!("[crawdad/mptrie]");
        crawdad_common!(MpTrie, keys, queries, texts);
    }
    {
        println!("[crawdad/fmptrie]");
        crawdad_common!(FmpTrie, keys, queries, texts);
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
        println!("constr_sec: {:.3}", duration.as_secs_f64());

        let da = yada::DoubleArray::new(data);
        {
            // Warmup
            let mut dummy = 0;
            for q in &queries {
                dummy += da.exact_match_search(q).unwrap();
            }
            // Measure
            let start = Instant::now();
            for _ in 0..TRIALS {
                for q in &queries {
                    dummy += da.exact_match_search(q).unwrap();
                }
            }
            let duration = start.elapsed();
            println!(
                "exact_match: {:.3} [us/query]",
                duration.as_secs_f64() * 1000000. / TRIALS as f64 / SAMPLES as f64
            );
            println!("dummy: {}", dummy);
        }

        if let Some(texts) = texts.as_ref() {
            // Warmup
            let mut dummy = 0;
            for text in texts {
                let text_bytes = text.as_bytes();
                for i in 0..text_bytes.len() {
                    for (id, length) in da.common_prefix_search(&text_bytes[i..]) {
                        dummy += i + length + id as usize;
                    }
                }
            }
            // Measure
            let start = Instant::now();
            for _ in 0..10 {
                for text in texts {
                    let text_bytes = text.as_bytes();
                    for i in 0..text_bytes.len() {
                        for (id, length) in da.common_prefix_search(&text_bytes[i..]) {
                            dummy += i + length + id as usize;
                        }
                    }
                }
            }
            let duration = start.elapsed();
            println!(
                "common_prefix_search: {:.3} [us/text]",
                duration.as_secs_f64() * 1000000. / TRIALS as f64 / texts.len() as f64
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
    rand::seq::sample_slice(&mut rng, keys, SAMPLES)
}
