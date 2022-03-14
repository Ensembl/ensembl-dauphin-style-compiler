use std::sync::Arc;

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzleValue, ClonablePuzzleValue, PuzzleSolution};

use crate::{allotment::{core::rangeused::RangeUsed, transformers::transformers::Transformer}, CoordinateSystem};

pub trait Coordinated {
    fn coordinate_system(&self) -> &CoordinateSystem;
}

pub trait Stackable : Coordinated {
    fn set_top(&self, value: &PuzzleValueHolder<f64>);
    fn height(&self) -> PuzzleValueHolder<f64>;
    fn set_indent(&self, value: &PuzzleValueHolder<f64>);

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64>;

    fn bottom_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        let piece = puzzle.new_piece(None);
        let top = self.top_anchor(puzzle);
        let height = self.height();
        piece.add_solver(&[top.dependency(),height.dependency()],move |solution| {
            Some(top.get_clone(solution) + height.get_clone(solution))
        });
        PuzzleValueHolder::new(piece)
    }
}

pub trait Ranged {
    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>>;
}

pub trait Transformable {
    fn cloned(&self) -> Arc<dyn Transformable>;
    fn make(&self, solution: &PuzzleSolution) -> Arc<dyn Transformer>;
}