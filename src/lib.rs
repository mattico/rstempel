pub mod java;

pub trait Stem {
    fn stem<'a>(&self, word: &'a str) -> Option<std::borrow::Cow<'a, str>>;
}
