use std::sync::{Arc, Mutex};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, ClonablePuzzleValue, PuzzleValue, PuzzleBuilder, ConstantPuzzlePiece}, lock};

use crate::allotment::{style::style::Padding, boxes::boxtraits::Stackable, core::allotmentmetadata2::AllotmentMetadata2Builder};

use super::{padder::{Padder, PadderInfo}};

#[derive(Clone)]
pub struct Stacker(Padder<UnpaddedStacker>);

impl Stacker {
    pub fn new(puzzle: &PuzzleBuilder, padding: &Padding, metadata: &mut AllotmentMetadata2Builder) -> Stacker {
        Stacker(Padder::new(puzzle,padding,metadata,|info| UnpaddedStacker::new(puzzle,info)))
    }

    pub fn add_child(&mut self, child: &dyn Stackable) {
        self.0.child_mut().add_child(child)
    }
}

#[derive(Clone)]
struct UnpaddedStacker {
    puzzle: PuzzleBuilder,
    padder_info: PadderInfo,
    top: PuzzleValueHolder<f64>,
    current_height: Arc<Mutex<PuzzleValueHolder<f64>>>
}

impl UnpaddedStacker {
    fn new(puzzle: &PuzzleBuilder, padder_info: &PadderInfo) -> UnpaddedStacker {
        let top = padder_info.draw_top.clone();
        let current_height = Arc::new(Mutex::new(PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.))));
        let current_height2 = current_height.clone();
        let total_height = padder_info.child_height.clone();
        let total_height2 = total_height.clone();
        total_height.add_ready(move |_| {
            let current_height = lock!(current_height2).clone();
            total_height2.add_solver(&[current_height.dependency()], move |solution| {
                Some(current_height.get_clone(solution))
            })
        });
        UnpaddedStacker { puzzle: puzzle.clone(), padder_info: padder_info.clone(), current_height, top }
    }

    fn add_child(&mut self, child: &dyn Stackable) {
        let piece = self.puzzle.new_piece(None);
        let top = self.top.clone();
        let current_height = lock!(self.current_height).clone();
        piece.add_solver(&[current_height.dependency(),top.dependency()], move |solution| {
            Some(top.get_clone(solution) + current_height.get_clone(solution))
        });
        child.set_top(&PuzzleValueHolder::new(piece));
        child.set_indent(&PuzzleValueHolder::new(self.padder_info.indent.clone()));
        let child_height = child.height();
        let old_current_height = lock!(self.current_height).clone();
        let new_current_height = self.puzzle.new_piece(None);
        new_current_height.add_solver(&[child_height.dependency(),old_current_height.dependency()], move |solution| {
            Some(old_current_height.get_clone(solution) + child_height.get_clone(solution))
        });
        *lock!(self.current_height) = PuzzleValueHolder::new(new_current_height);
    }
}

impl Stackable for Stacker {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn height(&self) -> PuzzleValueHolder<f64> { self.0.height() }
    fn set_indent(&self, value: &PuzzleValueHolder<f64>) { self.0.set_indent(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
}
