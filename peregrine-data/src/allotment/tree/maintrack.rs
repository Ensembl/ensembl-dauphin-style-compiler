use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper}, offsetbuilder::LinearOffsetBuilder}, core::{allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}, arbitrator::{Arbitrator, SymbolicAxis}}}};

use super::{leaftransformer::{LeafTransformer, LeafGeometry}, allotmentbox::AllotmentBox, maintrackspec::MTSpecifier};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafTransformer>>>>
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata) -> MainTrackRequest {
        MainTrackRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new())
        }
    }
}

impl LinearGroupEntry for MainTrackRequest {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut new = AllotmentMetadataRequest::rebuild(&self.metadata);
        let requests = lock!(self.requests);
        for (_,request) in requests.iter() {
            request.add_allotment_metadata_values(&mut new);
        }
        out.push(AllotmentMetadata::new(new));
    }

    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, geometry: &LeafGeometry, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            let our_geometry = specifier.our_geometry(geometry);
            Arc::new(AllotmentRequestImpl::new(&self.metadata,&our_geometry,specifier.base().depth(),!specifier.sized()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }

    fn allot(&self, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, arbitrator: &mut Arbitrator) {
        arbitrator.add_symbolic(&SymbolicAxis::ScreenVert, self.metadata.name(), offset.primary());
        let requests = lock!(self.requests);
        let allot_box = AllotmentBox::empty().merge_requests(requests.values());
        let top_offset = offset.primary() + allot_box.top_space();
        for (specifier,request) in requests.iter() {
            let secondary = specifier.arbitrator_horiz(arbitrator).or_else(|| secondary.clone()).unwrap_or(0);
            let transformer = LeafTransformer::new(&request.geometry(),secondary,top_offset,allot_box.height(),request.depth());
            request.set_allotment(Arc::new(transformer));
        }
        offset.advance(allot_box.height());
    }
}

pub struct MainTrackLinearHelper;

impl LinearGroupHelper for MainTrackLinearHelper {
    type Key = String;

    fn make_linear_group_entry(&self, _geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(MainTrackRequest::new(&metadata))
    }

    fn entry_key(&self, name: &str) -> String {
        let specifier = MTSpecifier::new(name);
        specifier.base().name().to_string()
    }
}
