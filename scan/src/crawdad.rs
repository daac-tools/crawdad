use std::error::Error;
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::PathBuf;
use std::vec;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "crawdad", about = "A program to run crawdad.")]
struct Args {
    #[clap(short = 'i', long)]
    dict_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let dict_path = args.dict_path;

    let mut bytes = vec![];
    {
        let mut reader = File::open(dict_path)?;
        reader.read_to_end(&mut bytes)?;
    }
    let (trie, _) = crawdad::Trie::deserialize_from_slice(&bytes);

    let mut dummy = 0;
    let mut haystack = vec![];

    let lines = std::io::stdin().lock().lines();
    for line in lines {
        let line = line?;
        haystack.clear();
        haystack.extend(line.chars());
        for i in 0..haystack.len() {
            for (v, j) in trie.common_prefix_search(haystack[i..].iter().copied()) {
                dummy += j + v as usize;
            }
        }
    }
    dbg!(dummy);

    Ok(())
}
