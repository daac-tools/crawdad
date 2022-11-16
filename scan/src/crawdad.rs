use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::vec;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "crawdad", about = "A program to run crawdad.")]
struct Args {
    #[clap(short = 'i', long)]
    dict_path: PathBuf,

    #[clap(short = 't', long)]
    text_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let dict_path = args.dict_path;
    let text_path = args.text_path;

    let mut bytes = vec![];
    {
        let mut reader = File::open(dict_path)?;
        reader.read_to_end(&mut bytes)?;
    }
    let (trie, _) = crawdad::Trie::deserialize_from_slice(&bytes);

    let mut dummy = 0;
    let mut haystack = vec![];

    let reader = BufReader::new(File::open(text_path)?);
    for line in reader.lines() {
        let line = line?;
        haystack.clear();
        haystack.extend(line.chars());
        for i in 0..haystack.len() {
            for (v, _) in trie.common_prefix_search(haystack[i..].iter().copied()) {
                dummy += v as usize;
            }
        }
    }
    dbg!(dummy);

    Ok(())
}
