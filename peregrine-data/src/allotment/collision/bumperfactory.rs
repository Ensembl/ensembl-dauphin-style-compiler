use std::{collections::HashMap, sync::Arc};

use peregrine_toolkit::{puzzle::{short_memoized, variable, StaticValue}};

use crate::allotment::{style::allotmentname::AllotmentName};

use super::collisionalgorithm::CollisionAlgorithm;

pub struct BumperFactory {
    colliders: HashMap<AllotmentName,StaticValue<Arc<CollisionAlgorithm>>>
}

impl BumperFactory {
    pub fn new() -> BumperFactory {
        BumperFactory {
            colliders: HashMap::new()
        }
    }

    pub fn get(&mut self, name: &AllotmentName) -> StaticValue<Arc<CollisionAlgorithm>> {
        self.colliders.entry(name.clone()).or_insert_with(|| {
            short_memoized(variable(|answer_index| {
                CollisionAlgorithm::new()
            }))
        }).clone()
    }
}
