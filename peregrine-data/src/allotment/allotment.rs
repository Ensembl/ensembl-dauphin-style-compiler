use std::sync::{Arc};

use crate::{AllotmentDirection, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

pub trait AllotmentImpl {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn direction(&self) -> AllotmentDirection;    
}

pub trait AsAllotmentImpl {
    fn as_allotment_impl<'a>(self: Arc<Self>) -> Arc<dyn AllotmentImpl + 'a> where Self: 'a;
}

impl<T: AllotmentImpl + Sized> AsAllotmentImpl for T {
    fn as_allotment_impl<'a>(self: Arc<Self>) -> Arc<dyn AllotmentImpl + 'a> where Self: 'a { self }
}

#[derive(Clone)]
pub struct Allotment(Arc<dyn AllotmentImpl>);

impl Allotment {
    pub fn new<T>(allotment_impl: Arc<T>) -> Allotment where T: AsAllotmentImpl + 'static {
        Allotment(allotment_impl.as_allotment_impl())
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
