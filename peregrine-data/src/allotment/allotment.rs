use std::sync::{Arc};

use crate::{AllotmentDirection, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

pub trait AllotmentImpl {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn direction(&self) -> AllotmentDirection;    
}

#[derive(Clone)]
pub struct Allotment(Arc<dyn AllotmentImpl>);

impl Allotment {
    pub fn new(allotment_impl: Arc<dyn AllotmentImpl>) -> Allotment {
        Allotment(allotment_impl)
    }

    pub fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        self.0.transform_spacebase(input)
    }

    pub fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        self.0.transform_yy(values)
    }

    pub fn direction(&self) -> AllotmentDirection {
        self.0.direction().clone()
    }
}
