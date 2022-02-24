use std::borrow::Borrow;

use peregrine_toolkit::puzzle::{Puzzle, PuzzleValueHolder, PuzzlePiece, ConstantPuzzlePiece, DerivedPuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleSolution};

use crate::{AllotmentMetadata, allotment::core::arbitrator::Arbitrator};

fn draw_top(top: &PuzzlePiece<f64>, padding_top: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| *top + padding_top))
}

fn draw_bottom(puzzle: &Puzzle, draw_top: &PuzzleValueHolder<f64>, final_height: &PuzzlePiece<f64>) -> PuzzleValueHolder<f64> {
    let piece = puzzle.new_piece(None);
    let draw_top = draw_top.clone();
    let final_height = final_height.clone();
    piece.add_solver(&[draw_top.dependency(),final_height.dependency()], move |solution| {
        Some(draw_top.get_clone(solution) + final_height.get_clone(solution))
    });
    PuzzleValueHolder::new(piece)
}

fn bottom(draw_bottom: &PuzzleValueHolder<f64>, padding_bottom: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(draw_bottom.clone(),move |draw_bottom| *draw_bottom + padding_bottom))
}

pub struct FloatingAllotmentBox {
    puzzle: Puzzle,
    /* incoming variables */
    top: PuzzlePiece<f64>,
    self_indent: Option<PuzzleValueHolder<f64>>,
    /* outgoing variables */
    draw_top: PuzzleValueHolder<f64>,
    draw_bottom: PuzzleValueHolder<f64>,
    bottom: PuzzleValueHolder<f64>,
    /* private outgoing variables */
    final_height: PuzzlePiece<f64>,
    /* build process */
    current_height: PuzzleValueHolder<f64>,

    /* constants */
    padding_top: f64,
    padding_bottom: f64,
    min_height: Option<f64>,
}

impl FloatingAllotmentBox {
    pub fn new(arbitrator: &Arbitrator, metadata: &AllotmentMetadata, initial_height: f64, self_indent: Option<&PuzzleValueHolder<f64>>) -> FloatingAllotmentBox {
        let puzzle = arbitrator.puzzle().clone();
        let top = puzzle.new_piece(None);
        let final_height = arbitrator.puzzle().new_piece(None);
        let padding_top = metadata.get_f64("padding-top").unwrap_or(0.);
        let padding_bottom = metadata.get_f64("padding-bottom").unwrap_or(0.);
        let draw_top = draw_top(&top,padding_top);
        let draw_bottom = draw_bottom(&puzzle,&draw_top,&final_height);
        let bottom = bottom(&draw_bottom,padding_bottom);
        let current_height = PuzzleValueHolder::new(ConstantPuzzlePiece::new(initial_height));
        let min_height = metadata.get_f64("min-height");
        FloatingAllotmentBox {
            puzzle: arbitrator.puzzle().clone(),
            top: top.clone(), 
            self_indent: self_indent.cloned(),
            draw_top, draw_bottom, bottom, final_height, current_height, padding_top, padding_bottom, min_height
        }
    }

    pub fn empty(arbitrator: &Arbitrator, initial_height: f64, self_indent: Option<&PuzzleValueHolder<f64>>) -> FloatingAllotmentBox {
        Self::new(arbitrator,&AllotmentMetadata::empty(),initial_height,self_indent)
    }

    pub fn top(&self) -> PuzzleValueHolder<f64> { PuzzleValueHolder::new(self.top.clone()) }
    pub fn draw_top(&self) -> PuzzleValueHolder<f64> { self.draw_top.clone() }
    pub fn draw_bottom(&self) -> PuzzleValueHolder<f64> { self.draw_bottom.clone() }
    pub fn bottom(&self) -> PuzzleValueHolder<f64> { self.bottom.clone() }

    pub fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    pub fn overlay(&mut self, child: &FloatingAllotmentBox) {
        child.set_top(&self.draw_top());
    }

    pub fn append(&mut self, child: &FloatingAllotmentBox) {
        /* set child's top to current height */
        let child_top = self.puzzle.new_piece(None);
        let our_top = self.top.clone();
        let current_height = self.current_height.clone();
        child_top.add_solver(&[our_top.dependency(),current_height.dependency()], move |solution| {
            Some(our_top.get_clone(solution) + current_height.get_clone(solution))
        });
        child.set_top(&PuzzleValueHolder::new(child_top.clone()));
        self.current_height = child.bottom();
    }
}

pub struct AnchoredAllotmentBox<'s,'a> {
    solution: &'s PuzzleSolution,
    floating: &'a FloatingAllotmentBox
}

impl<'s,'a> AnchoredAllotmentBox<'s,'a> {
    pub fn new(solution: &'s PuzzleSolution, floating: &'a FloatingAllotmentBox) -> AnchoredAllotmentBox<'s,'a> {
        AnchoredAllotmentBox { solution, floating }
    }

    pub fn top(&self) -> f64 { self.floating.top().get_clone(&self.solution) }
    pub fn draw_top(&self) -> f64 { self.floating.draw_top().get_clone(&self.solution) }
    pub fn draw_bottom(&self) -> f64 { self.floating.draw_bottom().get_clone(&self.solution) }
    pub fn bottom(&self) -> f64 { self.floating.bottom().get_clone(&self.solution) }
    pub fn indent(&self) -> f64 { 0. } // XXX todo
}
