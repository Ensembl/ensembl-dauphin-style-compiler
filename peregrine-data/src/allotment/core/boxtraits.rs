use std::sync::Arc;

use peregrine_toolkit::{puzzle::{StaticValue, StaticAnswer, compose}};

use crate::{allotment::{transformers::transformers::{Transformer}, style::{style::{LeafStyle}}, util::rangeused::RangeUsed, core::{allotmentname::AllotmentName}}, CoordinateSystem};

use super::boxpositioncontext::BoxPositionContext;

pub trait Coordinated {
    fn coordinate_system(&self) -> &CoordinateSystem;
}

pub(crate) trait ContainerSpecifics {
    fn cloned(&self) -> Box<dyn ContainerSpecifics>;
    fn build_reduce(&mut self, prep: &mut BoxPositionContext, children: &[(&Box<dyn Stackable>,BuildSize)]) -> StaticValue<f64>;
    fn set_locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn Stackable>]);
}

pub(crate) struct BuildSize {
    pub name: AllotmentName,
    pub height: StaticValue<f64>,
    pub range: StaticValue<RangeUsed<f64>>
}

impl BuildSize {
    pub(crate) fn to_value(&self) -> StaticValue<(AllotmentName,f64,RangeUsed<f64>)> {
        let name = self.name.clone();
        compose(self.height.clone(),self.range.clone(),move |h,r| {
            (name.clone(),h,r)
        })
    }
}

pub(crate) trait Stackable : Coordinated {
    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize;
    fn locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>);
    fn name(&self) -> &AllotmentName;
    fn priority(&self) -> i64;
    fn cloned(&self) -> Box<dyn Stackable>;
}

pub trait Transformable {
    fn cloned(&self) -> Arc<dyn Transformable>;
    fn make(&self, solution: &StaticAnswer) -> Arc<dyn Transformer>;
    fn get_style(&self) -> &LeafStyle;
}
