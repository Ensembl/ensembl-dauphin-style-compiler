use std::{sync::{Arc}};
use peregrine_toolkit::{puzzle::{constant, StaticValue, StaticAnswer}};
use crate::{ allotment::{core::{allotmentname::AllotmentName, rangeused::RangeUsed}, style::styletree::StyleTree, leafs::{floating::FloatingLeaf, anchored::AnchoredLeaf}, layout::{layouttree::{ContainerOrLeaf}, layoutcontext::LayoutContext, contentsize::ContentSize}}, CoordinateSystem, LeafRequest, globals::trainstate::CarriageTrainStateSpec};
use super::haskids::HasKids;

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

    pub(crate) fn full_build(&mut self, prep: &mut LayoutContext) -> CarriageTrainStateSpec {
        for child in self.kids.children.values_mut() {
            let build_size = child.build(prep);
            prep.state_request.playing_field_mut().set(child.coordinate_system(),build_size.height);
            for entry in &build_size.metadata {
                entry.add(prep.state_request.metadata_mut());
            }
        }
        for child in self.kids.children.values_mut() {
            child.locate(prep,&constant(0.));
        }
        CarriageTrainStateSpec::new(&prep.state_request)
    }
}

impl ContainerOrLeaf for Root {
    /* these not used as we are root */
    fn priority(&self) -> i64 { 0 }
    fn coordinate_system(&self) -> &CoordinateSystem { &CoordinateSystem::Window }
    fn anchor_leaf(&self, _answer_index: &StaticAnswer) -> Option<AnchoredLeaf> { None }
    fn build(&mut self, _prep: &mut LayoutContext) -> ContentSize {
        ContentSize {
            height: constant(0.),
            range: RangeUsed::All,
            metadata: vec![]
        } 
    }

    fn locate(&mut self, _prep: &mut LayoutContext, _top: &StaticValue<f64>) {}
    fn name(&self) -> &AllotmentName { &self.root_name }

    fn get_leaf(&mut self, pending: &LeafRequest, cursor: usize, styles: &Arc<StyleTree>) -> FloatingLeaf {
        self.kids.get_leaf(pending,cursor,styles)
    }
}
