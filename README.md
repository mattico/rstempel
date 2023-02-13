# rstempel

A rust port of the [stempel Polish stemmer](http://www.getopt.org/stempel/index.html).

## Example

```toml
[dependencies]
rstempel = "0.1.0"
```

```rust
use rstempel::Stem;
let stemmer = &rstempel::rust::STEMMER;
let stemmed = stemmer.stem(word);
```

## Stemmer Implementations

Two implementations of stemmers are provided, in the `java` and `rust` modules, each enabled by the
corresponding cargo feature.

The `rust` stemmer, enabled by default, uses tables which can be stored directly as Rust code in a `static`.
This offers good performance, and simple usage, but very large tables can be difficult to compile.
The tables can be converted from a Java serialized table, see `examples/generate.rs`. The `rust_embedded_stempel`
feature embeds a ~240KiB stemming table converted from the stempel stemmer project as `rstempel::rust::STEMMER`.

The `java` stemmer can load tables in the format used by the Java `stempel` implementation. A compressed stemming
table from the stempel stemmer project is included in `src/tables/stemmer_2000.out.gz`. A much larger and more
accurate stemming table can be sourced from [pystempel](https://github.com/dzieciou/pystempel).

## License

The Rust code is ported from the stempel stemmer, which was extracted and modified from the Egothor project.
The code maintains the Egothor Software License version 1.00, which is a 4-clause BSD-style license.
See `LICENSE-EGOTHOR.txt`.

The stemming tables `src/tables/stemmer_2000.out.*` are converted from the stempel stemmer, offered under
the terms of the Apache License 2.0.

The list of test words `src/tables/polimorf_words.stemmed.tab.gz` is from the Polimorf Polish morphological dictionary.
The Polimorf dictionary is offered under the terms of a 2-clause BSD license. See `LICENSE-POLIMORF.txt`.

## Acknowledgements

This product includes software developed by the Egothor Project. http://egothor.sf.net/
