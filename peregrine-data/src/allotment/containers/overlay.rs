use peregrine_toolkit::{puzzle::{StaticValue, commute_clonable }};
use crate::{allotment::{ layout::{stylebuilder::{ContainerOrLeaf}, layoutcontext::LayoutContext, contentsize::ContentSize}}};

use super::container::ContainerSpecifics;

#[derive(Clone)]
pub(crate) struct Overlay;

impl Overlay {
    pub(super) fn new() -> Overlay { Overlay }
}

impl ContainerSpecifics for Overlay {
    fn build_reduce(&self, _prep: &mut LayoutContext, children: &[(&Box<dyn ContainerOrLeaf>,ContentSize)]) -> StaticValue<f64> {
        let heights = children.iter().map(|x| x.1.height.clone()).collect::<Vec<_>>();
        commute_clonable(&heights,0.,|a,b| f64::max(*a,*b))
    }

    fn set_locate(&self, prep: &mut LayoutContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn ContainerOrLeaf>]) {
        for child in children {
            child.locate(prep,&top);
        }
    }
}
