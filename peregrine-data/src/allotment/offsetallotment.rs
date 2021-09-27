use std::sync::Arc;

use peregrine_toolkit::{refs::{Upcast, UpcastFrom}, upcast};

use crate::{AllotmentDirection, AllotmentMetadata, AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

use super::{allotment::AllotmentImpl, allotmentrequest::BaseAllotmentRequest};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetAllotment {
    metadata: AllotmentMetadata,
    direction: AllotmentDirection,
    offset: i64,
    size: i64
}

upcast!(OffsetAllotment,dyn AllotmentImpl);

impl OffsetAllotment {
    fn new(request: &OffsetAllotmentRequest, offset: i64, size: i64) -> OffsetAllotment {
        OffsetAllotment {
            metadata: request.metadata().clone(),
            direction: request.direction(),
            offset,size
        }
    }

    pub(crate) fn max(&self) -> i64 { self.offset+self.size }

    pub(crate) fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        full_metadata.add_pair("type","track");
        full_metadata.add_pair("offset",&self.offset.to_string());
        full_metadata.add_pair("height",&self.size.to_string());
    }
}

impl AllotmentImpl for OffsetAllotment {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        output.normal += self.offset as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        let offset = self.offset as f64;
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }

    fn direction(&self) -> AllotmentDirection { self.direction.clone() }
}

pub type OffsetAllotmentRequest = BaseAllotmentRequest<OffsetAllotment>;

impl OffsetAllotmentRequest {
    pub fn make(&self, offset: i64, size: i64) {
        self.set_allotment(Arc::new(OffsetAllotment::new(&self,offset,size)));
    }
}
