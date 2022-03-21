use std::sync::Arc;

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzleValue, ClonablePuzzleValue, PuzzleSolution};

use crate::{allotment::{core::rangeused::RangeUsed, transformers::transformers::{Transformer, DustbinTransformer}, style::{style::{LeafCommonStyle, LeafAllotmentStyle}}}, CoordinateSystem};

pub trait Coordinated {
    fn coordinate_system(&self) -> &CoordinateSystem;
}

pub trait Stackable : Coordinated {
    fn set_top(&self, value: &PuzzleValueHolder<f64>);
    fn height(&self) -> PuzzleValueHolder<f64>;

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64>;

    fn bottom_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        let mut piece = puzzle.new_piece();
        #[cfg(debug_assertions)]
        piece.set_name("bottom_anchor");
        let top = self.top_anchor(puzzle);
        let height = self.height();
        piece.add_solver(&[top.dependency(),height.dependency()],move |solution| {
            Some(top.get_clone(solution) + height.get_clone(solution))
        });
        PuzzleValueHolder::new(piece)
    }

    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>>;
}

pub trait StackableAddable {
    fn add_child(&mut self, child: &dyn Stackable, priority: i64);
}

pub trait Transformable {
    fn cloned(&self) -> Arc<dyn Transformable>;
    fn make(&self, solution: &PuzzleSolution) -> Arc<dyn Transformer>;
    fn get_style(&self) -> &LeafCommonStyle;
}

#[derive(Clone)]
pub struct DustbinTransformable(Arc<DustbinTransformer>);

impl DustbinTransformable {
    pub fn new() -> DustbinTransformable {
        DustbinTransformable(Arc::new(DustbinTransformer::new()))
    }
}

impl Transformable for DustbinTransformable {
    fn cloned(&self) -> Arc<dyn Transformable> { Arc::new(self.clone()) }
    fn make(&self, _solution: &PuzzleSolution) -> Arc<dyn Transformer> { self.0.clone() }
    fn get_style(&self) -> &LeafCommonStyle { self.0.get_style() }
}
