use std::sync::Arc;

use crate::{CoordinateSystem, AllotmentMetadataStore, AllotmentRequest, AllotmentMetadata, AllotmentMetadataRequest};

use super::{leafboxallotment::LeafBoxAllotment, lineargroup::{secondary::SecondaryPositionStore, lineargroup::{LinearGroupHelper, LinearGroupEntry}}, baseallotmentrequest::{BaseAllotmentRequest}, basicallotmentspec::{BasicAllotmentSpec}, allotmentrequest::AllotmentRequestImpl, treeallotment::{tree_best_offset, tree_best_height}};

#[derive(Clone)]
struct BoxLinearEntry {
    request: Arc<BaseAllotmentRequest<LeafBoxAllotment>>,
    depth: i8,
    reverse: bool,
    name_for_secondary: String
}

impl BoxLinearEntry {
    fn new(metadata: &AllotmentMetadata, spec: &BasicAllotmentSpec, coord_system: &CoordinateSystem, reverse: bool) -> BoxLinearEntry {
        BoxLinearEntry {
            request: Arc::new(BaseAllotmentRequest::new(metadata,coord_system,spec.depth())),
            depth: spec.depth(),
            reverse,
            name_for_secondary: spec.name().to_string()
        } 
    }
}

impl LinearGroupEntry for BoxLinearEntry {
    fn allot(&self, secondary: i64, offset: i64, _secondary_store: &SecondaryPositionStore) -> i64 {
        let offset = tree_best_offset(&self.request,offset);
        let size = tree_best_height(&self.request);
        self.request.set_allotment(Arc::new(LeafBoxAllotment::new(&self.request.coord_system(),&self.request.metadata(),secondary,offset,offset,size,self.depth,self.reverse)));
        self.request.max_used()
    }

    fn get_all_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut full_metadata = AllotmentMetadataRequest::rebuild(self.request.metadata());
        if let Some(allotment) = self.request.base_allotment() {
            allotment.add_metadata(&mut full_metadata);
        }
        out.push(AllotmentMetadata::new(full_metadata));
    }

    fn name_for_secondary(&self) -> &str { &self.name_for_secondary }
    fn priority(&self) -> i64 { self.request.metadata().priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, _name: &str) -> Option<AllotmentRequest> {
        Some(AllotmentRequest::upcast(self.request.clone()))
    }
}

pub struct BoxAllotmentLinearGroupHelper(pub CoordinateSystem, pub bool);

impl LinearGroupHelper for BoxAllotmentLinearGroupHelper {
    type Key = BasicAllotmentSpec;

    fn make_linear_group_entry(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let spec = BasicAllotmentSpec::from_spec(full_path);
        let metadata = metadata.get_or_default(full_path);
        Arc::new(BoxLinearEntry::new(&metadata,&spec,&self.0,self.1))
    }

    fn entry_key(&self, name: &str) -> BasicAllotmentSpec { BasicAllotmentSpec::from_spec(name).depthless() }

    fn is_reverse(&self) -> bool { self.1 }
}
