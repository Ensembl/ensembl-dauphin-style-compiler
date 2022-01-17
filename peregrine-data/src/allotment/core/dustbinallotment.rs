use crate::{CoordinateSystem, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint, AllotmentMetadataRequest};

use super::{allotment::Transformer};

pub(crate) struct DustbinAllotment;

impl Transformer for DustbinAllotment {
    fn coord_system(&self) -> CoordinateSystem { CoordinateSystem::Window }
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> { input.make() }
    fn transform_yy(&self, _values: &[Option<f64>]) -> Vec<Option<f64>> { vec![] }
    fn depth(&self) -> i8 { 0 }
    fn add_transform_metadata(&self, _out: &mut AllotmentMetadataRequest) {}
}
