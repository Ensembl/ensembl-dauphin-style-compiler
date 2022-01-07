use std::{collections::{HashMap}};
use crate::{ command::EarpOperand, context::EarpAssemblerContext, earpfile::EarpFileWriter, error::EarpAssemblerError, parser::{EarpAssemblyLocation, EarpAssemblyOperand, EarpAssemblyStatement}, rellabels::RelativeLabelContext};

pub(crate) struct Assemble<'t> {
    context: &'t EarpAssemblerContext,
    pc: i64,
    max_pc: i64,
    earp_file: EarpFileWriter,
    labels: HashMap<String,i64>,
    rel_labels: RelativeLabelContext,
}

impl<'t> Assemble<'t> {
    fn new(context: &'t EarpAssemblerContext) -> Assemble<'t> {
        Assemble {
            context,
            pc: 0,
            max_pc: 0,
            earp_file: EarpFileWriter::new(),
            labels: HashMap::new(),
            rel_labels: RelativeLabelContext::new(),
        }
    }

    fn reset(&mut self) {
        self.pc = 0;
    }

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
                let opcode = self.context.instruction_suite().lookup(prefix,&identifier)?;
                let operands = arguments.iter().map(|x| EarpOperand::new(x,&self)).collect::<Result<Vec<_>,_>>()?;
                self.earp_file.add_instruction(opcode,&operands);
                self.pc += 1;
            },
            _ => {}
        }
        Ok(())
    }
    
    fn into_earpfile(self) -> EarpFileWriter {
        self.earp_file
    }
}

// XXX assets
// XXX check operand types
// XXX line numbers
pub(crate) fn assemble(context: &EarpAssemblerContext, statements: &[EarpAssemblyStatement]) -> Result<EarpFileWriter,EarpAssemblerError> {
    let mut assemble = Assemble::new(context);
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
