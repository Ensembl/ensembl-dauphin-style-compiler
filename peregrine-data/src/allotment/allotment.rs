use std::sync::{Arc, Mutex};

use peregrine_toolkit::lock;

use crate::{AllotmentDirection, AllotmentMetadata, Pitch, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

pub trait AllotmentImpl {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn direction(&self) -> AllotmentDirection;    
    fn apply_pitch(&self, pitch: &mut Pitch);
    fn metadata(&self) -> &AllotmentMetadata;
}

#[derive(Clone)]
pub struct Allotment(Arc<Mutex<Box<dyn AllotmentImpl>>>);

impl Allotment {
    pub fn new(allotment_impl: Box<dyn AllotmentImpl>) -> Allotment {
        Allotment(Arc::new(Mutex::new(allotment_impl)))
    }

    pub fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        lock!(self.0).transform_spacebase(input)
    }

    pub fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        lock!(self.0).transform_yy(values)
    }

    pub fn direction(&self) -> AllotmentDirection {
        lock!(self.0).direction().clone()
    }

    pub fn apply_pitch(&self, pitch: &mut Pitch) {
        lock!(self.0).apply_pitch(pitch)
    }

    pub fn metadata(&self) -> AllotmentMetadata {
        lock!(self.0).metadata().clone()
    }
}
