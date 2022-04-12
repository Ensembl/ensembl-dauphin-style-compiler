use peregrine_toolkit::{puzzle::{StaticValue, commute_clonable}};
use crate::{allotment::{core::{ aligner::Aligner, carriageoutput::BoxPositionContext}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, boxes::boxtraits::Stackable}, CoordinateSystem};
use super::{container::{Container}, boxtraits::{Coordinated, BuildSize, ContainerSpecifics }};

#[derive(Clone)]
pub struct Overlay(Container);

impl Overlay {
    pub(crate) fn new(name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Overlay {
        Overlay(Container::new(name,style,aligner,UnpaddedOverlay::new()))
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child)
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

impl Coordinated for Overlay {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl Stackable for Overlay {
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>) { self.0.locate(prep,top); }
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn priority(&self) -> i64 { self.0.priority() }
    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize { self.0.build(prep) }
}

impl ContainerSpecifics for UnpaddedOverlay {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&mut self, _prep: &mut BoxPositionContext, children: &[(&Box<dyn Stackable>,BuildSize)]) -> StaticValue<f64> {
        let heights = children.iter().map(|x| x.1.height.clone()).collect::<Vec<_>>();
        commute_clonable(&heights,0.,|a,b| f64::max(*a,*b))
    }

    fn set_locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn Stackable>]) {
        for child in children {
            child.locate(prep,&top);
        }
    }
}
