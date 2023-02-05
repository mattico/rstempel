//                 Egothor Software License version 1.00
//                 Copyright (C) 1997-2004 Leo Galambos.
//              Copyright (C) 2002-2004 "Egothor developers"
//                   on behalf of the Egothor Project.
//                          All rights reserved.
//
// This  software  is  copyrighted  by  the "Egothor developers". If this
// license applies to a single file or document, the "Egothor developers"
// are the people or entities mentioned as copyright holders in that file
// or  document.  If  this  license  applies  to the Egothor project as a
// whole,  the  copyright holders are the people or entities mentioned in
// the  file CREDITS. This file can be found in the same location as this
// license in the distribution.
//
// Redistribution  and  use  in  source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
// 1. Redistributions  of  source  code  must retain the above copyright
// notice, the list of contributors, this list of conditions, and the
// following disclaimer.
// 2. Redistributions  in binary form must reproduce the above copyright
// notice, the list of contributors, this list of conditions, and the
// disclaimer  that  follows  these  conditions  in the documentation
// and/or other materials provided with the distribution.
// 3. The name "Egothor" must not be used to endorse or promote products
// derived  from  this software without prior written permission. For
// written permission, please contact Leo.G@seznam.cz
// 4. Products  derived  from this software may not be called "Egothor",
// nor  may  "Egothor"  appear  in  their name, without prior written
// permission from Leo.G@seznam.cz.
//
// In addition, we request that you include in the end-user documentation
// provided  with  the  redistribution  and/or  in the software itself an
// acknowledgement equivalent to the following:
// "This product includes software developed by the Egothor Project.
//  http://egothor.sf.net/"
//
// THIS  SOFTWARE  IS  PROVIDED  ``AS  IS''  AND ANY EXPRESSED OR IMPLIED
// WARRANTIES,  INCLUDING,  BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
// MERCHANTABILITY  AND  FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
// IN  NO  EVENT  SHALL THE EGOTHOR PROJECT OR ITS CONTRIBUTORS BE LIABLE
// FOR   ANY   DIRECT,   INDIRECT,  INCIDENTAL,  SPECIAL,  EXEMPLARY,  OR
// CONSEQUENTIAL  DAMAGES  (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE  GOODS  OR  SERVICES;  LOSS  OF  USE,  DATA, OR PROFITS; OR
// BUSINESS  INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER  IN  CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE
// OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN
// IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// This  software  consists  of  voluntary  contributions  made  by  many
// individuals  on  behalf  of  the  Egothor  Project  and was originally
// created by Leo Galambos (Leo.G@seznam.cz).

use crate::serialize::*;
use std::io;
use std::{collections::BTreeMap, ops::Index};

pub trait TrieGet {
    /// Return the command for the string key
    fn get_last_on_path(&self, key: &str) -> Option<String>;
}

/// A Cell is a portion of a trie.
#[derive(Default, Debug, Clone)]
pub struct Cell {
    /// next row id in this way
    pub refr: Option<u32>,
    /// command of the cell
    pub cmd: Option<u32>,
    /// how many cmd-s was in subtrie before pack()
    pub cnt: u32,
    /// how many chars would be discarded from input key in this way
    pub skip: u32,
}

impl JavaDeserialize for Cell {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self> {
        let cmd = reader.read_u32_opt()?;
        let cnt = reader.read_u32()?;
        let refr = reader.read_u32_opt()?;
        let skip = reader.read_u32()?;
        Ok(Self {
            refr,
            cmd,
            cnt,
            skip,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct Row {
    pub cells: BTreeMap<char, Cell>,
    pub uniform_count: u32,
    pub uniform_skip: u32,
}

impl Row {
    pub fn get_cmd(&self, way: char) -> Option<u32> {
        self.cells.get(&way)?.cmd
    }

    pub fn get_ref(&self, way: char) -> Option<u32> {
        self.cells.get(&way)?.refr
    }
}

impl Index<char> for Row {
    type Output = Cell;

    fn index(&self, index: char) -> &Self::Output {
        &self.cells[&index]
    }
}

impl JavaDeserialize for Row {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self> {
        let num = reader.read_usize()?;
        let mut cells = BTreeMap::new();

        for _ in 0..num {
            let ch = reader.read_char()?;
            let cell = Cell::deserialize(reader)?;
            cells.insert(ch, cell);
        }

        Ok(Self {
            cells,
            ..Default::default()
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct Trie {
    pub(crate) rows: Vec<Row>,
    pub(crate) cmds: Vec<String>,
    pub(crate) root: u32,
    pub(crate) forward: bool,
}

// TODO: looks like rows and cmds are basically arenas and we're using indexes into
// them as handles. Should we switch to an arena or a trie crate or make a handle type or...?
impl Trie {
    pub fn row(&self, index: u32) -> Option<&Row> {
        self.rows.get(index as usize)
    }
}

impl TrieGet for Trie {
    fn get_last_on_path(&self, key: &str) -> Option<String> {
        let mut now = self.row(self.root)?;
        let mut chars = KeyIter::new(self.forward, key);
        let mut last = None;
        let last_ch = chars.next_back().unwrap();
        for ch in chars {
            if let Some(idx) = now.get_cmd(ch) {
                last = self.cmds.get(idx as usize);
            }
            if let Some(idx) = now.get_ref(ch) {
                now = self.row(idx)?;
            } else {
                return last.cloned();
            }
        }
        if let Some(idx) = now.get_cmd(last_ch) {
            self.cmds.get(idx as usize)
        } else {
            last
        }
        .cloned()
    }
}

impl JavaDeserialize for Trie {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self> {
        let forward = reader.read_bool()?;
        let root = reader.read_u32()?;
        let num_cmds = reader.read_usize()?;
        let mut cmds = Vec::with_capacity(num_cmds);
        for _ in 0..num_cmds {
            cmds.push(reader.read_string()?);
        }
        let num_rows = reader.read_usize()?;
        let mut rows = Vec::with_capacity(num_rows);
        for _ in 0..num_rows {
            rows.push(Row::deserialize(reader)?);
        }
        Ok(Self {
            forward,
            root,
            cmds,
            rows,
        })
    }
}

#[derive(Clone)]
struct KeyIter<'a> {
    inner: std::str::Chars<'a>,
    forward: bool,
}

impl<'a> KeyIter<'a> {
    pub fn new(forward: bool, key: &'a str) -> Self {
        Self {
            inner: key.chars(),
            forward,
        }
    }
}

impl<'a> Iterator for KeyIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.forward {
            self.inner.next()
        } else {
            self.inner.next_back()
        }
    }
}

impl<'a> DoubleEndedIterator for KeyIter<'a> {
    fn next_back(&mut self) -> Option<char> {
        if self.forward {
            self.inner.next_back()
        } else {
            self.inner.next()
        }
    }
}
