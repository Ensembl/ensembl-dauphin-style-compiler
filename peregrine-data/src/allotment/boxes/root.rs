use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{lock, puzzle::{constant}};
use crate::{ allotment::core::{carriageoutput::BoxPositionContext, trainstate::CarriageTrainStateSpec, boxtraits::Stackable}};

#[derive(Clone)]
pub struct Root {
    children: Arc<Mutex<Vec<Box<dyn Stackable>>>>
}

impl Root {
    pub fn new() -> Root { 
        Root { children: Arc::new(Mutex::new(vec![])) }
    }

    pub(crate) fn add_child(&self, child: &dyn Stackable) {
        lock!(self.children).push(child.cloned());
    }

    pub(crate) fn build(&mut self, prep: &mut BoxPositionContext) -> CarriageTrainStateSpec {
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
