use std::{sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{ConstantPuzzlePiece, PuzzleValueHolder, PuzzleBuilder, PuzzleSolution}, lock };

use crate::{ allotment::core::{playingfield::{PlayingFieldHolder, PlayingFieldPieces, PlayingField}, carriageuniverse::CarriageUniversePrep}};

use super::boxtraits::Stackable;

#[derive(Clone)]
pub struct Root {
    playing_field: Arc<Mutex<PlayingFieldHolder>>,
    children: Arc<Mutex<Vec<Box<dyn Stackable>>>>
}

impl Root {
    pub fn new(puzzle: &mut PuzzleBuilder) -> Root { 
        let playing_field = Arc::new(Mutex::new(PlayingFieldHolder::new(puzzle)));
        let playing_field2 = playing_field.clone();
        puzzle.add_ready(move |_| { lock!(playing_field2).ready(); });
        Root { playing_field, children: Arc::new(Mutex::new(vec![])) }
    }

    pub(crate) fn add_child(&self, child: &dyn Stackable) {
        lock!(self.children).push(child.cloned());
    }

    pub fn playing_field_pieces(&self) -> PlayingFieldPieces {
        PlayingFieldPieces::new(&&*lock!(self.playing_field))
    }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField {
        lock!(self.playing_field).get(solution)
    }

    pub(crate) fn build(&mut self, prep: &mut CarriageUniversePrep) {
        let mut children = lock!(self.children);
        for child in &mut *children {
            let build_size = child.build(prep);
            lock!(self.playing_field).set(child.coordinate_system(),&build_size.height);
        }
        for child in &mut *children {
            child.locate(prep,&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        }
    }
}
