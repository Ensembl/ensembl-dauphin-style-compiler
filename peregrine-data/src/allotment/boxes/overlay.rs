use std::sync::Arc;

use peregrine_toolkit::{puzzle::{StaticValue, commute_clonable, StaticAnswer}};
use crate::{allotment::{core::{ allotmentname::{AllotmentNamePart, AllotmentName}, boxtraits::{ContainerSpecifics, BuildSize, ContainerOrLeaf}, boxpositioncontext::BoxPositionContext}, style::{style::{ContainerAllotmentStyle}}, stylespec::stylegroup::AllStylesForProgram}, CoordinateSystem, LeafRequest};
use super::{container::{Container}, leaf::{AnchoredLeaf, FloatingLeaf}};

pub struct Overlay(Container);

impl Overlay {
    pub(crate) fn new(name: &AllotmentNamePart, style: &ContainerAllotmentStyle) -> Overlay {
        Overlay(Container::new(name,style,UnpaddedOverlay::new()))
    }
}

#[derive(Clone)]
struct UnpaddedOverlay {
}

impl UnpaddedOverlay {
    fn new() -> UnpaddedOverlay {
        UnpaddedOverlay {}
    }
}

impl ContainerOrLeaf for Overlay {
    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<AllStylesForProgram>) -> FloatingLeaf {
        self.0.get_leaf(pending,cursor,styles)
    }
    fn anchor_leaf(&self, answer_index: &StaticAnswer) -> Option<AnchoredLeaf> { None }
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
    fn locate(&self, prep: &mut BoxPositionContext, top: &StaticValue<f64>) { self.0.locate(prep,top); }
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn priority(&self) -> i64 { self.0.priority() }
    fn build(&self, prep: &mut BoxPositionContext) -> BuildSize { self.0.build(prep) }
}

impl ContainerSpecifics for UnpaddedOverlay {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&self, _prep: &mut BoxPositionContext, children: &[(&Box<dyn ContainerOrLeaf>,BuildSize)]) -> StaticValue<f64> {
        let heights = children.iter().map(|x| x.1.height.clone()).collect::<Vec<_>>();
        commute_clonable(&heights,0.,|a,b| f64::max(*a,*b))
    }

    fn set_locate(&self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn ContainerOrLeaf>]) {
        for child in children {
            child.locate(prep,&top);
        }
    }
}
