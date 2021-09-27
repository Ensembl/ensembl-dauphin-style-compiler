use std::{sync::Arc};
use crate::{AllotmentDirection, AllotmentGroup, AllotmentMetadata, AllotmentMetadataRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::AllotmentImpl, baseallotmentrequest::BaseAllotmentRequest, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearAllotmentRequestImpl}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetAllotment {
    metadata: AllotmentMetadata,
    direction: AllotmentDirection,
    offset: i64,
    size: i64
}

impl OffsetAllotment {
    pub(crate) fn new(metadata: &AllotmentMetadata, direction: &AllotmentDirection, offset: i64, size: i64) -> OffsetAllotment {
        OffsetAllotment {
            metadata: metadata.clone(),
            direction: direction.clone(),
            offset,size
        }
    }
}

impl LinearAllotmentImpl for OffsetAllotment {
    fn max(&self) -> i64 { self.offset+self.size }

    fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        full_metadata.add_pair("type","track");
        full_metadata.add_pair("offset",&self.offset.to_string());
        full_metadata.add_pair("height",&self.size.to_string());
    }

    fn up(self: Arc<Self>) -> Arc<dyn LinearAllotmentImpl> { self }
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

impl LinearAllotmentRequestImpl for OffsetAllotmentRequest {
    fn linear_allotment_impl<'a> (&'a self) -> Option<Arc<dyn LinearAllotmentImpl>> {
        self.base_allotment().map(|x| x.clone().up())
    }

    fn make(&self, offset: i64, size: i64) {
        self.set_allotment(Arc::new(OffsetAllotment::new(&self.metadata(),&self.direction(),offset,size)));
    }
}

pub struct OffsetAllotmentRequestCreator();

impl LinearAllotmentRequestCreatorImpl for OffsetAllotmentRequestCreator {
    fn make(&self, metadata: &AllotmentMetadata, group: &AllotmentGroup) -> Arc<dyn LinearAllotmentRequestImpl> {
        Arc::new(OffsetAllotmentRequest::new(metadata,group))
    }
}
