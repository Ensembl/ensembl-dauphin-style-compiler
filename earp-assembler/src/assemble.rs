use std::{collections::{HashMap}};
use crate::{context::EarpAssemblerContext, error::EarpAssemblerError, parser::{EarpAssemblyLocation, EarpAssemblyOperand, EarpAssemblyStatement}, rellabels::RelativeLabelContext};

pub(crate) enum EarpOperand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl EarpOperand {
    fn new(value: &EarpAssemblyOperand, context: &Assemble) -> Result<EarpOperand,EarpAssemblerError> {
        Ok(match value {
            EarpAssemblyOperand::Register(r) => EarpOperand::Register(*r),
            EarpAssemblyOperand::UpRegister(r) => EarpOperand::UpRegister(*r),
            EarpAssemblyOperand::String(s) => EarpOperand::String(s.clone()),
            EarpAssemblyOperand::Boolean(b) => EarpOperand::Boolean(*b),
            EarpAssemblyOperand::Integer(n) => EarpOperand::Integer(*n),
            EarpAssemblyOperand::Float(f) => EarpOperand::Float(*f),
            EarpAssemblyOperand::Location(loc) => EarpOperand::Integer(context.resolve_label(loc)?)
        })
    }
}

struct Assemble<'t> {
    context: &'t EarpAssemblerContext,
    pc: i64,
    max_pc: i64,
    entry_points: HashMap<String,i64>,
    labels: HashMap<String,i64>,
    rel_labels: RelativeLabelContext,
    instructions: Vec<(u64,Vec<EarpOperand>)>
}

impl<'t> Assemble<'t> {
    fn new(context: &'t EarpAssemblerContext) -> Assemble<'t> {
        Assemble {
            context,
            pc: 0,
            max_pc: 0,
            entry_points: HashMap::new(),
            labels: HashMap::new(),
            rel_labels: RelativeLabelContext::new(),
            instructions: vec![]
        }
    }

    fn reset(&mut self) {
        self.pc = 0;
    }

    fn resolve_label(&self, location: &EarpAssemblyLocation) -> Result<i64,EarpAssemblerError> {
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
                self.entry_points.insert(program.to_string(),self.pc);
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
                self.instructions.push((opcode,operands));
                self.pc += 1;
            },
            _ => {}
        }
        Ok(())
    }

    fn into_instructions(self) -> Vec<(u64,Vec<EarpOperand>)> {
        self.instructions
    }
}

// XXX check operand types
// XXX line numbers
pub(crate) fn assemble(context: &EarpAssemblerContext, statements: &[EarpAssemblyStatement]) -> Result<Vec<(u64,Vec<EarpOperand>)>,EarpAssemblerError> {
    let mut assemble = Assemble::new(context);
    assemble.reset();
    for stmt in statements {
        assemble.set_labels(stmt)?;
    }
    assemble.reset();
    for stmt in statements {
        assemble.add_instructions(stmt)?;
    }
    Ok(assemble.into_instructions())
}
