# How to evaluate cache hits

First, you extract a set of keys from a lexicon file.

```console
./lex2keys.py path/to/lex.csv > keys.txt
```

Second, you compile crawdad and yada dictionaries.

```console
cargo build --release
./target/release/write -i keys.txt -o dict
```

`dict.crawdad` and `dict.yada` will be output.

You can measure cache hits with the following commands.

```console
alias my_perf="perf stat -e cache-misses -e LLC-load-misses -e LLC-loads -e LLC-store-misses -e LLC-stores --no-big-num --repeat=100"
my_perf ./target/release/crawdad -i dict.crawdad -t path/to/haystack.txt
my_perf ./target/release/yada -i dict.yada -t path/to/haystack.txt
```
