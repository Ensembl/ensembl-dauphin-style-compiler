use std::{sync::{Arc, Mutex}};

use peregrine_toolkit::{lock, puzzle::{constant, StaticAnswer}, log };

use crate::{ allotment::core::{playingfield::{PlayingFieldHolder, PlayingFieldPieces, PlayingField}, carriageoutput::BoxPositionContext, heighttracker::HeightTracker2Values, trainstate::CarriageTrainStateSpec}};

use super::boxtraits::Stackable;

#[derive(Clone)]
pub struct Root {
    playing_field: Arc<Mutex<PlayingFieldHolder>>,
    children: Arc<Mutex<Vec<Box<dyn Stackable>>>>
}

impl Root {
    pub fn new() -> Root { 
        let playing_field = Arc::new(Mutex::new(PlayingFieldHolder::new()));
        Root { playing_field, children: Arc::new(Mutex::new(vec![])) }
    }

    pub(crate) fn add_child(&self, child: &dyn Stackable) {
        lock!(self.children).push(child.cloned());
    }

    pub fn playing_field_pieces(&self) -> PlayingFieldPieces {
        PlayingFieldPieces::new(&&*lock!(self.playing_field))
    }

    pub fn playing_field(&self, answer_index: &mut StaticAnswer) -> PlayingField {
        lock!(self.playing_field).get(answer_index)
    }

    pub(crate) fn build(&mut self, prep: &mut BoxPositionContext) -> CarriageTrainStateSpec {
        let mut children = lock!(self.children);
        for child in &mut *children {
            let build_size = child.build(prep);
            lock!(self.playing_field).set(child.coordinate_system(),&build_size.height);
        }
        for child in &mut *children {
            child.locate(prep,&constant(0.));
        }
        let spec = CarriageTrainStateSpec::new(&prep.state_request,&prep.independent_answer);
        log!("spec: {:?}",spec);
        lock!(self.playing_field).ready();
        spec
    }
}
