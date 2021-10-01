use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::Arc};
use crate::{AllotmentDirection, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, SpaceBasePointRef, shape::shape::FilterMinMax, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::{AllotmentImpl, CoordinateSystem}, allotmentrequest::AllotmentRequestImpl, baseallotmentrequest::{BaseAllotmentRequest, remove_depth}, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearGroupEntry}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetAllotment {
    metadata: AllotmentMetadata,
    direction: AllotmentDirection,
    top: i64,
    offset: i64,
    size: i64,
    depth: i8,
    secret: bool
}

impl OffsetAllotment {
    pub(crate) fn new(metadata: &AllotmentMetadata, direction: &AllotmentDirection, top: i64, offset: i64, size: i64, depth: i8) -> OffsetAllotment {
        let secret = metadata.get_i64("secret-track").unwrap_or(0) != 0;
        OffsetAllotment {
            metadata: metadata.clone(),
            direction: direction.clone(),
            top, offset, size, depth, secret
        }
    }

    pub(super) fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        if !self.secret {
            full_metadata.add_pair("type","track");
            full_metadata.add_pair("offset",&self.top.to_string());
            full_metadata.add_pair("height",&self.size.to_string());
        }
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
    fn depth(&self) -> i8 { self.depth }
}

#[derive(Clone)]
pub struct OffsetAllotmentRequest(Arc<BaseAllotmentRequest<OffsetAllotment>>,i8);

impl LinearGroupEntry for OffsetAllotmentRequest {
    fn make(&self, offset: i64) -> i64 {
        self.0.set_allotment(Arc::new(OffsetAllotment::new(&self.0.metadata(),&self.0.direction(),offset,self.0.best_offset(offset),self.0.best_height(),self.1)));
        self.0.max_used()
    }

    fn get_all_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.0.metadata());
        if let Some(allotment) = self.0.base_allotment() {
            allotment.add_metadata(&mut full_metadata);
        }
        out.push(AllotmentMetadata::new(full_metadata));
    }

    fn name(&self) -> &str { self.0.metadata().name() }
    fn priority(&self) -> i64 { self.0.metadata().priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, _name: &str) -> Option<AllotmentRequest> {
        Some(AllotmentRequest::upcast(self.0.clone()))
    }
}

pub struct OffsetAllotmentRequestCreator(pub CoordinateSystem,pub AllotmentDirection);

impl LinearAllotmentRequestCreatorImpl for OffsetAllotmentRequestCreator {
    fn make(&self, metadata: &AllotmentMetadata) -> Arc<dyn LinearGroupEntry> {
        let mut name = metadata.name().to_string();
        let depth = remove_depth(&mut name);
        Arc::new(OffsetAllotmentRequest(Arc::new(BaseAllotmentRequest::new(metadata,&self.0,&self.1,depth)),depth))
    }

    fn hash(&self, name: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
    }
}
