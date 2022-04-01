use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use crawdad::Statistics;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "bench", about = "A program to measure the performance.")]
struct Args {
    #[clap(short = 'k', long)]
    keys_filename: String,

    #[clap(short = 't', long)]
    texts_filename: Option<String>,
}

macro_rules! crawdad_common {
    ($trie:ident, $keys:ident, $texts:ident) => {
        let start = Instant::now();
        let trie = crawdad::$trie::from_keys(&$keys).unwrap();
        let duration = start.elapsed();
        print_heap_bytes(trie.heap_bytes());
        println!("num_elems: {}", trie.num_elems());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());

        if let Some(texts) = $texts.as_ref() {
            let start = Instant::now();
            let mut dummy = 0;
            let mut mapped = Vec::with_capacity(256);
            for _ in 0..10 {
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
            println!("search_sec: {:.3}", duration.as_secs_f64() / 10.);
            println!("dummy: {}", dummy);
        }
    };
}

fn main() {
    let args = Args::parse();

    println!("keys_filename: {}", &args.keys_filename);
    let keys = load_file(&args.keys_filename);
    let texts = if let Some(texts_filename) = args.texts_filename {
        println!("texts_filename: {}", &texts_filename);
        Some(load_file(&texts_filename))
    } else {
        None
    };

    println!("#keys: {}", keys.len());
    {
        println!("[crawdad/trie]");
        crawdad_common!(Trie, keys, texts);
    }
    {
        println!("[crawdad/mptrie]");
        crawdad_common!(MpTrie, keys, texts);
    }
    {
        println!("[crawdad/fmptrie]");
        crawdad_common!(FmpTrie, keys, texts);
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
        println!("constr_sec: {:.3}", duration.as_secs_f64());

        if let Some(texts) = texts.as_ref() {
            let start = Instant::now();
            let mut dummy = 0;
            let da = yada::DoubleArray::new(data);
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
            println!("search_sec: {:.3}", duration.as_secs_f64() / 10.);
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
