use std::{collections::HashMap};

use crate::instructionset::{EarpInstructionSetIdentifier, EarpInstructionSet};

pub(crate) struct Suite {
    sets: HashMap<EarpInstructionSetIdentifier,EarpInstructionSet>
}

impl Suite {
    pub(crate) fn new() -> Suite {
        Suite {
            sets: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, set: EarpInstructionSet) {
        self.sets.insert(set.identifier().clone(),set);
    }

    pub(crate) fn get(&self, id: &EarpInstructionSetIdentifier) -> Option<&EarpInstructionSet> {
        self.sets.get(id)
    }
}
