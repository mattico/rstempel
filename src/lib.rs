use multitrie::MultiTrie2;
use serialize::{DataInput, JavaDeserialize};
use std::{borrow::Cow, io};
use trie::{Trie, TrieGet};

mod diff;
mod multitrie;
mod serialize;
mod trie;

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

#[cfg(test)]
mod test {
    use super::*;
    use flate2::bufread::GzDecoder;
    use std::io::{prelude::*, BufReader};
    use std::fs;

    #[test]
    fn test_compare_stem_to_stempel() {
        let path = "src/tables/polimorf-out.tab.gz";
        let file = fs::File::open(path).unwrap();
        let mut reader = BufReader::new(GzDecoder::new(BufReader::new(file)));
        let mut line = String::new();

        let stemmer = Stemmer::load(BufReader::new(fs::File::open("src/tables/stemmer_2000.out").unwrap())).unwrap();

        let mut num = 0;
        while reader.read_line(&mut line).unwrap() > 0 {
            num += 1;
            let mut line = line.split_ascii_whitespace();
            let input = line.next().unwrap();
            let output = line.next().unwrap();

            if let Some(our_output) = stemmer.stem(input) {
                if output != our_output {
                    panic!("On line {} input={} output={} our_output={}", num, input, output, our_output);
                }
            }
        }
    }
}
