use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{lock, puzzle::{constant, StaticValue}};
use crate::{ allotment::{core::{trainstate::CarriageTrainStateSpec, boxtraits::{Stackable, BuildSize}, boxpositioncontext::BoxPositionContext, allotmentname::AllotmentName}, util::rangeused::RangeUsed}, CoordinateSystem};

#[derive(Clone)]
pub struct Root {
    root_name: AllotmentName,
    children: Arc<Mutex<Vec<Box<dyn Stackable>>>>
}

impl Root {
    pub fn new() -> Root { 
        Root { children: Arc::new(Mutex::new(vec![])), root_name: AllotmentName::new("") }
    }

    pub(crate) fn full_build(&mut self, prep: &mut BoxPositionContext) -> CarriageTrainStateSpec {
        let mut children = lock!(self.children);
        for child in &mut *children {
            let build_size = child.build(prep);
            prep.state_request.playing_field_mut().set(child.coordinate_system(),build_size.height);
        }
        for child in &mut *children {
            child.locate(prep,&constant(0.));
        }
        CarriageTrainStateSpec::new(&prep.state_request)
    }
}

impl Stackable for Root {
    /* not used as we are root */
    fn priority(&self) -> i64 { 0 } // doesn't matter
    fn coordinate_system(&self) -> &CoordinateSystem { &CoordinateSystem::Window }

    fn build(&self, _prep: &mut BoxPositionContext) -> BuildSize {
        BuildSize {
            name: self.root_name.clone(),
            height: constant(0.),
            range: constant(RangeUsed::All)
        } 
    }

    fn locate(&self, prep: &mut BoxPositionContext, top: &StaticValue<f64>) {}
    fn name(&self) -> &AllotmentName { &self.root_name }
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn add_child(&self, child: &dyn Stackable) {
        lock!(self.children).push(child.cloned());
    }
}
