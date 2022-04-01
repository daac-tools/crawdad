# 🦞 Crawdad: ChaRActer-Wise Double-Array Dictionary

Crawdad is a library of natural language dictionaries using character-wise double-array tries.
The implementation is optimized to CJK strings.

## What can do

- **Key-value mapping**: Crawdad stores a set of string keys with mapping arbitrary integer values.
- **Exact match**: Crawdad supports fast query to look up a given key.
- **Common prefix search**: Crawdad supports fast *common prefix search* that can be used to enumerate all keys appearing in a text.

## Data structures

Crawdad provides the three implementations:

- `crawdad::Trie` is a standard trie form.
- `crawdad::MpTrie` is the minimal-prefix trie.
- `crawdad::FmpTrie` is the minimal-prefix trie.

## Disclaimer

This software is developed by LegalForce, Inc.,
but not an officially supported LegalForce product.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
