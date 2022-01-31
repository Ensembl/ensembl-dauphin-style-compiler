use std::collections::HashMap;

use crate::runtime::command::Command;

use super::instructionset::{InstructionSetId, InstructionSet};

pub struct Suite {
    instruction_sets: HashMap<InstructionSetId,InstructionSet>
}

impl Suite {
    pub fn new() -> Suite {
        Suite {
            instruction_sets: HashMap::new()
        }
    }

    pub fn add_instruction_set(&mut self, set: InstructionSet) {
        self.instruction_sets.insert(set.id().clone(),set);
    }

    pub fn lookup(&self, set: &InstructionSetId, offset: u64) -> Option<&Command> {
        self.instruction_sets.get(set).and_then(|s| s.get(offset as usize))
    }
}
