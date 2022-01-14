use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{secondary::SecondaryPositionStore, lineargroup::{LinearGroupEntry, LinearGroupHelper}}, core::{allotmentrequest::{AllotmentRequestImpl, AgnosticAllotmentRequestImpl}}}};

use super::{leafboxallotment::LeafBoxAllotment, treeallotment::{tree_best_height, tree_best_offset}, maintrackspec::MTSpecifier};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafBoxAllotment>>>>,
    reverse: bool
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata, reverse: bool) -> MainTrackRequest {
        MainTrackRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new()),
            reverse
        }
    }
}

impl LinearGroupEntry for MainTrackRequest {
    fn allot(&self, secondary: i64, offset: i64, secondary_store: &SecondaryPositionStore) -> i64 {
        let mut best_offset_val = 0;
        let mut best_height_val = 0;
        let requests = lock!(self.requests);
        for (specifier,request) in requests.iter() {
            if specifier.sized() {
                best_offset_val = best_offset_val.max(tree_best_offset(&request,offset));
                best_height_val = best_height_val.max(tree_best_height(&request));
            }
        }
        for (specifier,request) in requests.iter() {
            let our_secondary = specifier.get_secondary(secondary,secondary_store);
            request.set_allotment(Arc::new(LeafBoxAllotment::new(&request.coord_system(),request.metadata(),our_secondary,offset,best_offset_val,best_height_val,specifier.base().depth(),self.reverse)));
        }
        best_height_val
    }

    fn get_all_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let requests = lock!(self.requests);
        for (specifier,request) in requests.iter() {
            let mut full_metadata = AllotmentMetadataRequest::rebuild(&self.metadata);
            if specifier.sized() { // XXX wallpaper metadata
                if let Some(allotment) = request.base_allotment() {
                    allotment.add_metadata(&mut full_metadata);
                }
            }
            out.push(AllotmentMetadata::new(full_metadata));
        }
    }

    fn name_for_secondary(&self) -> &str { self.metadata.name() }
    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            Arc::new(AllotmentRequestImpl::new(&self.metadata,&specifier.coord_system(self.reverse),specifier.base().depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }
}

pub struct MainTrackLinearHelper(pub bool);

impl LinearGroupHelper for MainTrackLinearHelper {
    type Key = String;

    fn make_linear_group_entry(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(MainTrackRequest::new(&metadata,self.0))
    }

    fn entry_key(&self, name: &str) -> String {
        let specifier = MTSpecifier::new(name);
        specifier.base().name().to_string()
    }

    fn is_reverse(&self) -> bool { self.0 }
}
