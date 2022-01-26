use std::{collections::HashMap};

use crate::{instructionset::{InstructionSetId, InstructionSet}, assets::{AssetSource, AssetLoader}, fileloader::FileLoader};

pub struct Suite {
    sets: HashMap<InstructionSetId,InstructionSet>,
    source_loader: FileLoader,
    asset_loaders: HashMap<AssetSource,Box<dyn AssetLoader>>
}

impl Suite {
    pub fn new() -> Suite {
        Suite {
            sets: HashMap::new(),
            source_loader: FileLoader::new(),
            asset_loaders: HashMap::new()
        }
    }

    pub fn add_instruction_set(&mut self, set: InstructionSet) {
        self.sets.insert(set.identifier().clone(),set);
    }

    pub(crate) fn get_instruction_set(&self, id: &InstructionSetId) -> Option<&InstructionSet> {
        self.sets.get(id)
    }

    pub fn add_loader<L>(&mut self, source: AssetSource, loader: L) where L: AssetLoader + 'static {
        self.asset_loaders.insert(source,Box::new(loader));
    }

    pub(crate) fn get_loader(&self, source: &AssetSource) -> Option<&Box<dyn AssetLoader>> {
        self.asset_loaders.get(source)
    }

    pub fn source_loader(&self) -> &FileLoader { &self.source_loader }
    pub fn source_loader_mut(&mut self) -> &mut FileLoader { &mut self.source_loader }
}
