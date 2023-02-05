use crate::serialize::*;
use crate::trie::{Trie, TrieGet};
use std::io;

pub struct MultiTrie {
    tries: Vec<Trie>,
    #[allow(dead_code)]
    forward: bool,
    #[allow(dead_code)]
    by: i32,
}

impl JavaDeserialize for MultiTrie {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self> {
        let forward = reader.read_bool()?;
        let by = reader.read_i32()?;
        let count = reader.read_usize()?;
        let mut tries = Vec::with_capacity(count);
        for _ in 0..count {
            tries.push(Trie::deserialize(reader)?);
        }
        Ok(Self { tries, forward, by })
    }
}

impl TrieGet for MultiTrie {
    fn get_last_on_path(&self, key: &str) -> Option<String> {
        let mut result = String::with_capacity(self.tries.len() * 2);
        for trie in &self.tries {
            let r = trie.get_last_on_path(key)?;
            if r == "*" {
                return Some(result);
            }
            result.push_str(&r);
        }
        Some(result)
    }
}

pub struct MultiTrie2 {
    t: MultiTrie,
}

impl JavaDeserialize for MultiTrie2 {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self> {
        let t = reader.read()?;
        Ok(Self { t })
    }
}

fn cannot_follow(after: char, goes: char) -> bool {
    match after {
        '-' | 'D' => after == goes,
        _ => false,
    }
}

fn length_pp(cmd: &String) -> usize {
    let mut len = 0;
    let mut iter = cmd.chars();
    while let (Some(cmd), Some(arg)) = (iter.next(), iter.next()) {
        match (cmd, arg) {
            ('-' | 'D', c) => {
                assert!(c.is_ascii_lowercase());
                len += 1 + ((c as u8) - b'a') as usize;
            }
            ('R', _) => len += 1,
            _ => {}
        }
    }
    len
}

fn skip<'a>(trie: &Trie, i: &'a str, cnt: usize) -> Option<&'a str> {
    let mut iter = i.char_indices();
    if trie.forward {
        for _ in 0..cnt {
            iter.next()?;
        }
    } else {
        for _ in 0..cnt {
            iter.next_back()?;
        }
    }
    let start = iter.next()?.0;
    let end = iter.next_back()?.0;
    Some(&i[start..end])
}

fn get_last_on_path_(
    trie: &Trie,
    key: &mut &str,
    last_key: &str,
    last_ch: &mut char,
    p: &mut Option<String>,
) -> Option<String> {
    let r = trie.get_last_on_path(last_key)?;
    if r == "*" {
        return None;
    }
    if cannot_follow(*last_ch, r.chars().next()?) {
        return None;
    } else {
        *last_ch = r.chars().nth_back(1)?;
    }
    if r.starts_with('-') {
        let p = p.as_ref().unwrap_or(&r);
        *key = skip(trie, key, length_pp(p))?;
    }
    *p = Some(r.clone());
    Some(r)
}

impl TrieGet for MultiTrie2 {
    fn get_last_on_path(&self, mut key: &str) -> Option<String> {
        let mut result = String::with_capacity(self.t.tries.len() * 2);
        let mut last_key = key;
        let mut p = None;
        let mut last_ch = ' ';
        for trie in &self.t.tries {
            match get_last_on_path_(trie, &mut key, last_key, &mut last_ch, &mut p) {
                None => break,
                Some(r) => result.push_str(&r),
            }
            if !key.is_empty() {
                last_key = key;
            }
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tries() {
        let trie_files = [
            (
                "stemmer_100.out",
                include_bytes!("tables/stemmer_100.out").as_slice(),
            ),
            (
                "stemmer_200.out",
                include_bytes!("tables/stemmer_200.out").as_slice(),
            ),
            (
                "stemmer_500.out",
                include_bytes!("tables/stemmer_500.out").as_slice(),
            ),
            (
                "stemmer_700.out",
                include_bytes!("tables/stemmer_700.out").as_slice(),
            ),
            (
                "stemmer_1000.out",
                include_bytes!("tables/stemmer_1000.out").as_slice(),
            ),
            (
                "stemmer_2000.out",
                include_bytes!("tables/stemmer_2000.out").as_slice(),
            ),
        ];
        for (name, data) in trie_files {
            let mut reader = DataInput::new(io::Cursor::new(data));
            let multi = reader.read_string().unwrap();
            let multi = multi.contains(['M', 'm']);
            assert!(
                multi,
                "Expected stemmer table {} to contain a multitrie",
                name
            );
            match MultiTrie2::deserialize(&mut reader) {
                Err(e) => {
                    panic!("Loading trie {} failed with {:?}", name, e);
                }
                Ok(trie) => {
                    println!("tries {}", trie.t.tries.len());
                }
            }
        }
    }
}
