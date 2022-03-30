use std::sync::Arc;

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder, PuzzleValue, ClonablePuzzleValue, PuzzleSolution};

use crate::{allotment::{transformers::transformers::{Transformer}, style::{style::{LeafCommonStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, util::rangeused::RangeUsed, core::carriageuniverse::CarriageUniversePrep}, CoordinateSystem};

pub trait Coordinated {
    fn coordinate_system(&self) -> &CoordinateSystem;
}

pub(crate) trait ContainerSpecifics {
    fn cloned(&self) -> Box<dyn ContainerSpecifics>;
    fn build_reduce(&mut self, prep: &mut CarriageUniversePrep, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64>;
    fn set_locate(&mut self, prep: &mut CarriageUniversePrep, top: &PuzzleValueHolder<f64>, children: &mut [&mut Box<dyn Stackable>]);
}

pub(crate) struct BuildSize {
    pub name: AllotmentName,
    pub height: PuzzleValueHolder<f64>,
    pub range: PuzzleValueHolder<RangeUsed<f64>>
}

pub(crate) trait Stackable : Coordinated {
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize;
    fn locate(&mut self, prep: &mut CarriageUniversePrep, top: &PuzzleValueHolder<f64>);
    fn name(&self) -> &AllotmentName;
    fn priority(&self) -> i64;
    fn cloned(&self) -> Box<dyn Stackable>;
}

pub trait Transformable {
    fn cloned(&self) -> Arc<dyn Transformable>;
    fn make(&self, solution: &PuzzleSolution) -> Arc<dyn Transformer>;
    fn get_style(&self) -> &LeafCommonStyle;
}
