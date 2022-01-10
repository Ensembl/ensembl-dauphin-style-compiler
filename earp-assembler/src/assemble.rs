use std::{collections::{HashMap}};
use crate::{ command::EarpOperand, earpfile::EarpFileWriter, error::EarpAssemblerError, parser::{EarpAssemblyLocation, EarpAssemblyStatement}, rellabels::RelativeLabelContext, suite::Suite, lookup::Lookup, instructionset::EarpInstructionSetIdentifier};

pub(crate) struct Assemble<'t> {
    pc: i64,
    max_pc: i64,
    earp_file: EarpFileWriter<'t>,
    labels: HashMap<String,i64>,
    rel_labels: RelativeLabelContext,
    lookup: Lookup
}

impl<'t> Assemble<'t> {
    fn new(suite: &'t Suite) -> Assemble<'t> {
        Assemble {
            pc: 0,
            max_pc: 0,
            earp_file: EarpFileWriter::new(suite),
            labels: HashMap::new(),
            rel_labels: RelativeLabelContext::new(),
            lookup: Lookup::new()
        }
    }

    fn reset(&mut self) { self.pc = 0; }

    pub(crate) fn resolve_label(&self, location: &EarpAssemblyLocation) -> Result<i64,EarpAssemblerError> {
        let label = match location {
            EarpAssemblyLocation::Here(delta) => {
                return Ok(self.pc+*delta);
            },
            EarpAssemblyLocation::Label(label) => {
                label.to_string()
            },
            EarpAssemblyLocation::RelativeLabel(label,fwd) => {
                let suffix = if *fwd { "f" } else { "r" };
                format!("{}{}",label,suffix)
            }
        };
        if let Some(location) = self.labels.get(&label) {
            Ok(*location)
        } else {
            Err(EarpAssemblerError::UnknownLabel(label))
        }
    }

    fn set_labels(&mut self, statement: &EarpAssemblyStatement) -> Result<(),EarpAssemblerError> {
        match statement {
            EarpAssemblyStatement::Instruction(_,_,_) => {
                self.pc += 1;
                self.max_pc += 1;
            },
            EarpAssemblyStatement::Program(program) => {
                if self.labels.contains_key(program) {
                    return Err(EarpAssemblerError::DuplicateLabel(format!("program:{}",program)));
                }
                self.earp_file.add_entry_point(program,self.pc);
            },
            EarpAssemblyStatement::Label(label) => {
                if self.labels.contains_key(label) {
                    return Err(EarpAssemblerError::DuplicateLabel(label.to_string()))
                }
                self.labels.insert(label.to_string(),self.pc);
            },
            EarpAssemblyStatement::RelativeLabel(label) => {
                self.rel_labels.add_label(self.pc,label);
            },
            _ => {}
        }
        Ok(())
    }

    fn add_instructions(&mut self, statement: &EarpAssemblyStatement) -> Result<(),EarpAssemblerError> {
        self.rel_labels.fix_labels(self.pc, &mut self.labels);
        match statement {
            EarpAssemblyStatement::Instruction(prefix,identifier,arguments) => {
                let prefix = prefix.as_ref().map(|x| x.as_str());
                let opcode = self.lookup.lookup(self.earp_file.set_mapper_mut(),&prefix,&identifier)?;
                let operands = arguments.iter().map(|x| EarpOperand::new(x,&self)).collect::<Result<Vec<_>,_>>()?;
                self.earp_file.add_instruction(opcode,&operands);
                self.pc += 1;
            },
            EarpAssemblyStatement::InstructionsDecl(prefix,name,version) => {
                self.lookup.add_mapping(prefix, &EarpInstructionSetIdentifier(name.to_string(),*version));
            },
            _ => {}
        }
        Ok(())
    }
    
    fn into_earpfile(self) -> EarpFileWriter<'t> { self.earp_file }
}

// XXX include
// XXX prefix collisions
// XXX assets
// XXX check operand types
// XXX line numbers
pub(crate) fn assemble<'t>(suite: &'t Suite, statements: &[EarpAssemblyStatement]) -> Result<EarpFileWriter<'t>,EarpAssemblerError> {
    let mut assemble = Assemble::new(suite);
    assemble.reset();
    for stmt in statements {
        assemble.set_labels(stmt)?;
    }
    assemble.reset();
    for stmt in statements {
        assemble.add_instructions(stmt)?;
    }
    Ok(assemble.into_earpfile())
}

#[cfg(test)]
mod test {
    use minicbor::Encoder;
    use peregrine_cli_toolkit::hexdump;

    use crate::{testutil::no_error, suite::Suite, opcodemap::load_opcode_map, parser::earp_parse, hexfile::load_hexfile};

    use super::assemble;

    #[test]
    fn assemble_smoke() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("maps/standard.map"))) {
            suite.add(set);
        }
        let source = no_error(earp_parse(include_str!("test/test.earp")));
        let file = no_error(assemble(&suite,&source));
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&file));
        let cmp = no_error(load_hexfile(include_str!("test/smoke-earp.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }
}
