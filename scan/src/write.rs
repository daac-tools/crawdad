use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "write", about = "A program to write dictionaries.")]
struct Args {
    #[clap(short = 'i', long)]
    keys_path: PathBuf,

    #[clap(short = 'o', long)]
    dict_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let keys_path = args.keys_path;
    let dict_path = args.dict_path;

    let keys: Vec<_> = BufReader::new(File::open(keys_path)?)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    {
        let mut out_path = dict_path.clone();
        out_path.set_extension("crawdad");

        let trie = crawdad::Trie::from_keys(&keys).unwrap();
        let bytes = trie.serialize_to_vec();
        let mut writer = File::create(out_path)?;
        writer.write_all(&bytes)?;
    }

    {
        let mut out_path = dict_path.clone();
        out_path.set_extension("yada");

        let bytes = yada::builder::DoubleArrayBuilder::build(
            &keys
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, key)| (key, u32::try_from(i).unwrap()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let mut writer = File::create(out_path)?;
        writer.write_all(&bytes)?;
    }

    Ok(())
}
