use std::{collections::HashMap};

use crate::{instructionset::{InstructionSetId, InstructionSet}, assets::{AssetSource, AssetLoader}};

pub(crate) struct Suite {
    sets: HashMap<InstructionSetId,InstructionSet>,
    asset_loaders: HashMap<AssetSource,Box<dyn AssetLoader>>
}

impl Suite {
    pub(crate) fn new() -> Suite {
        Suite {
            sets: HashMap::new(),
            asset_loaders: HashMap::new()
        }
    }

    pub(crate) fn add_instruction_set(&mut self, set: InstructionSet) {
        self.sets.insert(set.identifier().clone(),set);
    }

    pub(crate) fn get_instruction_set(&self, id: &InstructionSetId) -> Option<&InstructionSet> {
        self.sets.get(id)
    }

    pub(crate) fn add_loader<L>(&mut self, source: AssetSource, loader: L) where L: AssetLoader + 'static {
        self.asset_loaders.insert(source,Box::new(loader));
    }

    pub(crate) fn get_loader(&self, source: &AssetSource) -> Option<&Box<dyn AssetLoader>> {
        self.asset_loaders.get(source)
    }
}
