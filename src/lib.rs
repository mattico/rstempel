#[cfg(feature = "java")]
pub mod java;
#[cfg(feature = "rust")]
pub mod rust;

pub trait Stem {
    /// If the stemmed word is unchanged, returns `Cow::Borrowed(word)`,
    /// else returns `Cow::Owned` with the stemmed word.
    fn stem<'a>(&self, word: &'a str) -> std::borrow::Cow<'a, str>;
}
