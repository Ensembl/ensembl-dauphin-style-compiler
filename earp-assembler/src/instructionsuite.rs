use std::collections::HashMap;

use crate::{error::EarpAssemblerError, instructionset::{EarpInstructionSet, EarpInstructionSetIdentifier}};

#[derive(Debug)]
struct EarpInstructionSuiteIncludedInstructionSet {
    identifier: EarpInstructionSetIdentifier,
    offset: u64
}

#[derive(Debug)]
pub(crate) struct EarpInstructionSuite {
    sets: Vec<EarpInstructionSuiteIncludedInstructionSet>,
    instructions: HashMap<Option<String>,EarpInstructionSet>,
    next_instr: u64
}

impl EarpInstructionSuite {
    pub(crate) fn new() -> EarpInstructionSuite {
        EarpInstructionSuite {
            sets: vec![],
            instructions: HashMap::new(),
            next_instr: 0
        }
    }

    pub(crate) fn add_instruction_set(&mut self, prefix: Option<&str>, src_set: &EarpInstructionSet) -> Result<(),EarpAssemblerError> {
        // XXX check dups
        // XXX anon sets
        let prefix = prefix.map(|x| x.to_string());
        let dst_set = self.instructions.entry(prefix).or_insert_with(|| 
            EarpInstructionSet::new(&EarpInstructionSetIdentifier("".to_string(),0))
        );
        let next_set_instr = src_set.next_value();
        for (name,opcode) in src_set.opcodes() {
            dst_set.add(&name,opcode+self.next_instr)?;
        }
        self.sets.push(EarpInstructionSuiteIncludedInstructionSet { identifier: src_set.identifier().clone(), offset: self.next_instr });
        self.next_instr += next_set_instr;
        Ok(())
    }

    pub(crate) fn lookup(&self, prefix: Option<&str>, opcode: &str) -> Result<u64,EarpAssemblerError> {
        if let Some(set) = self.instructions.get(&prefix.map(|x| x.to_string())) {
            set.lookup(opcode)
        } else {
            let name = if let Some(prefix) = prefix { format!("{}:{}",prefix,opcode) } else { opcode.to_string() };
            Err(EarpAssemblerError::UnknownOpcode(name))
        }
    }
}
