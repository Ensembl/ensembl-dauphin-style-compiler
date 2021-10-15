use std::sync::Arc;
use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};
use super::{allotment::{AllotmentImpl, CoordinateSystem}, baseallotmentrequest::{BaseAllotmentRequest, remove_depth, remove_secondary}, lineargroup::{LinearAllotmentImpl, LinearAllotmentRequestCreatorImpl, LinearGroupEntry, SecondaryPositionStore}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetAllotment {
    metadata: AllotmentMetadata,
    secondary: i64,
    top: i64,
    offset: i64,
    size: i64,
    depth: i8,
    secret: bool,
    reverse: bool
}

impl OffsetAllotment {
    pub(crate) fn new(metadata: &AllotmentMetadata, secondary: i64, top: i64, offset: i64, size: i64, depth: i8, reverse: bool) -> OffsetAllotment {
        let secret = metadata.get_i64("secret-track").unwrap_or(0) != 0;
        OffsetAllotment {
            metadata: metadata.clone(),
            secondary, top, offset, size, depth, secret, reverse
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
        if self.reverse {
            output.normal = (self.offset + self.size) as f64 - output.normal;
        } else {
            output.normal += self.offset as f64;
        }
        output.tangent += self.secondary as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        if self.reverse {
            let offset = (self.offset + self.size) as f64;
            values.iter().map(|x| x.map(|y| offset-y)).collect()
    
        } else {
            let offset = self.offset as f64;
            values.iter().map(|x| x.map(|y| y+offset)).collect()
        }
    }

    fn depth(&self) -> i8 { self.depth }
}

#[derive(Clone)]
pub struct OffsetAllotmentRequest(Arc<BaseAllotmentRequest<OffsetAllotment>>,i8,bool,String);

impl LinearGroupEntry for OffsetAllotmentRequest {
    fn make(&self, secondary: i64, offset: i64, secondary_store: &SecondaryPositionStore) -> i64 {
        let offset = self.0.best_offset(offset);
        let size = self.0.best_height();
        self.0.set_allotment(Arc::new(OffsetAllotment::new(&self.0.metadata(),secondary,offset,offset,size,self.1,self.2)));
        self.0.max_used()
    }

    fn get_all_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.0.metadata());
        if let Some(allotment) = self.0.base_allotment() {
            allotment.add_metadata(&mut full_metadata);
        }
        out.push(AllotmentMetadata::new(full_metadata));
    }

    fn name(&self) -> &str { &self.3 }
    fn priority(&self) -> i64 { self.0.metadata().priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, _name: &str) -> Option<AllotmentRequest> {
        Some(AllotmentRequest::upcast(self.0.clone()))
    }
}

pub struct OffsetAllotmentRequestCreator(pub CoordinateSystem, pub bool);

impl LinearAllotmentRequestCreatorImpl for OffsetAllotmentRequestCreator {
    fn make(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let mut name =full_path.to_string();
        let depth = remove_depth(&mut name);
        let metadata = metadata.get(full_path).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(full_path,0)));
        Arc::new(OffsetAllotmentRequest(Arc::new(BaseAllotmentRequest::new(&metadata,&self.0,depth)),depth,self.1,name))
    }

    fn base(&self, name: &str) -> String {
        let mut out = name.to_string();
        remove_depth(&mut out);
        out
    }

    fn is_reverse(&self) -> bool { self.1 }
}
