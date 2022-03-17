use std::{sync::{Arc, Mutex}, mem, cmp::max};

use peregrine_toolkit::{puzzle::{ConstantPuzzlePiece, PuzzleValueHolder, PuzzleBuilder, PuzzlePiece, PuzzleValue, PuzzleSolution, FoldValue}, lock, log};

use crate::{CoordinateSystemVariety, CoordinateSystem};

use super::boxtraits::Stackable;

#[derive(PartialEq,Clone,Debug)]
pub struct PlayingField2 {
    height: f64,
    squeeze: (f64,f64),
}

impl PlayingField2 {
    pub fn empty() -> PlayingField2 {
        PlayingField2 {
            height: 0.,
            squeeze: (0.,0.)
        }
    }

    pub fn union(&self, other: &PlayingField2) -> PlayingField2 {
        PlayingField2 {
            height: self.height.max(other.height),
            squeeze: (self.squeeze.0.max(other.squeeze.0),self.squeeze.1.max(other.squeeze.1))
        }
    }

    pub fn height(&self) -> f64 { self.height }
    pub fn squeeze(&self) -> (f64,f64) { self.squeeze }
}

fn new_max(puzzle: &mut PuzzleBuilder) -> FoldValue<f64> {
    let mut piece = puzzle.new_piece_default(0.);
    #[cfg(debug_assertions)]
    piece.set_name("new_max");
    FoldValue::new(piece,|a,b| a.max(b))
}

pub struct PlayingFieldHolder {
    top: FoldValue<f64>,
    bottom: FoldValue<f64>,
    left: FoldValue<f64>,
    right: FoldValue<f64>
}

impl PlayingFieldHolder {
    fn new(puzzle: &mut PuzzleBuilder) -> PlayingFieldHolder {
        PlayingFieldHolder {
            top: new_max(puzzle),
            bottom: new_max(puzzle),
            left: new_max(puzzle),
            right: new_max(puzzle),
        }
    }

    fn get(&self, solution: &PuzzleSolution) -> PlayingField2 {
        PlayingField2 {
            height: *self.top.get().get(solution) + *self.bottom.get().get(solution),
            squeeze: (*self.left.get().get(solution),*self.right.get().get(solution))
        }
    }

    fn set(&mut self, coord_system: &CoordinateSystem, value: &PuzzleValueHolder<f64>) {
        let var = match (&coord_system.0,coord_system.1) {
            (CoordinateSystemVariety::Tracking, false) => &mut self.top,
            (CoordinateSystemVariety::Tracking, true) => &mut self.bottom,
            (CoordinateSystemVariety::TrackingWindow, false) => &mut self.top,
            (CoordinateSystemVariety::TrackingWindow, true) => &mut self.bottom,
            (CoordinateSystemVariety::Sideways, false) => &mut self.left,
            (CoordinateSystemVariety::Sideways, true) => &mut self.right,
            _ => { return; }
        };
        var.add(value);
    }

    fn ready(&mut self) {
        self.top.build();
        self.bottom.build();
        self.left.build();
        self.right.build();
    }
}

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
        child.set_indent(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        child.set_top(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        lock!(self.playing_field).set(child.coordinate_system(),&child.height());
    }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField2 {
        lock!(self.playing_field).get(solution)
    }
}
