use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

fn main() {
    {
        println!("== data/unidic/unidic ==");
        let mut keys = load_file("data/unidic/unidic");
        keys.sort_unstable();
        show_memory_stats(&keys);
    }
    {
        println!("== data/ipadic.txt ==");
        let mut keys = load_file("data/ipadic.txt");
        keys.sort_unstable();
        show_memory_stats(&keys);
    }
}

fn show_memory_stats(keys: &[String]) {
    {
        println!("[crawdad::nomap]");
        let start = Instant::now();
        let trie = crawdad::builder::nomap::Builder::new().from_keys(keys);
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
        println!("constr_sec: {:.3}", duration.as_secs_f64());
    }
    {
        println!("[crawdad::freqmap]");
        let start = Instant::now();
        let trie = crawdad::builder::freqmap::Builder::new().from_keys(keys);
        let duration = start.elapsed();
        print_memory("heap_bytes", trie.heap_bytes());
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
