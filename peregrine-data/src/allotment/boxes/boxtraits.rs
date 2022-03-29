use std::sync::Arc;

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzleValue, ClonablePuzzleValue, PuzzleSolution};

use crate::{allotment::{transformers::transformers::{Transformer}, style::{style::{LeafCommonStyle}}, util::rangeused::RangeUsed, core::carriageuniverse::CarriageUniversePrep}, CoordinateSystem};

pub trait Coordinated {
    fn coordinate_system(&self) -> &CoordinateSystem;
}

pub(crate) struct BuildSize {
    pub height: PuzzleValueHolder<f64>,
    pub range: PuzzleValueHolder<RangeUsed<f64>>
}

pub(crate) trait Stackable : Coordinated {
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize;

    fn set_top(&self, value: &PuzzleValueHolder<f64>);
    fn priority(&self) -> i64;

    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64>;

    fn cloned(&self) -> Box<dyn Stackable>;
}

pub trait Transformable {
    fn cloned(&self) -> Arc<dyn Transformable>;
    fn make(&self, solution: &PuzzleSolution) -> Arc<dyn Transformer>;
    fn get_style(&self) -> &LeafCommonStyle;
}
