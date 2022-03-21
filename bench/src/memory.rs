use std::convert::TryFrom;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

fn main() {
    {
        println!("== data/unidic/unidic ==");
        let mut keys = load_file("data/unidic/unidic");
        keys.sort_unstable();
        show_memory_stats(&keys);
    }
}

fn show_memory_stats(keys: &[String]) {
    {
        let trie = crawdad::builder::xor::Builder::new().from_keys(keys);
        format_memory("trie", trie.heap_bytes());
    }
    {
        let data = yada::builder::DoubleArrayBuilder::build(
            &keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, u32::try_from(i).unwrap()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        format_memory("yada", data.len());
    }
}

fn format_memory(title: &str, bytes: usize) {
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
