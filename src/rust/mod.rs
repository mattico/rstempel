use std::{
    borrow::Cow,
    collections::HashMap,
    num::{NonZeroU16, NonZeroU32},
};

use crate::Stem;

#[path = "../tables/stemmer_2000.rs"]
mod generated_stemmer;

pub use generated_stemmer::STEMMER;

#[cfg(feature = "generate")]
pub mod generate;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    Skip { chars: u8 },
    Delete { chars: u8 },
    Replace { char: char },
    Insert { char: char },
}

impl Command {
    pub fn parse(cmd: char, param: char) -> Option<Self> {
        match cmd {
            '-' => {
                assert!(param.is_ascii_lowercase());
                let chars = 1 + (param as u8) - b'a';
                Some(Self::Skip { chars })
            }
            'D' => {
                assert!(param.is_ascii_lowercase());
                let chars = 1 + (param as u8) - b'a';
                Some(Self::Delete { chars })
            }
            'R' => Some(Self::Replace { char: param }),
            'I' => Some(Self::Insert { char: param }),
            _ => None,
        }
    }

    fn cannot_follow(&self, prev: Command) -> bool {
        matches!(
            (&self, prev),
            (Command::Skip { .. }, Command::Skip { .. })
                | (Command::Delete { .. }, Command::Delete { .. })
        )
    }

    fn is_skip(&self) -> bool {
        matches!(*self, Command::Skip { .. })
    }
    #[allow(dead_code)]
    fn is_delete(&self) -> bool {
        matches!(*self, Command::Delete { .. })
    }
    #[allow(dead_code)]
    fn is_replace(&self) -> bool {
        matches!(*self, Command::Replace { .. })
    }
    #[allow(dead_code)]
    fn is_insert(&self) -> bool {
        matches!(*self, Command::Insert { .. })
    }

