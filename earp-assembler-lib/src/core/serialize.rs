use crate::{suite::assets::AssetFormat, AssetSource, assemble::parser::ParseOperand};

const TAB_OFFSETS : &[usize] = &[0,4,8];
const INLINE_COMMENT_OFFSET : usize = 40;
const LINE_LENGTH : usize = 70;

fn repeat(c: &str, num: usize) -> String {
    (0..num).map(|_| c).collect::<String>()
}

fn split_at_end(input: &str) -> Vec<String> {
    let mut out = vec![];
    for line in input.lines() {
        let mut all = line.to_string();
        while all.len() > LINE_LENGTH {
            if let Some(space_index) = &all[0..LINE_LENGTH].rfind(" ") {
                let (head,rest) = all.split_at(*space_index);
                out.push(head.trim().to_string());
                all = rest.trim().to_string();
            } else {
                out.push(all[0..LINE_LENGTH].trim().to_string());
                all = all[LINE_LENGTH..].trim().to_string();
            }
        }
        out.push(all.trim().to_string());    
    }
    println!("{:?} -> {:?}",input,out);
    out
}

#[derive(PartialEq,Eq)]
enum Phase {
    Decls,
    ProgramStart,
    Main,
    Any
}

impl Phase {
    fn compatible(&self, other: &Phase) -> bool {
        if let Phase::Any = self { return true; }
        if let Phase::Any = other { return true; }
        self == other
    }
}

#[derive(Debug)]
pub enum SerializeStatement {
    InstructionsDecl(Option<String>,String,u64),
    AssetDecl(String,AssetFormat,AssetSource,String),
    /**/
    Program(String),
    Instruction(Option<String>,String,Vec<ParseOperand>),
    Label(String),
    RelativeLabel(String),
    Noop,
    Include(String),
    /**/
    InlineComment(String), /* previous line */
    BlockComment(String),
    TopBlockComment(String),
    Blank
}

impl SerializeStatement {
    fn new_line(&self) -> bool {
        match self {
            SerializeStatement::Noop => false,
            SerializeStatement::InlineComment(_) => false,
            _ => true
        }
    }

    fn phase(&self) -> Phase {
        match self {
            SerializeStatement::InstructionsDecl(_, _, _) => Phase::Decls,
            SerializeStatement::AssetDecl(_, _, _, _) => Phase::Decls,
            SerializeStatement::Program(_) => Phase::ProgramStart,
            SerializeStatement::Noop => Phase::Any,
            SerializeStatement::Include(_) => Phase::Decls,
            SerializeStatement::InlineComment(_) => Phase::Any,
            SerializeStatement::TopBlockComment(_) => Phase::ProgramStart,
            SerializeStatement::Blank => Phase::Any,
            _ => Phase::Main,
        }
    }

    fn serialize(&self, line: &mut OutputLine) {
        match self {
            SerializeStatement::InstructionsDecl(prefix,name,version) => {
                if let Some(prefix) = prefix {
                    line.content(0,format!("$instructions {} {}/{}",prefix,name,version));
                } else {
                    line.content(0,format!("$instructions {}/{}",name,version));
                }
            },

            SerializeStatement::AssetDecl(id,format,source, path) => {
                line.content(0,format!("$asset {} {} {} {}",id,format,source,path));
            },

            SerializeStatement::Program(name) => {
                line.content(0,format!("program:{}:",name));
            },

            SerializeStatement::Instruction(prefix,name,operands) => {
                let mut out = if let Some(prefix) = prefix {
                    format!("{}.{}",prefix,name)
                } else {
                    name.to_string()
                };
                if operands.len() > 0 { out.push(' '); }
                out.push_str(&operands.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "));
                line.content(2,out);
            },

            SerializeStatement::Label(name) | SerializeStatement::RelativeLabel(name) => {
                line.content(1,format!("{}:",name));
            },

            SerializeStatement::Noop => {},

            SerializeStatement::Include(name) => {
                line.content(0,format!("$include {}",name));
            },

            SerializeStatement::InlineComment(comment) => {
                line.comment.get_or_insert("".to_string()).push_str(&format!(" {}",comment));
            },

            SerializeStatement::BlockComment(comment) => {
                for c in split_at_end(comment).iter() {
                    line.content(2,format!("; {}",c));
                }
            },

            SerializeStatement::TopBlockComment(comment) => {
                for c in split_at_end(comment) {
                    line.content(0,format!("; {}",c));
                }
            },

            SerializeStatement::Blank => {
                line.content(0,"".to_string());
            }
        }
    }
}

struct OutputLine {
    tabs: usize,
    content: Vec<String>,
    comment: Option<String>
}

impl OutputLine {
    fn new() -> OutputLine {
        OutputLine {
            tabs: 0,
            content: vec![],
            comment: None
        }
    }

