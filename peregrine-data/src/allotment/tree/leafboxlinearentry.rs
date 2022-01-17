use std::sync::Arc;

use crate::{AllotmentMetadataStore, AllotmentRequest, AllotmentMetadata, AllotmentMetadataRequest, allotment::{core::{allotmentrequest::{AllotmentRequestImpl}, basicallotmentspec::BasicAllotmentSpec, allotment::Transformer, arbitrator::{Arbitrator, SymbolicAxis}}, lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper}, offsetbuilder::LinearOffsetBuilder}}};
use super::{leaftransformer::{LeafTransformer, LeafGeometry}, allotmentbox::AllotmentBox};

#[derive(Clone)]
struct BoxLinearEntry {
    request: Arc<AllotmentRequestImpl<LeafTransformer>>,
    metadata: AllotmentMetadata,
    depth: i8,
    name_for_arbitrator: String
}

impl BoxLinearEntry {
    fn new(metadata: &AllotmentMetadata, spec: &BasicAllotmentSpec, geometry: &LeafGeometry) -> BoxLinearEntry {
        BoxLinearEntry {
            request: Arc::new(AllotmentRequestImpl::new(metadata,geometry,spec.depth(),false)),
            metadata: metadata.clone(),
            depth: spec.depth(),
            name_for_arbitrator: spec.name().to_string()
        } 
    }
}

impl LinearGroupEntry for BoxLinearEntry {
    fn allot(&self, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, arbitrator: &mut Arbitrator) {
        arbitrator.add_symbolic(&SymbolicAxis::ScreenHoriz, &self.name_for_arbitrator, offset.primary());
        let allot_box = AllotmentBox::new(&self.request.metadata(),self.request.max_used());
        let top_offset = offset.primary() + allot_box.top_space();
        self.request.set_allotment(Arc::new(LeafTransformer::new(self.request.geometry(),secondary.unwrap_or(0),top_offset,allot_box.height(),self.depth)));
        offset.advance(self.request.max_used());
    }

    fn priority(&self) -> i64 { self.request.metadata().priority() }

    fn make_request(&self, _geometry: &LeafGeometry, _allotment_metadata: &AllotmentMetadataStore, _name: &str) -> Option<AllotmentRequest> {
        Some(AllotmentRequest::upcast(self.request.clone()))
    }

    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let secret = self.metadata.get_i64("secret-track").unwrap_or(0) != 0;
        if secret { return; }
        if let Some(allotment) = self.request.transformer() {
            let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.metadata);
            allotment.add_transform_metadata(&mut full_metadata);
            out.push(AllotmentMetadata::new(full_metadata));
        }
    }
}

pub struct BoxAllotmentLinearGroupHelper;

impl LinearGroupHelper for BoxAllotmentLinearGroupHelper {
    type Key = BasicAllotmentSpec;

    fn make_linear_group_entry(&self, geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let spec = BasicAllotmentSpec::from_spec(full_path);
        let metadata = metadata.get_or_default(full_path);
        Arc::new(BoxLinearEntry::new(&metadata,&spec,geometry))
    }

    fn entry_key(&self, name: &str) -> BasicAllotmentSpec { BasicAllotmentSpec::from_spec(name).depthless() }
}
