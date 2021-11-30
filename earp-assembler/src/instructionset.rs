use std::collections::HashMap;
use crate::error::EarpAssemblerError;

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub(crate) struct EarpInstructionSetIdentifier(pub String,pub u32);

#[derive(Debug)]
pub(crate) struct EarpInstructionSet {
    opcodes: HashMap<String,u64>,
    identifier: EarpInstructionSetIdentifier
}

impl EarpInstructionSet {
    pub(crate) fn new(identifier: &EarpInstructionSetIdentifier) -> EarpInstructionSet {
        EarpInstructionSet {
            opcodes: HashMap::new(),
            identifier: identifier.clone()
        }
    }

    pub(crate) fn add(&mut self, name: &str, opcode: u64) -> Result<(),EarpAssemblerError> {
        if self.opcodes.contains_key(name) {
            return Err(EarpAssemblerError::OpcodeInUse(name.to_string()))
        }
        self.opcodes.insert(name.to_string(),opcode);
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: &EarpInstructionSet) -> Result<(),EarpAssemblerError> {
        for (name,opcode) in other.opcodes() {
            self.add(name,opcode)?;
        }
        Ok(())
    }

    pub(crate) fn identifier(&self) -> &EarpInstructionSetIdentifier { &self.identifier }

    pub(crate) fn next_value(&self) -> u64 {
        self.opcodes.iter().map(|(_,v)| *v+1).max().unwrap_or(0)
    }

    pub(crate) fn opcodes(&self) -> impl Iterator<Item=(&str,u64)> {
        self.opcodes.iter().map(|(k,v)| (k.as_str(),*v))
    }

    pub(crate) fn lookup(&self, opcode: &str) -> Result<u64,EarpAssemblerError> {
        Ok(*self.opcodes.get(opcode).ok_or_else(|| EarpAssemblerError::UnknownOpcode(opcode.to_string()))?)
    }
}
