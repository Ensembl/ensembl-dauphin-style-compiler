use std::fmt::Debug;
use std::sync::{Arc};
use crate::spacebase::spacebase2::SpaceBase2PointRef;
use crate::{AllotmentMetadataRequest, SpaceBase, SpaceBase2Point};
use crate::{SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

pub trait Transformer {
    fn transform_spacebase_point(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_spacebase2_point(&self, input: &SpaceBase2PointRef<f64,Allotment>) -> SpaceBase2Point<f64,Allotment>;
    fn transform_spacebase(&self, input: SpaceBase<f64>) -> SpaceBase<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn add_transform_metadata(&self, out: &mut AllotmentMetadataRequest);
}

#[derive(Clone)]
pub struct Allotment(Arc<dyn Transformer>);

#[cfg(debug_assertions)]
impl Debug for Allotment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Allotment(...)")
    }
}

impl Allotment {
    pub fn new(allotment_impl: Arc<dyn Transformer>) -> Allotment {
        Allotment(allotment_impl)
    }

    pub fn transform_spacebase_point(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        self.0.transform_spacebase_point(input)
    }

    pub fn transform_spacebase2_point(&self, input: &SpaceBase2PointRef<f64,Allotment>) -> SpaceBase2Point<f64,Allotment> {
        self.0.transform_spacebase2_point(input)
    }

    pub fn transform_spacebase(&self, input: SpaceBase<f64>) -> SpaceBase<f64> {
        self.0.transform_spacebase(input)
    }

    pub fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        self.0.transform_yy(values)
    }
}
