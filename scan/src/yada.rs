use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::vec;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "yada", about = "A program to run yada.")]
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
    let trie = yada::DoubleArray::new(bytes);
    let mut dummy = 0;

    let reader = BufReader::new(File::open(text_path)?);
    for line in reader.lines() {
        let text = line?;
        let text_bytes = text.as_bytes();
        for i in 0..text_bytes.len() {
            for (id, _) in trie.common_prefix_search(&text_bytes[i..]) {
                dummy += id as usize;
            }
        }
    }
    dbg!(dummy);

    Ok(())
}
