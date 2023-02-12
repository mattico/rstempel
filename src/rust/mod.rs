use std::{
    borrow::Cow,
    collections::HashMap,
    num::{NonZeroU16, NonZeroU32},
};

use crate::Stem;

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

pub enum StemResult {
    /// Continue with the next trie for stemming.
    Continue,
    /// No edit command found, return the input word.
    Unchanged,
    /// Stemming completed by this trie, don't continue to the next trie.
    Completed,
}

impl Trie {
    fn stem(&self, commands: &'static [Command], result: &mut Vec<char>) -> StemResult {
        let cmds = match self.get(result) {
            None => return StemResult::Unchanged,
            Some(c) if c.is_eom() => return StemResult::Completed,
            Some(c) => c.lookup(commands),
        };

        // TODO: implement commands like in MultiTrie2 with the lengthPP stuff
        let mut idx = result.len() - 1;
        for &command in cmds {
            match command {
                Command::Skip { chars } => match idx.checked_sub(chars.into()) {
                    Some(r) => idx = r,
                    None => break,
                },
                Command::Delete { chars } => {
                    let end = idx;
                    idx = idx.saturating_sub(chars.into());
                    result.drain(idx..end);
                }
                Command::Replace { char } => result[idx] = char,
                Command::Insert { char } => {
                    idx += 1;
                    result.insert(idx, char);
                }
            }
            if result.is_empty() {
                return StemResult::Unchanged;
            }
        }

        if result.is_empty() {
            StemResult::Unchanged
        } else {
            StemResult::Continue
        }
    }

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
        if word.len() < 3 {
            return Cow::Borrowed(word);
        }
        let mut result = word.chars().collect::<Vec<char>>();
        for trie in self.tries {
            match trie.stem(self.commands, &mut result) {
                StemResult::Continue => {}
                StemResult::Unchanged => return Cow::Borrowed(word),
                StemResult::Completed => break,
            }
        }
        if result.is_empty() {
            Cow::Borrowed(word)
        } else {
            Cow::Owned(result.into_iter().collect())
        }
    }
}
