#![allow(dead_code)]

use std::{io, borrow::Cow};
use multitrie::MultiTrie2;
use serialize::{DataInput, JavaDeserialize};
use trie::{TrieGet, Trie};

mod trie;
mod multitrie;
mod serialize;
mod diff;

pub struct Stemmer {
    trie: Box<dyn TrieGet>
}

impl Stemmer {
    pub fn load<R: io::Read>(reader: R) -> io::Result<Self> {
        let mut reader = DataInput::new(reader);
        let method = reader.read_string()?;
        let multi = method.contains(['M', 'm']);
        let trie: Box<dyn TrieGet> = if multi {
            Box::new(MultiTrie2::deserialize(&mut reader)?)
        } else {
            Box::new(Trie::deserialize(&mut reader)?)
        };
        Ok(Self { trie })
    }

    pub fn stem<'a>(&self, word: &'a str) -> Option<Cow<'a, str>> {
        if word.len() < 3 {
            return Some(Cow::Borrowed(word));
        }
        let cmd = self.trie.get_last_on_path(word)?;
        let res = diff::apply(word, &cmd);
        if res.is_empty() {
            None
        } else {
            Some(Cow::Owned(res))
        }
    }
}
