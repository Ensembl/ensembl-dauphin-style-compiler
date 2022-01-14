use std::sync::Mutex;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, CoordinateSystem, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

use super::{baseallotmentrequest::BaseAllotmentRequest, allotment::AllotmentImpl};

pub(crate) struct DustbinAllotment;

impl AllotmentImpl for DustbinAllotment {
    fn coord_system(&self) -> CoordinateSystem { CoordinateSystem::Window }
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> { input.make() }
    fn transform_yy(&self, _values: &[Option<f64>]) -> Vec<Option<f64>> { vec![] }
    fn depth(&self) -> i8 { 0 }
}
