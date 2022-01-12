use std::{collections::HashMap};

use crate::instructionset::{InstructionSetId, InstructionSet};

pub(crate) struct Suite {
    sets: HashMap<InstructionSetId,InstructionSet>
}

impl Suite {
    pub(crate) fn new() -> Suite {
        Suite {
            sets: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, set: InstructionSet) {
        self.sets.insert(set.identifier().clone(),set);
    }

    pub(crate) fn get(&self, id: &InstructionSetId) -> Option<&InstructionSet> {
        self.sets.get(id)
    }
}
