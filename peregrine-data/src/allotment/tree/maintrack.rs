use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{arbitrator::{Arbitrator, SymbolicAxis}, lineargroup::{LinearGroupEntry, LinearGroupHelper}, offsetbuilder::LinearOffsetBuilder}, core::{allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}}}};

use super::{leafboxtransformer::{LeafBoxTransformer, LeafGeometry}, allotmentbox::AllotmentBox, maintrackspec::MTSpecifier};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafBoxTransformer>>>>,
    geometry: LeafGeometry
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata, geometry: &LeafGeometry) -> MainTrackRequest {
        MainTrackRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new()),
            geometry: geometry.clone()
        }
    }
}

impl LinearGroupEntry for MainTrackRequest {
    fn allot(&self, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, arbitrator: &mut Arbitrator) {
        arbitrator.add_symbolic(&SymbolicAxis::ScreenVert, self.metadata.name(), offset.size());
        let requests = lock!(self.requests);
        let mut allot_box = AllotmentBox::empty();
        for (specifier,request) in requests.iter() {
            if specifier.sized() {
                allot_box = allot_box.merge(&AllotmentBox::new(request.metadata(),request.max_used()));
            }
        }
        let total_offset = offset.size() + allot_box.top_space();
        for (specifier,request) in requests.iter() {
            let secondary = specifier.arbitrator_horiz(arbitrator).or_else(|| secondary.clone()).unwrap_or(0);
            let transformer = LeafBoxTransformer::new(&request.geometry(),secondary,total_offset,allot_box.height(),request.depth());
            request.set_allotment(Arc::new(transformer));
        }
        offset.advance(allot_box.height());
    }

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
            Arc::new(AllotmentRequestImpl::new(&self.metadata,&our_geometry,specifier.base().depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }
}

pub struct MainTrackLinearHelper;

impl LinearGroupHelper for MainTrackLinearHelper {
    type Key = String;

    fn make_linear_group_entry(&self, geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(MainTrackRequest::new(&metadata,geometry))
    }

    fn entry_key(&self, name: &str) -> String {
        let specifier = MTSpecifier::new(name);
        specifier.base().name().to_string()
    }
}
