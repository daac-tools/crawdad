# crawdad/bench

## Tools

Here provides benchmark tools to compare the performance of the data structures:

- [`crawdad::Trie`](https://docs.rs/crawdad/latest/crawdad/trie/struct.Trie.html)
- [`crawdad::MpTrie`](https://docs.rs/crawdad/latest/crawdad/mptrie/struct.MpTrie.html)
- [`yada::DoubleArray`](https://docs.rs/yada/latest/yada/struct.DoubleArray.html)
- [`fst::Map`](https://docs.rs/fst/latest/fst/struct.Map.html)
- [`daachorse::DoubleArrayAhoCorasick`](https://docs.rs/daachorse/1.0.0-rc.1/daachorse/bytewise/struct.DoubleArrayAhoCorasick.html)
- [`daachorse::CharwiseDoubleArrayAhoCorasick`](https://docs.rs/daachorse/1.0.0-rc.1/daachorse/charwise/struct.CharwiseDoubleArrayAhoCorasick.html)
- [`std::collections::BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
- [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [`hashbrown::HashMap`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html)

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