    fn content(&mut self, tabs: usize, content: String) {
        self.tabs = tabs;
        self.content.push(content);
    }

    fn serialize(&self) -> Vec<String> {
        let empty = vec!["".to_string()];
        let the_content = if self.content.len() != 0 { &self.content } else { &empty };
        let mut out = vec![];
        for (i,line) in the_content.iter().enumerate() {
            let mut line_out = String::new();
            let initial_indent = TAB_OFFSETS[self.tabs.min(2)];
            line_out.push_str(&repeat(" ",initial_indent));
            line_out.push_str(line);
            if i == 0 {
                if let Some(comment) = &self.comment {
                    let inline_comment_pos = (line_out.len()+1).max(INLINE_COMMENT_OFFSET);
                    let inline_comment_spaces = inline_comment_pos - line_out.len();
                    line_out.push_str(&repeat(" ",inline_comment_spaces));
                    line_out.push_str(&format!(";{}",comment));
                }
            }
            out.push(line_out);
        }
        out
    }
}

pub fn serialize(statements: &[SerializeStatement]) -> String {
    let mut lines = vec![];
    let mut prev_phase = Phase::Any;
    for statement in statements {
        let phase = statement.phase();
        if statement.new_line() || lines.len() == 0 {
            if !phase.compatible(&prev_phase) {
                lines.push(OutputLine::new());
            }
            lines.push(OutputLine::new());
        }
        statement.serialize(lines.last_mut().unwrap());
        prev_phase = phase;
    }
    let mut lines_out = vec![];
    for output in &lines {
        let mut serials = output.serialize();
        lines_out.append(&mut serials);
    }
    lines_out.push("".to_string());
    lines_out.join("\n")
}

#[cfg(test)]
mod test {
    use std::{collections::hash_map::DefaultHasher, hash::Hash, iter::FromIterator};

    use rand::{prelude::SmallRng, SeedableRng, RngCore};

    use crate::{suite::assets::AssetFormat, AssetSource, assemble::parser::{ParseOperand, AssemblyLocation}};
    use super::{serialize, SerializeStatement, INLINE_COMMENT_OFFSET };

    #[test]
    fn serialize_smoke() {
        let input = vec![
            SerializeStatement::InstructionsDecl(None,"std".to_string(),0),
            SerializeStatement::AssetDecl("test".to_string(),AssetFormat::Raw,AssetSource::File,"raw-asset.bin".to_string()),
            SerializeStatement::TopBlockComment("test program".to_string()),
            SerializeStatement::Program("test".to_string()),
            SerializeStatement::Instruction(None,"halt".to_string(),vec![]),
            SerializeStatement::Noop,
            SerializeStatement::Instruction(None,"copy".to_string(),vec![
                ParseOperand::Register(0),
                ParseOperand::Float(1.)
            ]),
            SerializeStatement::BlockComment("comment".to_string()),
            SerializeStatement::Label("here".to_string()),
            SerializeStatement::Instruction(None,"copy".to_string(),vec![
                ParseOperand::UpRegister(1),
                ParseOperand::Boolean(true)
            ]),
            SerializeStatement::RelativeLabel("1".to_string()),
            SerializeStatement::Include("path.earp".to_string()),
            SerializeStatement::Instruction(None,"copy".to_string(),vec![
                ParseOperand::Location(AssemblyLocation::Here(0)),
                ParseOperand::Location(AssemblyLocation::RelativeLabel("1".to_string(),false))
            ]),
            SerializeStatement::Blank,
            SerializeStatement::Instruction(None,"copy".to_string(),vec![
                ParseOperand::Location(AssemblyLocation::Here(-1)),
                ParseOperand::Location(AssemblyLocation::Label("here".to_string()))
            ]),
            SerializeStatement::Instruction(None,"copy".to_string(),vec![
                ParseOperand::Location(AssemblyLocation::Here(1)),
                ParseOperand::Location(AssemblyLocation::RelativeLabel("1".to_string(),true))
            ]),
            SerializeStatement::InlineComment("comment".to_string()),
            SerializeStatement::InlineComment("another".to_string())
        ];
        let output = serialize(&input);
        println!("{}",output);
        assert_eq!(include_str!("../test/serialize/smoke.earp"),output);
    }

