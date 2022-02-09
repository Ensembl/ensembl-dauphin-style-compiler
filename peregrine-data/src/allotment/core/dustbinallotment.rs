use crate::{SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint, AllotmentMetadataRequest, CoordinateSystemVariety};

use super::{allotment::{Transformer}};

pub(crate) struct DustbinAllotment;

impl Transformer for DustbinAllotment {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> { input.make() }
    fn transform_yy(&self, _values: &[Option<f64>]) -> Vec<Option<f64>> { vec![] }
    fn add_transform_metadata(&self, _out: &mut AllotmentMetadataRequest) {}
}
