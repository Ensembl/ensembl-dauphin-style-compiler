use std::fmt::Debug;
use std::sync::{Arc};
use crate::allotment::tree::allotmentbox::AllotmentBox;
use crate::spacebase::spacebase2::SpaceBase2PointRef;
use crate::{AllotmentMetadataRequest, SpaceBase, SpaceBase2Point, SpaceBase2};
use crate::{SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

pub trait Transformer {
    fn transform_spacebase_point(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_spacebase2(&self, input: &SpaceBase2<f64,Allotment>) -> SpaceBase2<f64,Allotment>;
    fn transform_spacebase2_point(&self, input: &SpaceBase2PointRef<f64,Allotment>) -> SpaceBase2Point<f64,Allotment>;
    fn transform_spacebase(&self, input: SpaceBase<f64>) -> SpaceBase<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn add_transform_metadata(&self, out: &mut AllotmentMetadataRequest);
}

#[derive(Clone)]
pub struct Allotment(Arc<dyn Transformer>,Arc<AllotmentBox>);

#[cfg(debug_assertions)]
impl Debug for Allotment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Allotment(...)")
    }
}

impl Allotment {
    pub fn new(allotment_impl: Arc<dyn Transformer>, allot_box: Arc<AllotmentBox>) -> Allotment {
        Allotment(allotment_impl,allot_box)
    }

    pub fn allotment_box(&self) -> &Arc<AllotmentBox> { &self.1 }

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
