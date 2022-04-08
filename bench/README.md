# crawdad/bench

## Tools

Here provides benchmark tools to compare the performance of the data structures:

- `crawdad::Trie`
- `crawdad::MpTrie`
- [`yada::DoubleArray`](https://docs.rs/yada/latest/yada/struct.DoubleArray.html)
- [`fst::Map`](https://docs.rs/fst/latest/fst/struct.Map.html)
- [`daachorse::DoubleArrayAhoCorasick`](https://docs.rs/daachorse/latest/daachorse/struct.DoubleArrayAhoCorasick.html)
- [`daachorse::charwise::CharwiseDoubleArrayAhoCorasick`](https://docs.rs/daachorse/latest/daachorse/charwise/struct.CharwiseDoubleArrayAhoCorasick.html)

You can measure search time and memory usage with your datasets in the following command.

```
$ cargo run --release --bin measure -- -k data/unidic/unidic -t data/wagahaiwa_nekodearu.txt
```

Or, you can measure search time more accurately with [`criterion.rs`](https://github.com/bheisler/criterion.rs) in the following command.

```
$ cargo bench
```

## Datasets

The datasets contained here are copied from third party repositories.

- `unidic`: [National Institute for Japanese Language and Linguistics](https://ccd.ninjal.ac.jp/unidic/)
- `wagahaiwa_nekodearu.txt`: [Aozora Bunko](https://www.aozora.gr.jp/cards/000148/card789.html)