    fn length_pp(&self) -> usize {
        match *self {
            Command::Skip { chars } | Command::Delete { chars } => chars as usize,
            Command::Replace { .. } => 1,
            Command::Insert { .. } => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/// Represents a slice of commands in the Stemmer's commands vec, packed into a u32.
pub struct CommandSlice(pub NonZeroU32);

impl CommandSlice {
    #[must_use]
    pub fn new_eom() -> Self {
        Self(NonZeroU32::new(u32::MAX).unwrap())
    }

    #[must_use]
    pub fn new(index: usize, len: usize) -> Self {
        assert!(len > 0 && len <= 0xF);
        assert!(index < (1 << 24));
        let packed = (index as u32) << 4 | (len as u32);
        assert!(packed != u32::MAX);
        Self(NonZeroU32::new(packed).unwrap())
    }

    /// True if this is an EndOfMultiTrie marker.
    #[must_use]
    pub fn is_eom(self) -> bool {
        self.0.get() == u32::MAX
    }

    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(self) -> usize {
        debug_assert!(!self.is_eom());
        (self.0.get() & 0xF) as usize
    }

    #[must_use]
    pub fn start_index(self) -> usize {
        debug_assert!(!self.is_eom());
        (self.0.get() >> 4) as usize
    }

    #[must_use]
    pub fn lookup(self, commands: &[Command]) -> &[Command] {
        debug_assert!(!self.is_eom());
        let idx = self.start_index();
        let len = self.len();
        &commands[idx..(idx + len)]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// A reference to the next row in the trie, if present.
    /// With this + the next char in the word you can find the next cell.
    /// The row index starts at 1 so we can use the NonZeroU16 + Option size optimization.
    pub refr: Option<NonZeroU16>,
    /// An index+len of commands in the Stemmer's command vec, or an EndOfMultiTrie marker.
    pub cmds: Option<CommandSlice>,
}

#[derive(Clone, Copy)]
/// A row is basically a `Map<char, Cell>`.
pub struct Row {
    /// List of cell values. Each cell's `char` key is at the corresponding index in `chars`.
    pub cells: &'static [Cell],
    /// Sorted list of `char`s, used to lookup the matching index of the cell.
    /// Stored separately from cells for better cache efficiency during lookup.
    pub chars: &'static [char],
}

impl Row {
    pub fn get(&self, ch: char) -> Option<&Cell> {
        let idx = self.chars.binary_search(&ch).ok()?;
        Some(&self.cells[idx])
    }
}

pub struct Trie {
    pub rows: &'static [Row],
}

impl Trie {
    fn get(&self, word: &[char]) -> Option<CommandSlice> {
        let mut row = self.rows[0];
        let mut last = None;
        for &ch in word.iter().rev() {
            if let Some(cell) = row.get(ch) {
                if let Some(cmds) = cell.cmds {
                    last = Some(cmds);
                }
                if let Some(next_row) = cell.refr {
                    let next_row = (next_row.get() - 1) as usize;
                    row = self.rows[next_row];
                }
            } else {
                break;
            }
        }
        last
    }
}

pub struct Stemmer {
    /// Flattened list of deduplicated command lists.
    commands: &'static [Command],
    tries: &'static [Trie],
}

impl Stem for Stemmer {
    fn stem<'a>(&self, word: &'a str) -> Cow<'a, str> {
        if word.chars().count() <= 3 {
            return Cow::Borrowed(word);
        }
        let result = word.chars().collect::<Vec<char>>();
        let cmds = match self.get_cmd(&result) {
            Some(c) => c,
            None => return Cow::Borrowed(word),
        };
        apply_edits(result, &cmds).map_or(Cow::Borrowed(word), Cow::from)
    }
}

fn apply_edits(mut result: Vec<char>, cmds: &[Command]) -> Option<String> {
    let mut pos = (result.len() - 1) as isize;
    for &command in cmds {
        match command {
            Command::Skip { chars } => pos -= chars as isize,
            Command::Delete { chars } => {
                let e = usize::try_from(pos).ok()?;
                pos -= chars as isize;
                let s = usize::try_from(pos).ok()?;
                result.drain(s..=e);
            }
            Command::Replace { char } => *result.get_mut(usize::try_from(pos).ok()?)? = char,
            Command::Insert { char } => {
                pos += 1;
                result.insert(usize::try_from(pos).ok()?, char);
            }
        }
        pos -= 1;
    }
    if result.is_empty() {
        None
    } else {
        Some(result.into_iter().collect())
    }
}

fn skip(key: &mut &[char], cmds: &[Command]) -> bool {
    let cnt: usize = cmds.iter().map(|c| c.length_pp()).sum();
    if cnt == 0 {
        return true;
    }
    if cnt >= key.len() {
        return false;
    }
    let end = key.len() - cnt;
    *key = &key[..end];
    true
}

impl Stemmer {
    fn get_cmd(&self, mut key: &[char]) -> Option<Vec<Command>> {
        let mut result = Vec::new();
        let mut last_key = key;
        let mut prev_cmds = None;
        let mut last_cmd = None;
        for trie in self.tries {
            let cmd = match trie.get(last_key) {
                Some(cs) if cs.is_eom() => break,
                Some(cs) => cs.lookup(self.commands),
                None => break,
            };
            if let Some(lc) = last_cmd {
                if cmd[0].cannot_follow(lc) {
                    break;
                }
            }
            last_cmd = cmd.last().cloned();
            if cmd[0].is_skip() {
                if let Some(prev_cmds) = prev_cmds {
                    if !skip(&mut key, prev_cmds) {
                        break;
                    }
                }
                if !skip(&mut key, cmd) {
                    break;
                }
            }
            prev_cmds = Some(cmd);
            result.extend_from_slice(cmd);
            if !key.is_empty() {
                last_key = key;
            }
        }
        Some(result)
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
        let path = "src/tables/polimorf-out.tab.gz";
        let file = fs::File::open(path).unwrap();
        let mut reader = BufReader::new(GzDecoder::new(BufReader::new(file)));
        let mut line = String::new();

        let stemmer: &Stemmer = &STEMMER;

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
