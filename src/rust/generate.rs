use super::*;
use crate::java::multitrie::MultiTrie2;
use crate::java::serialize::{DataInput, JavaDeserialize};
use crate::java::trie::{Trie as JTrie, Cell as JCell, Row as JRow};
use std::collections::BTreeMap;
use std::fs;
use std::io;

struct RowBuilder {
    cells: BTreeMap<char, Cell>
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
    pub fn convert_java_multitrie(jmultitrie: &MultiTrie2) -> Self {
        let mut gen = Self::default();
        for jtrie in &jmultitrie.t.tries {
            gen.convert_java_trie(jtrie);
        }
        gen
    }

    fn convert_java_trie(&mut self, jtrie: &JTrie) {
        let mut trie = TrieBuilder { rows: Vec::with_capacity(jtrie.rows.len()) };
        
    }

    fn convert_java_command(&mut self, cmds: &str) -> CommandSlice {
        CommandSlice::new(1, 1)
    }
}

// pub fn generate_rust_table() -> Result<String, Box<dyn std::error::Error>> {
//     // Map from java command string to packed command index+len
//     let mut command_map = HashMap::new();
//     let mut commands = Vec::new();
//     let mut tries = Vec::new();
    
//     let path = "src/tables/stemmer_2000.out";

//     let mut reader = DataInput::new(io::BufReader::new(fs::File::open(path).unwrap()));
//     let multi = reader.read_string().unwrap();
//     let multi = multi.contains(['M', 'm']);
//     assert!(
//         multi,
//         "Expected stemmer table {} to contain a multitrie",
//         path
//     );
//     let java_data = MultiTrie2::deserialize(&mut reader)?;

//     for trie in &java_data.t.tries {
//         for row in &trie.rows {
//             for cell in row.cells.values() {
//                 if let Some(cmd_idx) = cell.cmd {
//                     let cmd = trie.cmds[cmd_idx as usize];
//                     if cmd == "*" {

//                     }
//                     let packed_idx = *command_map.entry(cmd).or_insert_with(|| {
//                         let len = cmd.chars().count();
//                         assert!(len > 0, "Empty Command");
//                         assert!(len % 2 == 0, "Command string not even in length");
//                         assert!(len / 2 <= 0xF, "Command length too long for packed index");
//                         let start_idx = commands.len() as u32;
//                         let packed_idx = (len as u32 / 2) | start_idx << 4;
//                         let mut cmd = cmd.chars();
//                         while let (Some(cmd), Some(param)) = (cmd.next(), cmd.next()) {
//                             let cmd = Command::parse(cmd, param).expect("Failed to parse command");
//                             commands.push(cmd);
//                         }
//                         packed_idx
//                     });
//                 }
//             }
//         }
//     }

//     Ok("foo".into())
// }
