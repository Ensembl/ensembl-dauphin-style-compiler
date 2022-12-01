use std::{sync::{Arc, Mutex}, collections::HashMap};
use peregrine_toolkit::{lock, puzzle::{constant, StaticValue, StaticAnswer}};
use crate::{ allotment::{core::{trainstate::CarriageTrainStateSpec, boxtraits::{ContainerOrLeaf, BuildSize }, boxpositioncontext::BoxPositionContext, allotmentname::AllotmentName}, util::rangeused::RangeUsed, stylespec::stylegroup::AllStylesForProgram}, CoordinateSystem, LeafRequest};

use super::{leaf::{AnchoredLeaf, FloatingLeaf}, container::HasKids};

#[derive(Clone)]
pub struct Root {
    root_name: AllotmentName,
    kids: HasKids
}

impl Root {
    pub fn new() -> Root { 
        Root {
            kids: HasKids::new(),
            root_name: AllotmentName::new("")
        }
    }

    pub(crate) fn full_build(&mut self, prep: &mut BoxPositionContext) -> CarriageTrainStateSpec {
        let mut children = lock!(self.kids.children);
        for child in children.values() {
            let build_size = child.build(prep);
            prep.state_request.playing_field_mut().set(child.coordinate_system(),build_size.height);
        }
        for child in children.values() {
            child.locate(prep,&constant(0.));
        }
        CarriageTrainStateSpec::new(&prep.state_request)
    }
}

impl ContainerOrLeaf for Root {
    /* these not used as we are root */
    fn priority(&self) -> i64 { 0 }
    fn coordinate_system(&self) -> &CoordinateSystem { &CoordinateSystem::Window }
    fn anchor_leaf(&self, answer_index: &StaticAnswer) -> Option<AnchoredLeaf> { None }

    fn build(&self, _prep: &mut BoxPositionContext) -> BuildSize {
        BuildSize {
            name: self.root_name.clone(),
            height: constant(0.),
            range: RangeUsed::All
        } 
    }

    fn locate(&self, _prep: &mut BoxPositionContext, _top: &StaticValue<f64>) {}
    fn name(&self) -> &AllotmentName { &self.root_name }
    fn cloned(&self) -> Box<dyn ContainerOrLeaf> { Box::new(self.clone()) }

    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<AllStylesForProgram>) -> FloatingLeaf {
        self.kids.get_leaf(pending,cursor,styles)
    }
}
