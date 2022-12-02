use std::sync::Arc;

use peregrine_toolkit::{puzzle::{StaticValue, derived, StaticAnswer}};
use crate::{allotment::{util::rangeused::RangeUsed, core::{allotmentname::AllotmentName}, boxes::leaf::{AnchoredLeaf, FloatingLeaf}, stylespec::{styletree::StyleTree}}, CoordinateSystem, LeafRequest};
use super::{boxpositioncontext::BoxPositionContext};

pub(crate) trait ContainerSpecifics {
    fn build_reduce(&self, prep: &mut BoxPositionContext, children: &[(&Box<dyn ContainerOrLeaf>,BuildSize)]) -> StaticValue<f64>;
    fn set_locate(&self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn ContainerOrLeaf>]);
}

pub(crate) struct BuildSize {
    pub name: AllotmentName,
    pub height: StaticValue<f64>,
    pub range: RangeUsed<f64>
}

impl BuildSize {
    pub(crate) fn to_value(&self) -> StaticValue<(AllotmentName,f64,RangeUsed<f64>)> {
        let name = self.name.clone();
        let range = self.range.clone();
        derived(self.height.clone(),move |h| {
            (name.clone(),h,range.clone())
        })
    }
}

pub(crate) trait ContainerOrLeaf {
    fn coordinate_system(&self) -> &CoordinateSystem;
    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize;
    fn locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>);
    fn name(&self) -> &AllotmentName;
    fn priority(&self) -> i64;
    fn anchor_leaf(&self, answer_index: &StaticAnswer) -> Option<AnchoredLeaf>;
    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<StyleTree>) -> FloatingLeaf;
}
