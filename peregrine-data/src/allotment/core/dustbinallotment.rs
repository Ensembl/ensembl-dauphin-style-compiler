use crate::{AllotmentMetadataRequest, SpaceBase};

use super::{allotment::{Transformer, Allotment}};

pub(crate) struct DustbinAllotment;

impl Transformer for DustbinAllotment {
    fn add_transform_metadata(&self, _out: &mut AllotmentMetadataRequest) {}
}
