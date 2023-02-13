use multitrie::MultiTrie2;
use serialize::{DataInput, JavaDeserialize};
use std::{borrow::Cow, io};
use trie::{Trie, TrieGet};

pub(crate) mod diff;
pub(crate) mod multitrie;
pub(crate) mod serialize;
pub(crate) mod trie;

pub struct Stemmer {
    trie: Box<dyn TrieGet>,
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
}

impl crate::Stem for Stemmer {
    fn stem<'a>(&self, word: &'a str) -> Cow<'a, str> {
        // Technically this should be grapheme clusters but the java version assumes that a UTF-16 char is a single char
        // so this should work everywhere that does.
        if word.chars().count() <= 3 {
            return Cow::Borrowed(word); // No change
        }
        let cmd = match self.trie.get_cmd(word) {
            Some(c) => c,
            None => return Cow::Borrowed(word),
        };
        diff::apply(word, &cmd).map_or(Cow::Borrowed(word), Cow::from)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Stem;
    use flate2::bufread::GzDecoder;
    use std::fs;
    use std::io::{prelude::*, BufReader};

    #[test]
    fn test_compare_stem_to_stempel() {
        let path = "src/tables/polimorf_words_stemmed.tab.gz";
        let file = fs::File::open(path).unwrap();
        let mut reader = BufReader::new(GzDecoder::new(BufReader::new(file)));
        let mut line = String::new();

        let stemmer = Stemmer::load(BufReader::new(
            fs::File::open("src/tables/stemmer_2000.out").unwrap(),
        ))
        .unwrap();

        let mut num = 0;
        while reader.read_line(&mut line).unwrap() > 0 {
            num += 1;
            let mut split = line.split_ascii_whitespace();
            let input = split.next().unwrap();
            let output = split.next().unwrap();
            println!("On line {} input={} output={}", num, input, output);

            let ours = stemmer.stem(input);
            if output != ours {
                panic!(
                    "On line {} input={} output={} ours={}",
                    num, input, output, ours
                );
            }
            line.clear();
        }
    }
}
