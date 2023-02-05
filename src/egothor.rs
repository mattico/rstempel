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

use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::io;
use std::{collections::BTreeMap, ops::Index};

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

impl Cell {
    pub fn is_used(&self) -> bool {
        self.refr.is_some() || self.cmd.is_some()
    }
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

impl JavaSerialize for Cell {
    fn serialize<W: io::Write>(&self, writer: &mut DataOutput<W>) -> io::Result<()> {
        writer.write_u32_opt(self.cmd)?;
        writer.write_u32(self.cnt)?;
        writer.write_u32_opt(self.refr)?;
        writer.write_u32(self.skip)?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct Row {
    pub cells: BTreeMap<char, Cell>,
    pub uniform_count: u32,
    pub uniform_skip: u32,
}

impl Row {
    pub fn from_row_cells(old: &Row) -> Self {
        Self {
            cells: old.cells.clone(),
            ..Default::default()
        }
    }

    pub fn set_cmd(&mut self, way: char, cmd: Option<u32>) {
        let cnt = cmd.iter().count() as u32;
        let cell = self.cells.entry(way).or_default();
        cell.cmd = cmd;
        cell.cnt = cnt;
    }

    pub fn set_ref(&mut self, way: char, refr: Option<u32>) {
        let cell = self.cells.entry(way).or_default();
        cell.refr = refr;
    }

    /// Return the number of cells in use.
    pub fn num_used_cells(&self) -> usize {
        self.cells.values().filter(|c| c.is_used()).count()
    }

    /// Return the number of references (how many transitions) to other rows.
    pub fn num_referenced_cells(&self) -> usize {
        self.cells.values().filter(|c| c.refr.is_some()).count()
    }

    /// Return the number of patch commands saved in this Row.
    pub fn num_patch_commands(&self) -> usize {
        self.cells.values().filter(|c| c.cmd.is_some()).count()
    }

    pub fn get_cmd(&self, way: char) -> Option<u32> {
        self.cells.get(&way)?.cmd
    }

    pub fn get_cnt(&self, way: char) -> Option<u32> {
        Some(self.cells.get(&way)?.cnt)
    }

    pub fn get_ref(&self, way: char) -> Option<u32> {
        self.cells.get(&way)?.refr
    }

    /// Return the number of identical Cells (containing patch commands) in this Row.
    ///
    /// eq_skip: when set to `false` the removed patch commands are considered
    ///
    /// Returns the number of identical Cells, or `None` if there are (at least) two different cells.
    // TODO: change this so it doesn't modify uniform_count, etc. The logic of this is weird but I don't understand it yet.
    pub fn uniform_cmds(&mut self, eq_skip: bool) -> Option<u32> {
        self.uniform_count = 1;
        self.uniform_skip = 0;
        let mut ret = None;
        for cell in self.cells.values() {
            if cell.refr.is_some() {
                return None;
            }
            if let Some(cmd) = cell.cmd {
                match ret {
                    None => {
                        ret = Some(cmd);
                        self.uniform_skip = cell.skip;
                    }
                    Some(r) => {
                        if r == cmd {
                            if eq_skip {
                                if self.uniform_skip == cell.skip {
                                    self.uniform_count += 1;
                                } else {
                                    return None;
                                }
                            }
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
        ret
    }

    pub fn try_get(&self, index: char) -> Option<&Cell> {
        self.cells.get(&index)
    }

    pub fn try_get_mut(&mut self, index: char) -> Option<&mut Cell> {
        self.cells.get_mut(&index)
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

impl JavaSerialize for Row {
    fn serialize<W: io::Write>(&self, writer: &mut DataOutput<W>) -> io::Result<()> {
        writer.write_usize(self.cells.len())?;
        for (&ch, cell) in self.cells.iter() {
            if cell.is_used() {
                writer.write_char(ch)?;
                cell.serialize(writer)?;
            }
        }
        Ok(())
    }
}

pub mod reduce {
    use super::*;
    pub fn optimize(orig: Trie) -> Trie {
        orig
    }
}

#[derive(Default, Debug, Clone)]
pub struct Trie {
    rows: Vec<Row>,
    cmds: Vec<String>,
    root: usize,
    forward: bool,
}

// TODO: looks like rows and cmds are basically arenas and we're using indexes into
// them as handles. Should we switch to an arena or a trie crate or make a handle type or...?
impl Trie {
    pub fn new(forward: bool) -> Self {
        Self {
            rows: vec![Row::default()],
            cmds: Vec::new(),
            root: 0,
            forward,
        }
    }

    // TODO
    // pub fn get_all(key: &str) -> Vec<String> {}

    pub fn num_used_cells(&self) -> usize {
        self.rows.iter().map(|r| r.num_used_cells()).sum()
    }

    pub fn num_referenced_cells(&self) -> usize {
        self.rows.iter().map(|r| r.num_referenced_cells()).sum()
    }

    pub fn num_patch_commands(&self) -> usize {
        self.rows.iter().map(|r| r.num_patch_commands()).sum()
    }

    pub fn get_fully(&self, key: &str) -> Option<String> {
        let mut now = self.row(self.root)?;
        let mut cmd = None;
        let mut chars = KeyIter::new(self.forward, key);
        while let Some(ch) = chars.next() {
            let cell = now.try_get(ch)?;
            cmd = cell.cmd;

            for _ in 0..cell.skip {
                let _ = chars.next()?;
            }

            // TODO: I think the ? logic is incorrect here
            // Should only return None if we're before the last char
            // otherwise fallthrough to the last line and return cmd.
            // Probably a better way to translate this.
            now = self.rows.get(now.get_ref(ch)? as usize)?;
        }
        cmd.and_then(|idx| self.cmds.get(idx as usize)).cloned()
    }

    pub fn get_last_on_path(&self, key: &str) -> Option<String> {
        let mut now = self.row(self.root)?;
        let mut chars = KeyIter::new(self.forward, key);
        let mut last = None;
        let last_ch = chars.next_back().unwrap();
        for ch in chars {
            if let Some(idx) = now.get_cmd(ch) {
                last = self.cmds.get(idx as usize);
            }
            if let Some(idx) = now.get_ref(ch) {
                now = self.row(idx as usize)?;
            } else {
                return last.cloned();
            }
        }
        if let Some(idx) = now.get_cmd(last_ch) {
            self.cmds.get(idx as usize)
        } else {
            last
        }.cloned()
    }

    pub fn add(&mut self, key: &str, cmd: &str) {
        if cmd.is_empty() || key.is_empty() {
            return;
        }

        let id_cmd = self.cmds.iter().position(|x| x == cmd).unwrap_or_else(|| {
            let id = self.cmds.len();
            self.cmds.push(cmd.to_string());
            id
        });

        let mut node = self.root as u32;
        let rows_len = self.rows.len();
        let mut chars = KeyIter::new(self.forward, key);
        let last = chars.next_back().unwrap();
        for ch in chars {
            if let Some(n) = r.get_ref(ch) {
                r = self.rows.get_mut(n as usize).unwrap();
            } else {
                node = rows_len as u32;
                self.rows.push(Row::default());
                let n = self.rows.last_mut().unwrap();
                r.set_ref(ch, Some(node));
                r = n;
            }
        }
        r.set_cmd(last, Some(id_cmd as u32));
    }

    pub fn node(&self, row_idx: impl Into<usize>, char: char) -> Option<()> {
        self.row(row_idx)?.get_
    }

    pub fn row(&self, index: impl Into<usize>) -> Option<&Row> {
        self.rows.get(index.into())
    }

    pub fn row_mut(&mut self, index: impl Into<usize>) -> Option<&mut Row> {
        self.rows.get_mut(index.into())
    }
}

impl JavaSerialize for Trie {
    fn serialize<W: io::Write>(&self, writer: &mut DataOutput<W>) -> io::Result<()> {
        writer.write_bool(self.forward)?;
        writer.write_usize(self.root)?;
        writer.write_usize(self.cmds.len())?;
        for cmd in &self.cmds {
            writer.write_string(cmd)?;
        }
        writer.write_usize(self.rows.len())?;
        for row in &self.rows {
            row.serialize(writer)?;
        }
        Ok(())
    }
}

impl JavaDeserialize for Trie {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self> {
        let forward = reader.inner.read_u8()? != 0;
        let root = reader.read_usize()?;
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

trait JavaSerialize {
    fn serialize<W: io::Write>(&self, writer: &mut DataOutput<W>) -> io::Result<()>;
}

trait JavaDeserialize: Sized {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self>;
}

/// Reads binary data in a manner compatible with
/// [Java's DataInput class](https://docs.oracle.com/javase/7/docs/api/java/io/DataInput.html).
struct DataInput<R: io::Read> {
    inner: R,
}

impl<R: io::Read> DataInput<R> {
    pub fn new(reader: R) -> Self {
        Self { inner: reader }
    }

    /// Like Java's `readBoolean`.
    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.inner.read_u8()? != 0)
    }

    /// Like Java's `readInt` but casts the result to `u32`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.inner
            .read_i32::<BE>()?
            .try_into()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    }

    /// Like Java's `readInt` but casts the result to `usize`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn read_usize(&mut self) -> io::Result<usize> {
        Ok(self.read_u32()? as usize)
    }

    /// Like Java's `readInt` but casts the result to `u32`, returning [`Option::None`] if the value is negative.
    pub fn read_u32_opt(&mut self) -> io::Result<Option<u32>> {
        Ok(self.inner.read_i32::<BE>()?.try_into().ok())
    }

    /// Like Java's `readChar`. Reads a UTF-16 code unit and converts it to a rust [`char`], returning
    /// [`std::io::ErrorKind::InvalidData`] if the single UTF-16 code unit is not a valid UTF-16 code point.
    pub fn read_char(&mut self) -> io::Result<char> {
        let utf16_char = self.inner.read_u16::<BE>()?;
        char::decode_utf16(std::iter::once(utf16_char))
            .next()
            .unwrap()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    }

    /// Like Java's `readUTF`. Reads a modified UTF-8 string with length. Returns [`std::io::ErrorKind::InvalidData`]
    /// if the value is not a valid UTF-8 string.
    pub fn read_string(&mut self) -> io::Result<String> {
        let len = self.inner.read_u16::<BE>()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let mut buf = vec![0u8; len];
        self.inner.read_exact(&mut buf)?;
        let str = cesu8::from_java_cesu8(&buf)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        Ok(str.into_owned())
    }
}

/// Writes binary data in a manner compatible with
/// [Java's DataOutput class](https://docs.oracle.com/javase/7/docs/api/java/io/DataOutput.html).
struct DataOutput<R: io::Write> {
    pub inner: R,
}

impl<R: io::Write> DataOutput<R> {
    pub fn new(writer: R) -> Self {
        Self { inner: writer }
    }

    /// Like Java's `writeBoolean`.
    pub fn write_bool(&mut self, x: bool) -> io::Result<()> {
        self.inner.write_u8(x as u8)
    }

    /// Like Java's `writeInt`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn write_u32(&mut self, x: u32) -> io::Result<()> {
        self.inner.write_i32::<BE>(
            x.try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
        )?;
        Ok(())
    }

    /// Like Java's `writeInt`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn write_usize(&mut self, x: usize) -> io::Result<()> {
        self.inner.write_i32::<BE>(
            x.try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
        )?;
        Ok(())
    }

    /// Like Java's `writeInt` but casts the result to `u32`, returning [`Option::None`] if the value is negative.
    pub fn write_u32_opt(&mut self, x: Option<u32>) -> io::Result<()> {
        let x = match x {
            Some(x) => x
                .try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
            None => -1,
        };
        self.inner.write_i32::<BE>(x)?;
        Ok(())
    }

    /// Like Java's `writeChar`. Writes a single UTF-16 code unit from a [`char`], returning
    /// [`std::io::ErrorKind::InvalidData`] if the [`char`] cannot be encoded as a single UTF-16 code unit.
    pub fn write_char(&mut self, x: char) -> io::Result<()> {
        let mut buf = [0u16; 2];
        let buf = x.encode_utf16(&mut buf);
        if buf.len() != 1 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        self.inner.write_u16::<BE>(buf[0])
    }

    /// Like Java's `writeUTF`. Writes a modified UTF-8 string with length. Returns [`std::io::ErrorKind::InvalidData`]
    /// if the string is longer than `u16::MAX`.
    pub fn write_string(&mut self, x: &str) -> io::Result<()> {
        let len: u16 = x
            .len()
            .try_into()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        self.inner.write_u16::<BE>(len)?;
        if len > 0 {
            let x = cesu8::to_java_cesu8(x);
            self.inner.write_all(&x)?;
        }
        Ok(())
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
