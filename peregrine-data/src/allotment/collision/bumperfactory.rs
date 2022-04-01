use std::collections::HashMap;

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder};

use crate::allotment::{style::allotmentname::AllotmentName};

use super::collisionalgorithm::CollisionAlgorithm;

pub struct BumperFactory {
    colliders: HashMap<AllotmentName,PuzzleValueHolder<CollisionAlgorithm>>
}

impl BumperFactory {
    pub fn new() -> BumperFactory {
        BumperFactory {
            colliders: HashMap::new()
        }
    }

    pub fn get(&mut self, puzzle: &PuzzleBuilder, name: &AllotmentName) -> &PuzzleValueHolder<CollisionAlgorithm> {
        self.colliders.entry(name.clone()).or_insert_with(|| {
            let piece = puzzle.new_piece();
            piece.add_solver(&[],|_| {
                Some(CollisionAlgorithm::new())
            });
            PuzzleValueHolder::new(piece)
        })
    }
}
