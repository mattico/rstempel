use super::*;
use crate::java::multitrie::MultiTrie2;
use crate::java::serialize::JavaDeserialize;
use crate::java::trie::{Row as JRow, Trie as JTrie};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

#[derive(Default)]
struct RowBuilder {
    cells: BTreeMap<char, Cell>,
}

struct TrieBuilder {
    rows: Vec<RowBuilder>,
}

#[derive(Default)]
pub struct RustGenerator {
    commands: Vec<Command>,
    command_map: HashMap<String, CommandSlice>,
    tries: Vec<TrieBuilder>,
}

impl RustGenerator {
    pub fn convert_java_table(input: &Path) -> io::Result<()> {
        use crate::java::serialize::DataInput;
        use std::fs;

        let output = input.with_extension("rs");

        let input = fs::File::open(input)?;
        let input = io::BufReader::new(input);
        let mut input = DataInput::new(input);
        let _ = input.read_string()?;
        let input = MultiTrie2::deserialize(&mut input)?;

        let output = fs::File::create(output)?;
        let output = io::BufWriter::new(output);

        let gen = Self::convert_java_multitrie(&input);
        gen.write_rust_table(output)?;

        Ok(())
    }

    pub fn convert_java_multitrie(jmultitrie: &MultiTrie2) -> Self {
        let mut gen = Self::default();
        for jtrie in &jmultitrie.t.tries {
            gen.convert_java_trie(jtrie);
        }
        gen
    }

    fn convert_java_trie(&mut self, jtrie: &JTrie) {
        let mut trie = TrieBuilder {
            rows: Vec::with_capacity(jtrie.rows.len()),
        };
        for ele in &jtrie.cmds {
            self.convert_java_command(ele);
        }
        for jrow in &jtrie.rows {
            let row = self.convert_java_row(jtrie, jrow);
            trie.rows.push(row);
        }
        self.tries.push(trie);
    }

    fn convert_java_command(&mut self, cmds: &str) -> CommandSlice {
        if cmds == "*" {
            return CommandSlice::new_eom();
        } else if let Some(&cs) = self.command_map.get(cmds) {
            return cs;
        }
        debug_assert!(cmds.chars().count() % 2 == 0);
        let mut chars = cmds.chars();
        let idx = self.commands.len();
        while let (Some(cmd), Some(param)) = (chars.next(), chars.next()) {
            let cmd = Command::parse(cmd, param)
                .unwrap_or_else(|| panic!("Unable to parse command: \"{}{}\"", cmd, param));
            self.commands.push(cmd);
        }
        let len = self.commands.len() - idx;
        let cs = CommandSlice::new(idx, len);
        self.command_map.insert(cmds.into(), cs);
        cs
    }

    fn convert_java_row(&mut self, jtrie: &JTrie, row: &JRow) -> RowBuilder {
        let mut result = RowBuilder::default();
        for (&ch, cell) in &row.cells {
            let refr = cell.refr.map(|r| {
                NonZeroU16::new(r.try_into().expect("Row index did not fit in u16")).unwrap()
            });
            let cmds = cell
                .cmd
                .and_then(|idx| jtrie.cmds.get(idx as usize))
                .and_then(|cmd| self.command_map.get(cmd))
                .cloned();
            result.cells.insert(ch, Cell { refr, cmds });
        }
        result
    }

    pub fn write_rust_table(&self, mut out: impl io::Write) -> io::Result<()> {
        use std::mem::size_of;
        writeln!(out, "use std::num::{{NonZeroU16, NonZeroU32}};")?;
        writeln!(
            out,
            "use crate::rust::{{Cell, Command, CommandSlice, Row, Stemmer, Trie}};\n"
        )?;
        let num_rows: usize = self.tries.iter().map(|t| t.rows.len()).sum();
        let num_cells: usize = self
            .tries
            .iter()
            .flat_map(|t| &t.rows)
            .map(|r| r.cells.len())
            .sum();
        let size = size_of::<Stemmer>()
            + self.tries.len() * size_of::<Trie>()
            + num_rows * size_of::<Row>()
            + num_cells * (size_of::<Cell>() + size_of::<char>())
            + self.commands.len() * size_of::<Command>();
        writeln!(out, "// approximate size: {} bytes", size)?;
        writeln!(out, "pub static STEMMER: Stemmer = Stemmer {{")?;
        writeln!(out, "commands: &[")?;
        for command in &self.commands {
            Self::write_rust_command(&mut out, command)?;
        }
        writeln!(out, "],")?;
        writeln!(out, "tries: &[")?;
        for trie in &self.tries {
            Self::write_rust_trie(&mut out, trie)?;
        }
        writeln!(out, "],")?;
        writeln!(out, "}};")?;
        Ok(())
    }

    fn write_rust_command(mut out: impl io::Write, cmd: &Command) -> io::Result<()> {
        match cmd {
            Command::Skip { chars } => {
                writeln!(out, "Command::Skip {{ chars: {} }},", chars)
            }
            Command::Delete { chars } => {
                writeln!(out, "Command::Delete {{ chars: {} }},", chars)
            }
            Command::Replace { char } => {
                writeln!(out, "Command::Replace {{ char: '{}' }},", char)
            }
            Command::Insert { char } => {
                writeln!(out, "Command::Insert {{ char: '{}' }},", char)
            }
        }
    }

    fn write_rust_trie(mut out: impl io::Write, trie: &TrieBuilder) -> io::Result<()> {
        writeln!(out, "Trie {{ rows: &[")?;
        for row in &trie.rows {
            Self::write_rust_row(&mut out, row)?;
        }
        writeln!(out, "] }},")?;
        Ok(())
    }

    fn write_rust_row(mut out: impl io::Write, row: &RowBuilder) -> io::Result<()> {
        writeln!(out, "Row {{")?;
        writeln!(out, "cells: &[")?;
        for cell in row.cells.values() {
            Self::write_rust_cell(&mut out, cell)?;
        }
        writeln!(out, "],")?;
        writeln!(out, "chars: &[")?;
        for (idx, &ch) in row.cells.keys().enumerate() {
            write!(out, "'{}', ", ch)?;
            if idx % 16 == 0 {
                writeln!(out)?;
            }
        }
        writeln!(out, "],")?;
        writeln!(out, "}},")?;
        Ok(())
    }

    fn write_rust_cell(mut out: impl io::Write, cell: &Cell) -> io::Result<()> {
        write!(out, "Cell {{ refr: ")?;
        match cell.refr {
            Some(refr) => write!(
                out,
                "Some(unsafe {{ NonZeroU16::new_unchecked({}) }})",
                refr
            )?,
            None => write!(out, "None")?,
        };
        write!(out, ", cmds: ")?;
        match cell.cmds {
            Some(cmds) => write!(
                out,
                "Some(CommandSlice(unsafe {{ NonZeroU32::new_unchecked({}) }}))",
                cmds.0.get()
            )?,
            None => write!(out, "None")?,
        };
        writeln!(out, " }},")?;
        Ok(())
    }
}
