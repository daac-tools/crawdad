# 🦞 Crawdad: ChaRActer-Wise Double-Array Dictionary

[![Crates.io](https://img.shields.io/crates/v/crawdad)](https://crates.io/crates/crawdad)
[![Documentation](https://docs.rs/crawdad/badge.svg)](https://docs.rs/crawdad)
![Build Status](https://github.com/daac-tools/crawdad/actions/workflows/rust.yml/badge.svg)

## Overview

Crawdad is a library of natural language dictionaries using character-wise double-array tries.
The implementation is optimized for strings of multibyte-characters,
and you can enjoy fast text processing on strings such as Japanese or Chinese.

Experimental results can be found in [Wiki](https://github.com/daac-tools/crawdad/wiki/Performance-Comparison).

### What can do

- **Key-value mapping**: Crawdad stores a set of string keys with mapping arbitrary integer values.
- **Exact match**: Crawdad supports a fast lookup for an input key.
- **Common prefix search**: Crawdad supports fast *common prefix search* that can be used to enumerate all keys appearing in a text.

### Data structures

Crawdad contains the two trie implementations:

- `crawdad::Trie` is a standard trie form that often provides the fastest queries.
- `crawdad::MpTrie` is a minimal-prefix trie form that is memory-efficient for long strings. 

## Installation

To use `crawdad`, depend on it in your Cargo manifest:

```toml
# Cargo.toml

[dependencies]
crawdad = "0.1"
```

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

For softwares under `bench/data`, follow the license terms of each software.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
