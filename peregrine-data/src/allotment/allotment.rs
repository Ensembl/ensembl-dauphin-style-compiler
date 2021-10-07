use std::sync::{Arc};
use std::hash::{ Hash };
use crate::{SpaceBasePointRef, shape::shape::FilterMinMax, spacebase::spacebase::SpaceBasePoint};

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub enum CoordinateSystem {
    Tracking,
    TrackingBottom,
    Window,
    SidewaysLeft,
    SidewaysRight
}

impl CoordinateSystem {
    pub(crate) fn filter_min_max(&self) -> FilterMinMax {
        match self {
            CoordinateSystem::Tracking => FilterMinMax::Base,
            _ => FilterMinMax::None
        }
    }
}

pub trait AllotmentImpl {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn depth(&self) -> i8;
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

    pub fn depth(&self) -> i8 { self.0.depth() }
}
