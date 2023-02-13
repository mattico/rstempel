use super::serialize::*;
use super::trie::{Trie, TrieGet};
use std::io;

pub struct MultiTrie {
    pub tries: Vec<Trie>,
    #[allow(dead_code)]
    pub forward: bool,
    #[allow(dead_code)]
    pub by: i32,
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
    fn get_cmd(&self, key: &str) -> Option<String> {
        let mut result = String::with_capacity(self.tries.len() * 2);
        for trie in &self.tries {
            let r = trie.get_cmd(key)?;
            if r == "*" {
                return Some(result);
            }
            result.push_str(&r);
        }
        Some(result)
    }
}

pub struct MultiTrie2 {
    pub t: MultiTrie,
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

fn length_pp(cmd: &str) -> usize {
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
    if cnt == 0 {
        return Some(i);
    }
    let mut iter = i.char_indices();
    let mut start = 0;
    let mut end = i.len();
    if trie.forward {
        for _ in 0..cnt {
            start = iter.next()?.0;
        }
    } else {
        for _ in 0..cnt {
            end = iter.next_back()?.0;
        }
    }
    Some(&i[start..end])
}

fn get_cmd_(
    trie: &Trie,
    key: &mut &str,
    last_key: &str,
    last_ch: &mut char,
    prev_cmd: &mut Option<String>,
) -> Option<String> {
    let r = trie.get_cmd(last_key)?;
    if r == "*" {
        return None;
    }
    if cannot_follow(*last_ch, r.chars().next()?) {
        return None;
    } else {
        *last_ch = r.chars().nth_back(1)?;
    }
    if r.starts_with('-') {
        if let Some(prev_cmd) = prev_cmd.as_ref() {
            *key = skip(trie, key, length_pp(prev_cmd))?;
        }
        *key = skip(trie, key, length_pp(&r))?;
    }
    *prev_cmd = Some(r.clone());
    Some(r)
}

impl TrieGet for MultiTrie2 {
    fn get_cmd(&self, mut key: &str) -> Option<String> {
        let mut result = String::with_capacity(self.t.tries.len() * 2);
        let mut last_key = key;
        let mut prev_cmd = None;
        let mut last_ch = ' ';
        for trie in &self.t.tries {
            match get_cmd_(trie, &mut key, last_key, &mut last_ch, &mut prev_cmd) {
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
    use std::fs;
    use flate2::bufread::GzDecoder;

    #[test]
    fn test_lookup_multi2() {
        let path = "src/tables/stemmer_2000.out.gz";
        let input = fs::File::open(path).unwrap();
        let input = io::BufReader::new(GzDecoder::new(io::BufReader::new(input)));
        let mut reader = DataInput::new(input);
        let params = reader.read_string().unwrap();
        assert!(params.contains('M'));
        let trie = MultiTrie2::deserialize(&mut reader).unwrap();
        let cmd = trie.get_cmd("Abadan").unwrap();
        assert_eq!(cmd, "Ia-e");
    }
}
