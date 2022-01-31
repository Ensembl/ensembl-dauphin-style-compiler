use std::fmt::Display;

use crate::{runtime::command::Command, core::error::EarpError};

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct InstructionSetId(pub String, pub u64);

impl Display for InstructionSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}

pub struct InstructionSet {
    id: InstructionSetId,
    commands: Vec<Option<Command>>
}

impl InstructionSet {
    pub fn new(id: &InstructionSetId) -> InstructionSet {
        InstructionSet {
            id: id.clone(),
            commands: vec![]
        }
    }

    pub fn id(&self) -> &InstructionSetId { &self.id }

    pub fn add(&mut self, offset: usize, command: Command) {
        while self.commands.len() <= offset {
            self.commands.push(None);
        }
        self.commands[offset] = Some(command);
    }

    pub fn get(&self, offset: usize) -> Option<&Command> {
        if offset >= self.commands.len() {
            return None;
        }
        self.commands[offset].as_ref()
    }

    pub fn merge(&mut self, other: &InstructionSet) -> Result<(),EarpError> {
        for (offset,command) in other.commands.iter().enumerate() {
            if let Some(command) = command {
                if self.commands.get(offset).is_some() {
                    return Err(EarpError::DuplicateInstruction(
                        format!("merging {} and {} at offset {}",self.id,other.id,offset)
                    ))
                }
                self.add(offset,command.clone());
            }
        }
        Ok(())
    }
}
