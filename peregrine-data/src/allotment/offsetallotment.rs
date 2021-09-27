use std::{sync::Arc};
use crate::{AllotmentDirection, AllotmentGroup, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::AllotmentImpl, baseallotmentrequest::BaseAllotmentRequest, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearGroupEntry}};

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

    fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        full_metadata.add_pair("type","track");
        full_metadata.add_pair("offset",&self.offset.to_string());
        full_metadata.add_pair("height",&self.size.to_string());
    }
}

impl LinearAllotmentImpl for OffsetAllotment {
    fn max(&self) -> i64 { self.offset+self.size }

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

#[derive(Clone)]
pub struct OffsetAllotmentRequest(Arc<BaseAllotmentRequest<OffsetAllotment>>);

impl LinearGroupEntry for OffsetAllotmentRequest {
    fn make(&self, offset: i64, size: i64) {
        self.0.set_allotment(Arc::new(OffsetAllotment::new(&self.0.metadata(),&self.0.direction(),offset,size)));
    }

    fn add_metadata(&self) -> AllotmentMetadataRequest {
        let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.0.metadata());
        if let Some(allotment) = self.0.base_allotment() {
            allotment.add_metadata(&mut full_metadata);
        }
        full_metadata
    }

    fn max(&self) -> i64 { self.0.base_allotment().map(|x| x.max()).unwrap_or(0) }
    fn priority(&self) -> i64 { self.0.metadata().priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, _name: &str) -> Option<AllotmentRequest> {
        Some(AllotmentRequest::upcast(self.0.clone()))
    }
}

pub struct OffsetAllotmentRequestCreator();

impl LinearAllotmentRequestCreatorImpl for OffsetAllotmentRequestCreator {
    fn make(&self, metadata: &AllotmentMetadata, group: &AllotmentGroup) -> Arc<dyn LinearGroupEntry> {
        let r = Arc::new(BaseAllotmentRequest::new(metadata,group));
        Arc::new(OffsetAllotmentRequest(r))
    }
}