    const BASE_WORD : &str = "blobbyblahblahdoodah";

    fn word(len: usize) -> String {
        let mut out = String::new();
        while len - out.len() > BASE_WORD.len() {
            out.push_str(BASE_WORD);
        }
        out.push_str(&BASE_WORD[0..(len-out.len())]);
        out
    }

    fn words(range: (u32,u32), num: usize, seed: u64) -> String {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut out = String::new();
        for _ in 0..num {
            let len = if range.0 == range.1 {
                range.0
            } else {
                range.0 + (rng.next_u32() % (range.1-range.0))
            };
            out.push_str(&word(len as usize));
            out.push(' ');
        }
        out
    }

    fn no_whitespace(input: &str) -> String {
        String::from_iter(input.chars().filter(|x| !x.is_ascii_whitespace()))
    }

    fn linewrap_test(length: (u32,u32), cmp: &str) {
        let mut input = vec![];
        let mut all_the_words = String::new();
        for seed in 0..20 {
            let the_words = words(length,100,seed);
            println!("words {}",the_words);
            input.push(SerializeStatement::TopBlockComment(the_words.clone()));
            input.push(SerializeStatement::Blank);
            all_the_words.push_str(&the_words.trim());
        }
        let output = serialize(&input);
        //fs::write("/tmp/x",output.clone());
        assert_eq!(output,cmp);
        let mut remade = String::new();
        for line in output.lines() {
            let line = line.trim();
            let data = if line.starts_with("; ") { &line[2..] } else { &line };
            remade.push_str(data.trim());
            if !data.trim().is_empty() {
                remade.push(' ');
            }
        }
        println!("{}",output);
        assert_eq!(&no_whitespace(&remade),&no_whitespace(&all_the_words));
    }

    #[test]
    fn serialize_linewrap_smoke() {
        linewrap_test((1,10), include_str!("../test/serialize/wrapped.txt"));
    }

    #[test]
    fn serialize_linewrap_long() {
        linewrap_test((50,100), include_str!("../test/serialize/wrapped-long.txt"));
    }

    #[test]
    fn serialize_wrap_newlines() {
        let input = vec![
            SerializeStatement::TopBlockComment(include_str!("../test/serialize/wrap-newlines.txt").to_string())
        ];
        let output = serialize(&input);
        println!("{}",output);
        assert_eq!(output,include_str!("../test/serialize/wrap-newlines-out.earp"));
    }

    #[test]
    fn serialize_longlines() {
        const RANGE : usize = 10;
        const INSTR_LEN: usize = 19; // "copy r0, \"...\"" + 8 spaces
        let mut input = vec![];
        for delta in 0..(2*RANGE)+1 {
            input.push(
                SerializeStatement::Instruction(None,"copy".to_string(),vec![
                    ParseOperand::Register(0),
                    ParseOperand::String(word(INLINE_COMMENT_OFFSET+delta-RANGE-INSTR_LEN))
                ])
            );
            input.push(
                SerializeStatement::InlineComment("comment".to_string())
            );
        }
        let output = serialize(&input);
        println!("{}",output);
        assert_eq!(&output,include_str!("../test/serialize/longlines.earp"));
    }

    #[test]
    fn serialize_string() {
        const TESTS: &[&str] = &[
            "hello",
            "hello\n",
            "hello\r",
            "hello world",
            "hello\0world",
            "hello\u{20AC}world"
        ];
        let mut input = vec![];
        for test in TESTS {
            input.push(
                SerializeStatement::Instruction(None,"copy".to_string(),vec![
                    ParseOperand::Register(0),
                    ParseOperand::String(test.to_string())
                ])
            );
        }
        let output = serialize(&input);
        println!("{}",output);
        assert_eq!(&output,include_str!("../test/serialize/strings.earp"));
    }
}
