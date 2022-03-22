use std::{sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{ConstantPuzzlePiece, PuzzleValueHolder, PuzzleBuilder, PuzzleSolution}, lock };

use crate::{ allotment::core::playingfield::{PlayingFieldHolder, PlayingFieldPieces, PlayingField}};

use super::boxtraits::Stackable;

#[derive(Clone)]
pub struct Root {
    playing_field: Arc<Mutex<PlayingFieldHolder>>
}

impl Root {
    pub fn new(puzzle: &mut PuzzleBuilder) -> Root { 
        let playing_field = Arc::new(Mutex::new(PlayingFieldHolder::new(puzzle)));
        let playing_field2 = playing_field.clone();
        puzzle.add_ready(move |_| { lock!(playing_field2).ready(); });
        Root { playing_field }
    }

    pub fn add_child(&self, child: &dyn Stackable) {
        child.set_top(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        lock!(self.playing_field).set(child.coordinate_system(),&child.height());
    }

    pub fn playing_field_pieces(&self) -> PlayingFieldPieces {
        PlayingFieldPieces::new(&&*lock!(self.playing_field))
    }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField {
        lock!(self.playing_field).get(solution)
    }
}
