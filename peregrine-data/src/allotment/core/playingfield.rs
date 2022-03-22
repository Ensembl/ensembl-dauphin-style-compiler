use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleSolution, PuzzleBuilder, FoldValue, PuzzlePiece, PuzzleValue};

use crate::{CoordinateSystemVariety, CoordinateSystem};

#[derive(PartialEq,Clone,Debug)]
pub struct PlayingField {
    height: f64,
    squeeze: (f64,f64),
}

impl PlayingField {
    pub fn empty() -> PlayingField {
        PlayingField {
            height: 0.,
            squeeze: (0.,0.)
        }
    }

    pub fn union(&self, other: &PlayingField) -> PlayingField {
        PlayingField {
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

#[derive(Clone)]
pub struct PlayingFieldPieces {
    pub top: PuzzlePiece<f64>,
    pub bottom: PuzzlePiece<f64>,
    pub left: PuzzlePiece<f64>,
    pub right: PuzzlePiece<f64>
}

impl PlayingFieldPieces {
    pub(crate) fn new(holder: &PlayingFieldHolder) -> PlayingFieldPieces {
        PlayingFieldPieces {
            top: holder.top.get().clone(),
            bottom: holder.bottom.get().clone(),
            left: holder.left.get().clone(),
            right: holder.right.get().clone(),
        }
    }
}

impl PlayingFieldHolder {
    pub(crate) fn new(puzzle: &mut PuzzleBuilder) -> PlayingFieldHolder {
        PlayingFieldHolder {
            top: new_max(puzzle),
            bottom: new_max(puzzle),
            left: new_max(puzzle),
            right: new_max(puzzle),
        }
    }

    pub(crate) fn get(&self, solution: &PuzzleSolution) -> PlayingField {
        PlayingField {
            height: *self.top.get().get(solution) + *self.bottom.get().get(solution),
            squeeze: (*self.left.get().get(solution),*self.right.get().get(solution))
        }
    }

    pub(crate) fn set(&mut self, coord_system: &CoordinateSystem, value: &PuzzleValueHolder<f64>) {
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

    pub(crate) fn ready(&mut self) {
        self.top.build();
        self.bottom.build();
        self.left.build();
        self.right.build();
    }
}
