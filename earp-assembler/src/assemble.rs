use std::{collections::{HashMap}};
use crate::{ command::Operand, earpfile::EarpFileWriter, error::AssemblerError, parser::{AssemblyLocation, ParseStatement}, rellabels::RelativeLabelContext, suite::Suite, lookup::Lookup, instructionset::InstructionSetId};

pub(crate) struct Assemble<'t> {
    pc: i64,
    max_pc: i64,
    earp_file: EarpFileWriter<'t>,
    labels: HashMap<String,i64>,
    rel_labels: RelativeLabelContext,
    lookup: Lookup,
    file_path: Option<String>
}

impl<'t> Assemble<'t> {
    fn new(suite: &'t Suite, file_path: Option<&str>) -> Assemble<'t> {
        Assemble {
            pc: 0,
            max_pc: 0,
            earp_file: EarpFileWriter::new(suite),
            labels: HashMap::new(),
            rel_labels: RelativeLabelContext::new(),
            lookup: Lookup::new(),
            file_path: file_path.map(|x| x.to_string())
        }
    }

    fn reset(&mut self) { self.pc = 0; }

    pub(crate) fn resolve_label(&self, location: &AssemblyLocation) -> Result<i64,AssemblerError> {
        let label = match location {
            AssemblyLocation::Here(delta) => {
                let target = self.pc+delta;
                if target < 0 || target >= self.max_pc {
                    return Err(AssemblerError::BadHereLabel("Here label out of range".to_string()));
                }
                return Ok(target);
            },
            AssemblyLocation::Label(label) => {
                label.to_string()
            },
            AssemblyLocation::RelativeLabel(label,fwd) => {
                let suffix = if *fwd { "f" } else { "r" };
                format!("{}{}",label,suffix)
            }
        };
        if let Some(location) = self.labels.get(&label) {
            Ok(*location)
        } else {
            Err(AssemblerError::UnknownLabel(label))
        }
    }

    fn set_labels(&mut self, statement: &ParseStatement) -> Result<(),AssemblerError> {
        match statement {
            ParseStatement::Instruction(_,_,_) => {
                self.pc += 1;
                self.max_pc += 1;
            },
            ParseStatement::Program(program) => {
                if self.labels.contains_key(program) {
                    return Err(AssemblerError::DuplicateLabel(format!("program:{}",program)));
                }
                self.earp_file.add_entry_point(program,self.pc);
            },
            ParseStatement::Label(label) => {
                if self.labels.contains_key(label) {
                    return Err(AssemblerError::DuplicateLabel(label.to_string()))
                }
                self.labels.insert(label.to_string(),self.pc);
            },
            ParseStatement::RelativeLabel(label) => {
                self.rel_labels.add_label(self.pc,label);
            },
            _ => {}
        }
        Ok(())
    }

    fn add_instructions(&mut self, statement: &ParseStatement) -> Result<(),AssemblerError> {
        match statement {
            ParseStatement::Instruction(prefix,identifier,arguments) => {
                self.rel_labels.fix_labels(self.pc, &mut self.labels);
                let prefix = prefix.as_ref().map(|x| x.as_str());
                let opcode = self.lookup.lookup(self.earp_file.set_mapper_mut(),&prefix,&identifier)?;
                let operands = arguments.iter().map(|x| Operand::new(x,&self)).collect::<Result<Vec<_>,_>>()?;
                self.earp_file.add_instruction(opcode,&operands);
                self.pc += 1;
            },
            ParseStatement::InstructionsDecl(prefix,name,version) => {
                self.lookup.add_mapping(prefix, &InstructionSetId(name.to_string(),*version));
            },
            ParseStatement::AssetDecl(name,format,source,path) => {
                self.earp_file.assets_mut().add(name,format,source,path,&self.file_path)?;
            },
            _ => {}
        }
        Ok(())
    }
    
    fn into_earpfile(self) -> EarpFileWriter<'t> { self.earp_file }
}

fn assemble_instructions<'t>(suite: &'t Suite, statements: &[ParseStatement], file_path: Option<&str>) -> Result<Assemble<'t>,AssemblerError> {
    let mut assemble = Assemble::new(suite,file_path);
    assemble.reset();
    for stmt in statements {
        assemble.set_labels(stmt)?;
    }
    assemble.reset();
    for stmt in statements {
        assemble.add_instructions(stmt)?;
    }
    Ok(assemble)
}

// XXX include
// XXX assets
// XXX check operand types
// XXX line numbers
pub(crate) fn assemble<'t>(suite: &'t Suite, statements: &[ParseStatement], file_path: Option<&str>) -> Result<EarpFileWriter<'t>,AssemblerError> {
    Ok(assemble_instructions(suite,statements,file_path)?.into_earpfile())
}

#[cfg(test)]
mod test {
    use minicbor::Encoder;
    use peregrine_cli_toolkit::hexdump;

    use crate::{testutil::{no_error, yes_error, test_suite, build}, suite::Suite, opcodemap::load_opcode_map, parser::{earp_parse, load_source_file}, hexfile::load_hexfile, command::{Command, Operand}};

    use super::{assemble, assemble_instructions};

