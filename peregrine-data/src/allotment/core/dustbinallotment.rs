use crate::{AllotmentMetadataRequest, SpaceBase};

use super::{allotment::{Transformer, Allotment}};

pub(crate) struct DustbinAllotment;

impl Transformer for DustbinAllotment {
    fn transform_spacebase(&self, input: &SpaceBase<f64,Allotment>) -> SpaceBase<f64,Allotment> { input.clone() }
    fn transform_yy(&self, _values: &[Option<f64>]) -> Vec<Option<f64>> { vec![] }
    fn add_transform_metadata(&self, _out: &mut AllotmentMetadataRequest) {}
}
