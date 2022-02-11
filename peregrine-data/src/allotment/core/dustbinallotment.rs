use crate::{SpaceBasePointRef, spacebase::{spacebase::SpaceBasePoint, spacebase2::SpaceBase2PointRef}, AllotmentMetadataRequest, CoordinateSystemVariety, SpaceBase, SpaceBase2Point, Allotment, SpaceBase2};

use super::{allotment::{Transformer}};

pub(crate) struct DustbinAllotment;

impl Transformer for DustbinAllotment {
    fn transform_spacebase(&self, input: SpaceBase<f64>) -> SpaceBase<f64> { input.clone() }
    fn transform_spacebase2(&self, input: &SpaceBase2<f64,Allotment>) -> SpaceBase2<f64,Allotment> { input.clone() }
    fn transform_spacebase_point(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> { input.make() }
    fn transform_spacebase2_point(&self, input: &SpaceBase2PointRef<f64,Allotment>) -> SpaceBase2Point<f64,Allotment> { input.make() }
    fn transform_yy(&self, _values: &[Option<f64>]) -> Vec<Option<f64>> { vec![] }
    fn add_transform_metadata(&self, _out: &mut AllotmentMetadataRequest) {}
}
