use std::sync::Arc;

use crate::{AllotmentMetadataStore, AllotmentRequest, AllotmentMetadata, AllotmentMetadataRequest, allotment::{core::{allotmentrequest::{AllotmentRequestImpl}, basicallotmentspec::BasicAllotmentSpec, allotment::Transformer}, lineargroup::{secondary::{SecondaryPositionStore}, lineargroup::{LinearGroupEntry, LinearGroupHelper}}}};
use super::{leafboxtransformer::{LeafBoxTransformer, LeafGeometry}, treeallotment::{tree_best_offset, tree_best_height}};

#[derive(Clone)]
struct BoxLinearEntry {
    request: Arc<AllotmentRequestImpl<LeafBoxTransformer>>,
    metadata: AllotmentMetadata,
    depth: i8,
    name_for_secondary: String
}

impl BoxLinearEntry {
    fn new(metadata: &AllotmentMetadata, spec: &BasicAllotmentSpec,geometry: &LeafGeometry) -> BoxLinearEntry {
        BoxLinearEntry {
            request: Arc::new(AllotmentRequestImpl::new(metadata,geometry,spec.depth())),
            metadata: metadata.clone(),
            depth: spec.depth(),
            name_for_secondary: spec.name().to_string()
        } 
    }
}

impl LinearGroupEntry for BoxLinearEntry {
    fn allot(&self, geometry: &LeafGeometry, secondary: &Option<i64>, offset: i64, _secondary_store: &SecondaryPositionStore) -> i64 {
        let offset = tree_best_offset(&self.request,offset);
        let size = tree_best_height(&self.request);
        self.request.set_allotment(Arc::new(LeafBoxTransformer::new(geometry,secondary,offset,offset,size,self.depth)));
        self.request.max_used()
    }

    fn name_for_secondary(&self) -> &str { &self.name_for_secondary }
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
