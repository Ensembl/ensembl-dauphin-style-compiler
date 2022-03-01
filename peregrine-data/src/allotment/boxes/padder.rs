use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzlePiece, DerivedPuzzlePiece, ClonablePuzzleValue, PuzzleValue, ConstantPuzzlePiece};

use crate::{allotment::{core::arbitrator::Arbitrator, style::style::Padding, boxes::boxtraits::Stackable}, AllotmentMetadata};

fn draw_top(top: &PuzzlePiece<f64>, padding_top: f64) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(top.clone(),move |top| *top + padding_top))
}

fn height(puzzle: &PuzzleBuilder, draw_top: &PuzzleValueHolder<f64>, child_height: &PuzzlePiece<f64>, min_height: f64, padding_bottom: f64) -> PuzzleValueHolder<f64> {
    let piece = puzzle.new_piece(None);
    let draw_top = draw_top.clone();
    let child_height = child_height.clone();
    piece.add_solver(&[draw_top.dependency(),child_height.dependency()], move |solution| {
        let internal_height = child_height.get_clone(solution).max(min_height);
        Some(draw_top.get_clone(solution) + internal_height + padding_bottom)
    });
    PuzzleValueHolder::new(piece)
}

fn indent(puzzle: &PuzzleBuilder, self_indent: f64, inherited_indent: &PuzzlePiece<f64>) -> PuzzleValueHolder<f64> {
    PuzzleValueHolder::new(DerivedPuzzlePiece::new(inherited_indent.clone(),move |v| v + self_indent))
}

pub struct Padder<T> {
    child: Box<T>,
    /* incoming variables */
    top: PuzzlePiece<f64>,
    inherited_indent: PuzzlePiece<f64>,
    self_indent: f64,
    /* outgoing variables */
    info: PadderInfo,
    height: PuzzleValueHolder<f64>
}

#[derive(Clone)]
pub struct PadderInfo {
    pub child_height: PuzzlePiece<f64>, /* children set this */
    pub draw_top: PuzzleValueHolder<f64>,
    pub indent: PuzzleValueHolder<f64>,
}

impl<T: Clone> Clone for Padder<T> {
    fn clone(&self) -> Self {
        Self {
            child: self.child.clone(),
            top: self.top.clone(),
            inherited_indent: self.inherited_indent.clone(),
            self_indent: self.self_indent.clone(),
            info: self.info.clone(),
            height: self.height.clone(),
        }
    }
}

impl<T> Padder<T> {
    pub fn new<F>(puzzle: &PuzzleBuilder, padding: &Padding, ctor: F) -> Padder<T> where F: FnOnce(&PadderInfo) -> T {
        let top = puzzle.new_piece(None);
        let padding_top = padding.padding_top;
        let padding_bottom = padding.padding_bottom;
        let min_height = padding.min_height;
        let inherited_indent = puzzle.new_piece(Some(0.));
        let self_indent = padding.indent;
        let draw_top = draw_top(&top,padding_top);
        let child_height = puzzle.new_piece(None);
        let height = height(&puzzle,&draw_top,&child_height,min_height,padding_bottom);
        let info = PadderInfo {
            draw_top, child_height,
            indent: indent(&puzzle,self_indent,&inherited_indent)
        };
        let child = ctor(&info);
        Padder {
            child: Box::new(child),
            top, inherited_indent, self_indent, height,
            info
        }
    }

    pub fn draw_top(&self) -> &PuzzleValueHolder<f64> { &self.info.draw_top }
    pub fn child(&self) -> &T { &self.child }
    pub fn child_mut(&mut self) -> &mut T { &mut self.child }
}

impl<T> Stackable for Padder<T> {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.top.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    fn height(&self) -> PuzzleValueHolder<f64> { self.height.clone() }

    fn set_indent(&self, value: &PuzzleValueHolder<f64>) {
        let value = value.clone();
        self.inherited_indent.add_solver(&[value.dependency()], move |solution| {
            Some(value.get_clone(solution))
        });
    }

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        PuzzleValueHolder::new(self.info.draw_top.clone())
    }
}
