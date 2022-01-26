use crate::{parser::ParseOperand, assets::AssetFormat, AssetSource};

const TAB_OFFSETS : &[usize] = &[0,4,8];
const INLINE_COMMENT_OFFSET : usize = 40;
const LINE_LENGTH : usize = 70;

fn repeat(c: &str, num: usize) -> String {
    (0..num).map(|_| c).collect::<String>()
}

fn split_at_end(c: &str) -> Vec<String> {
    let mut c = c.to_string();
    let mut out = vec![];
    while c.len() > LINE_LENGTH {
        if let Some(space_index) = &c[0..LINE_LENGTH].rfind(" ") {
            let (line,rest) = c.split_at(*space_index);
            out.push(line.trim().to_string());
            c = rest.trim().to_string();
        } else {
            out.push(c[0..LINE_LENGTH].trim().to_string());
            c = c[LINE_LENGTH..].trim().to_string();
        }
    }
    out.push(c);
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
            SerializeStatement::Noop => true,
            SerializeStatement::InlineComment(_) => true,
            _ => false
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
                line.comment.get_or_insert("".to_string()).push_str(comment);
            },

            SerializeStatement::BlockComment(comment) => {
                for c in split_at_end(comment) {
                    line.content(1,format!("; {}",c));
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
    content: String,
    comment: Option<String>
}

impl OutputLine {
    fn new() -> OutputLine {
        OutputLine {
            tabs: 0,
            content: "".to_string(),
            comment: None
        }
    }

    fn content(&mut self, tabs: usize, content: String) {
        self.tabs = tabs;
        self.content = content;
    }

    fn serialize(&self) -> String {
        let mut out = String::new();
        let initial_indent = TAB_OFFSETS[self.tabs.min(2)];
        out.push_str(&repeat(" ",initial_indent));
        out.push_str(&self.content);
        if let Some(comment) = &self.comment {
            let inline_comment_pos = (out.len()+1).max(INLINE_COMMENT_OFFSET);
            let inline_comment_spaces = inline_comment_pos - out.len();
            out.push_str(&repeat(" ",inline_comment_spaces));
            out.push_str(&format!("; {}",comment));
        }
        out.push('\n');
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
                lines.push(OutputLine::new());
            }
            lines.push(OutputLine::new());
        }
        statement.serialize(lines.last_mut().unwrap());
        prev_phase = phase;
    }
    lines.iter().map(|line| line.serialize()).collect::<Vec<_>>().join("\n")
}
