use crate::{spacebase::{spacebase::SpaceBasePointRef}, AllotmentMetadataRequest, CoordinateSystemVariety, SpaceBasePoint, Allotment, SpaceBase};

use super::{allotment::{Transformer}};

pub(crate) struct DustbinAllotment;

impl Transformer for DustbinAllotment {
    fn transform_spacebase(&self, input: &SpaceBase<f64,Allotment>) -> SpaceBase<f64,Allotment> { input.clone() }
    fn transform_spacebase_point(&self, input: &SpaceBasePointRef<f64,Allotment>) -> SpaceBasePoint<f64,Allotment> { input.make() }
    fn transform_yy(&self, _values: &[Option<f64>]) -> Vec<Option<f64>> { vec![] }
    fn add_transform_metadata(&self, _out: &mut AllotmentMetadataRequest) {}
}