    #[test]
    fn assemble_smoke() {
        let suite = test_suite();
        let source = no_error(earp_parse(include_str!("test/test.earp")));
        let file = no_error(assemble(&suite,&source,None));
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&file));
        let cmp = no_error(load_hexfile(include_str!("test/assembler/smoke-earp.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }

    #[test]
    fn test_labels() {
        let suite = test_suite();
        let source = no_error(build(&suite,include_str!("test/assembler-labels/labels.earp")));
        assert_eq!(source,vec![
            Command(5, vec![]),
            Command(5, vec![]),
            Command(1, vec![Operand::Integer(1)])]);
    }

    #[test]
    fn test_labels_dup() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/dup.earp"))).to_string();
        assert!(err.to_lowercase().contains("duplicate label"));
    }

    #[test]
    fn test_labels_missing() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/missing.earp"))).to_string();
        assert!(err.to_lowercase().contains("unknown label"));
    }

    fn get_gotos(contents: &str) -> Vec<i64> {
        let suite = test_suite();
        let mut gotos = vec![];
        for cmd in no_error(build(&suite,contents)) {
            if cmd.0 == 1 {
                assert_eq!(cmd.1.len(),1);
                match cmd.1[0] {
                    Operand::Integer(v) => { gotos.push(v); },
                    _ => {}
                }
            }
        }
        gotos
    }

    #[test]
    fn test_rel_labels() {
        assert_eq!(vec![1,1,5,5,5,5,9,9],get_gotos(include_str!("test/assembler-labels/rel.earp")));
    }

    #[test]
    fn test_rel_labels_too_soon() {
        let suite = test_suite();
        yes_error(build(&suite,include_str!("test/assembler-labels/rel-too-soon.earp")));
    }

    #[test]
    fn test_rel_labels_too_late() {
        let suite = test_suite();
        yes_error(build(&suite,include_str!("test/assembler-labels/rel-too-late.earp")));
    }

    #[test]
    fn test_rel_labels_multi() {
        assert_eq!(vec![5,1,1,5,5,5,5,9,9,5],get_gotos(include_str!("test/assembler-labels/rel-multi.earp")));
    }

    #[test]
    fn test_here_labels() {
        assert_eq!(vec![0,2,0,3,5,3],get_gotos(include_str!("test/assembler-labels/here.earp")));
    }

    #[test]
    fn test_here_labels_start() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/here-start.earp"))).to_string();
        assert!(err.to_lowercase().contains("bad here"));
    }

    #[test]
    fn test_here_labels_end() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/here-end.earp"))).to_string();
        assert!(err.to_lowercase().contains("bad here"));
    }

    #[test]
    fn test_opcode_mapping() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("test/assembler/full-test.map"))) {
            suite.add_instruction_set(set);
        }
        let commands = no_error(build(&suite,include_str!("test/assembler/opcode-mapping.earp")));
        let mut opcodes= vec![];
        for Command(opcode,_) in &commands {
            opcodes.push(*opcode);
        }
        assert_eq!(vec![
                0,1,2,3,5,
                6,7,8,9,11,12,
                13,14,15],
            opcodes);
    }

    #[test]
    fn test_program_label() {
        let suite = test_suite();
        let source = no_error(load_source_file(include_str!("test/assembler-labels/labels-program.earp")));
        let earp_file = no_error(assemble_instructions(&suite,&source,None)).into_earpfile();
        let mut out = vec![];
        for (label,pc) in earp_file.entry_points() {
            out.push((label.to_string(),*pc));
        }
        out.sort();
        assert_eq!(vec![("test1".to_string(),0),("test2".to_string(),0),("test3".to_string(),1)],out);
    }

    fn unshape(mut v: u64) -> String {
        let mut out = String::new();
        loop {
            let c = match v%4 {
                1 => { "r" },
                2 => { "u" },
                3 => { "c" },
                _ => { break; },
            };
            out += c;
            v /= 4;
        }
        out.chars().rev().collect::<String>()
    }

    #[test]
    fn test_type_value() {
        let suite = test_suite();
        let source = no_error(build(&suite,include_str!("test/assembler/shape.earp")));
        let mut shapes = vec![];
        for shape in &source {
            shapes.push(unshape(shape.type_value()));
        }
        assert_eq!(vec![
            "c","c","c","c",

            "","r","u","c",

            "rr","ru","rc",
            "ur","uu","uc",
            "cr","cu","cc",

            "rrr","rru","rrc",
            "rur","ruu","ruc",
            "rcr","rcu","rcc",

            "urr","uru","urc",
            "uur","uuu","uuc",
            "ucr","ucu","ucc",

        ],shapes);        
    }

    #[test]
    fn test_collide_ok() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("test/assembler/collide.map"))) {
            suite.add_instruction_set(set);
        }
        no_error(build(&suite,include_str!("test/assembler/collide-ok.earp")));
    }

    #[test]
    fn test_collide_bad() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("test/assembler/collide.map"))) {
            suite.add_instruction_set(set);
        }
        let e = yes_error(build(&suite,include_str!("test/assembler/collide-bad.earp")));
        assert!(e.to_string().to_lowercase().contains("duplicate"));
    }    
}
