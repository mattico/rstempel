//! # rstempel
//! 
//! A rust port of the [stempel Polish stemmer](http://www.getopt.org/stempel/index.html).
//! 
//! ## Example
//! 
//! ```toml
//! [dependencies]
//! rstempel = "0.1.0"
//! ```
//! 
//! ```rust
//! # let word = "foo";
//! use rstempel::Stem;
//! let stemmer = &rstempel::rust::STEMMER;
//! let stemmed = stemmer.stem(word);
//! ```
//! 
//! ## Unicode Normalization
//! 
//! It is recommended to Unicode normalize (NFC) the input before stemming, as combining diacritical marks are not
//! handled correctly on their own. The [unicode-normalization crate](https://github.com/unicode-rs/unicode-normalization)
//! can be used for this.
//! 
//! ## Stemmer Implementations
//! 
//! Two implementations of stemmers are provided, in the `java` and `rust` modules, each enabled by the
//! corresponding cargo feature.
//! 
//! The `rust` stemmer, enabled by default, uses tables which can be stored directly as Rust code in a `static`.
//! This offers good performance, and simple usage, but very large tables can be difficult to compile.
//! The tables can be converted from a Java serialized table, see `examples/generate.rs`. The `rust_embedded_stempel`
//! feature embeds a ~240KiB stemming table converted from the stempel stemmer project as `rstempel::rust::STEMMER`.
//! 
//! The `java` stemmer can load tables in the format used by the Java `stempel` implementation. A compressed stemming
//! table from the stempel stemmer project is included in `src/tables/stemmer_2000.out.gz`. A much larger and more
//! accurate stemming table can be sourced from [pystempel](https://github.com/dzieciou/pystempel).
//! 
//! ## Acknowledgements
//! 
//! This product includes software developed by the Egothor Project. http://egothor.sf.net/

#[cfg(feature = "java")]
pub mod java;

#[cfg(feature = "rust")]
pub mod rust;

pub trait Stem {
    /// If the stemmed word is unchanged, returns `Cow::Borrowed(word)`,
    /// else returns `Cow::Owned` with the stemmed word.
    fn stem<'a>(&self, word: &'a str) -> std::borrow::Cow<'a, str>;
}
