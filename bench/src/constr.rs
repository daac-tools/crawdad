use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "constr", about = "A program to measure.")]
struct Args {
    #[clap(short = 'k', long)]
    keys_filename: String,
}

fn main() {
    let args = Args::parse();

    println!("keys_filename\t{}", &args.keys_filename);
    let mut keys = load_file(&args.keys_filename);
    keys.sort_unstable();
    show_memory_stats(&keys);
}

fn show_memory_stats(keys: &[String]) {
    {
        println!("[crawdad/trie/nomap]");
        let start = Instant::now();
        let trie = crawdad::builder::nomap::Builder::new()
            .from_keys(keys)
            .release_trie();
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
    {
        println!("[crawdad/trie/freqmap]");
        let start = Instant::now();
        let trie = crawdad::builder::freqmap::Builder::new()
            .from_keys(keys)
            .release_trie();
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
    for t in 1..=3 {
        println!("[crawdad/rhtrie/{}/nomap]", t);
        let start = Instant::now();
        let trie = crawdad::builder::nomap::Builder::new()
            .set_suffix_thr(t)
            .from_keys(keys)
            .release_rhtrie(3);
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("vacant_ratio: {:.3}", trie.vacant_ratio());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
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
        print_memory("heap_bytes", data.len());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
}

fn print_memory(title: &str, bytes: usize) {
    println!(
        "{}: {} bytes, {:.3} MiB",
        title,
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
