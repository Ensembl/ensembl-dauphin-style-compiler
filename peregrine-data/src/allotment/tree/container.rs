use std::{borrow::Borrow, sync::Arc};

use peregrine_toolkit::puzzle::{Puzzle, PuzzleValueHolder, PuzzlePiece, ConstantPuzzlePiece, DerivedPuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleSolution, PuzzleBuilder};

use crate::{AllotmentMetadata, allotment::{core::arbitrator::Arbitrator, boxes::boxtraits::Stackable}, CoordinateSystem, SpaceBase, PartialSpaceBase, SpaceBaseArea, DataFilter};

use super::{allotmentbox::AllotmentBox};

fn draw_top(top: &PuzzlePiece<f64>, padding_top: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| *top + padding_top))
}

fn total_height(puzzle: &PuzzleBuilder, draw_top: &PuzzleValueHolder<f64>, final_height: &PuzzlePiece<f64>, min_height: Option<f64>, padding_bottom: f64) -> PuzzleValueHolder<f64> {
    let piece = puzzle.new_piece(None);
    let draw_top = draw_top.clone();
    let final_height = final_height.clone();
    piece.add_solver(&[draw_top.dependency(),final_height.dependency()], move |solution| {
        let internal_height = final_height.get_clone(solution).max(min_height.unwrap_or(0.));
        Some(draw_top.get_clone(solution) + internal_height + padding_bottom)
    });
    PuzzleValueHolder::new(piece)
}

fn indent(puzzle: &PuzzleBuilder, self_indent: &PuzzleValueHolder<f64>, inherited_indent: &PuzzlePiece<f64>) -> PuzzlePiece<f64> {
    let piece = puzzle.new_piece(Some(0.));
    let self_indent = self_indent.clone();
    let inherited_indent = inherited_indent.clone();
    piece.add_solver(&[self_indent.dependency()],move |solution| {
        Some(self_indent.get_clone(solution) + inherited_indent.get_clone(solution))
    });
    piece
}

pub struct FloatingContainer {
    puzzle: PuzzleBuilder,
    /* incoming variables */
    top: PuzzlePiece<f64>,
    inherited_indent: PuzzlePiece<f64>,
    self_indent: PuzzleValueHolder<f64>,
    /* outgoing variables */
    draw_top: PuzzleValueHolder<f64>,
    indent: PuzzlePiece<f64>,
    total_height: PuzzleValueHolder<f64>,
    /* private outgoing variables */
    final_height: PuzzlePiece<f64>,
    /* build process */
    current_height: PuzzleValueHolder<f64>,
}

impl FloatingContainer {
    pub fn new(arbitrator: &Arbitrator, metadata: &AllotmentMetadata, initial_height: f64, self_indent: Option<&PuzzleValueHolder<f64>>) -> FloatingContainer {
        let puzzle = arbitrator.puzzle().clone();
        let top = puzzle.new_piece(None);
        let final_height = arbitrator.puzzle().new_piece(None);
        let padding_top = metadata.get_f64("padding-top").unwrap_or(0.);
        let padding_bottom = metadata.get_f64("padding-bottom").unwrap_or(0.);
        let draw_top = draw_top(&top,padding_top);
        let current_height = PuzzleValueHolder::new(ConstantPuzzlePiece::new(initial_height));
        let min_height = metadata.get_f64("min-height");
        let total_height = total_height(&puzzle,&draw_top,&final_height,min_height,padding_bottom);
        let inherited_indent = puzzle.new_piece(Some(0.));
        let self_indent = self_indent.cloned().unwrap_or_else(||
            PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.))
        );
        let indent = indent(&puzzle,&self_indent,&&inherited_indent);
        FloatingContainer {
            puzzle: arbitrator.puzzle().clone(),
            top: top.clone(), 
            draw_top, final_height, current_height,
            total_height, self_indent, inherited_indent,
            indent
        }
    }

    pub fn overlay(&mut self, child: &dyn Stackable) {
        child.set_top(&self.draw_top.clone());
        child.set_indent(&PuzzleValueHolder::new(self.indent.clone()));
    }

    pub fn append(&mut self, child: &dyn Stackable) {
        /* set child's top to current height */
        let child_top = self.puzzle.new_piece(None);
        let our_top = self.draw_top.clone();
        let current_height = self.current_height.clone();
        child_top.add_solver(&[our_top.dependency(),current_height.dependency()], move |solution| {
            Some(our_top.get_clone(solution) + current_height.get_clone(solution))
        });
        child.set_top(&PuzzleValueHolder::new(child_top.clone()));
        child.set_indent(&PuzzleValueHolder::new(self.indent.clone()));
        let child_height = child.height();
        let old_height = self.current_height.clone();
        let new_height = self.puzzle.new_piece(None);
        new_height.add_solver(&[child_height.dependency(),old_height.dependency()], move |solution| {
            Some(old_height.get_clone(solution)+child_height.get_clone(solution))
        });
        self.current_height = PuzzleValueHolder::new(new_height);
    }
}

impl Stackable for FloatingContainer {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    fn height(&self) -> PuzzleValueHolder<f64> { self.total_height.clone() }

    fn set_indent(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.inherited_indent.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.top.clone())
    }
}
