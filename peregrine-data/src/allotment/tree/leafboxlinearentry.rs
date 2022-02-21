use std::sync::{Arc, Mutex};

use peregrine_toolkit::lock;

use crate::{AllotmentMetadataStore, AllotmentRequest, AllotmentMetadata, AllotmentMetadataRequest, allotment::{core::{allotmentrequest::{AllotmentRequestImpl}, basicallotmentspec::BasicAllotmentSpec, arbitrator::{Arbitrator, SymbolicAxis}}, lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper}}}, CoordinateSystem};
use super::{allotmentbox::{AllotmentBox, AllotmentBoxBuilder}};

#[derive(Clone)]
pub struct BoxLinearEntry {
    transformer: Arc<AllotmentRequestImpl>,
    metadata: AllotmentMetadata,
    name_for_arbitrator: String,
    allot_box: Arc<Mutex<Option<AllotmentBox>>>
}

impl BoxLinearEntry {
    fn new(metadata: &AllotmentMetadata, spec: &BasicAllotmentSpec, geometry: &CoordinateSystem) -> BoxLinearEntry {
        BoxLinearEntry {
            transformer: Arc::new(AllotmentRequestImpl::new(metadata,geometry,spec.depth(),false)),
            metadata: metadata.clone(),
            name_for_arbitrator: spec.name().to_string(),
            allot_box: Arc::new(Mutex::new(None))
        } 
    }
}

impl LinearGroupEntry for BoxLinearEntry {
    fn make_request(&self, _geometry: &CoordinateSystem, _allotment_metadata: &AllotmentMetadataStore, _name: &str) -> Option<AllotmentRequest> {
        Some(AllotmentRequest::upcast(self.transformer.clone()))
    }

    fn bump(&self, _arbitrator: &mut Arbitrator) {}

    fn allot(&self, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let allot_box = AllotmentBox::new(AllotmentBoxBuilder::new(&AllotmentMetadata::new(AllotmentMetadataRequest::new("", 0)),self.transformer.max_y()));
        arbitrator.add_symbolic(&SymbolicAxis::ScreenHoriz, &self.name_for_arbitrator, allot_box.top_delayed());
        self.transformer.set_allotment(Arc::new(allot_box.clone()));
        *lock!(self.allot_box) = Some(allot_box.clone());
        allot_box
    }

    fn priority(&self) -> i64 { self.transformer.metadata().priority() }

    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let secret = self.metadata.get_i64("secret-track").unwrap_or(0) != 0;
        if secret { return; }
        if let Some(allot_box) = lock!(self.allot_box).as_ref() {
            let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.metadata);
            allot_box.add_transform_metadata(&mut full_metadata);
            out.push(AllotmentMetadata::new(full_metadata));
        }
    }
}

pub struct BoxAllotmentLinearGroupHelper;

impl LinearGroupHelper for BoxAllotmentLinearGroupHelper {
    type Key = BasicAllotmentSpec;
    type Value = BoxLinearEntry;

    fn make_linear_group_entry(&self, geometry: &CoordinateSystem, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<BoxLinearEntry> {
        let spec = BasicAllotmentSpec::from_spec(full_path);
        let metadata = metadata.get_or_default(full_path);
        Arc::new(BoxLinearEntry::new(&metadata,&spec,geometry))
    }

    fn entry_key(&self, name: &str) -> BasicAllotmentSpec { BasicAllotmentSpec::from_spec(name).depthless() }

    fn bump(&self, arbitrator: &mut Arbitrator) {}
}
