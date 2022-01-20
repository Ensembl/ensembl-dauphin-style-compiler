use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper, LinearGroup}}, core::{allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}, arbitrator::{Arbitrator, SymbolicAxis}}}};

use super::{leaftransformer::{LeafTransformer, LeafGeometry}, allotmentbox::{AllotmentBox, AllotmentBoxBuilder}, maintrackspec::MTSpecifier};

/* MainTrack allotments are the allotment spec for the main gb tracks and so have complex spceifiers. The format is
 * track:NAME:(XXX todo sub-tracks) or wallpaper[depth]
 * where [depth] is the drawing priority, a possibly negative number
 */

pub struct MainTrackRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafTransformer>>>>,
    //requests2: Mutex<LinearGroup<CollideGroupLinearHelper>>
}

impl MainTrackRequest {
    fn new(metadata: &AllotmentMetadata, geometry: &LeafGeometry) -> MainTrackRequest {
        MainTrackRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new()),
            //requests2: Mutex::new(LinearGroup::new(geometry,CollideGroupLinearHelper))
        }
    }
}

impl LinearGroupEntry for MainTrackRequest {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        let mut new = AllotmentMetadataRequest::rebuild(&self.metadata);
        /* old */
        let requests = lock!(self.requests);
        for (_,request) in requests.iter() {
            request.add_allotment_metadata_values(&mut new);
        }
        /* new */
        /*
        for (_,entry) in lock!(self.requests2).iter() {
            entry.add_allotment_metadata_values(&mut new);
        }
        */
        /**/
        out.push(AllotmentMetadata::new(new));
    }

    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, geometry: &LeafGeometry, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        /* new */
        //return lock!(self.requests2).make_request(allotment_metadata,name);
        /* old */
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            let our_geometry = specifier.our_geometry(geometry);
            Arc::new(AllotmentRequestImpl::new(&self.metadata,&our_geometry,specifier.base().depth(),!specifier.sized()))
        });
        /**/
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }

    fn allot(&self, secondary: &Option<i64>, arbitrator: &mut Arbitrator) -> AllotmentBox {
        /* new */
        /*
        let requests = lock!(self.requests2);
        let mut allot_box = AllotmentBox::new(&self.metadata,0);
        for (_,request) in requests.iter() {
            let inner = AllotmentBox::new(request.metadata(),request.max_used());
            allot_box = allot_box.add_contain(inner);
        }
        */
        /* old */
        let requests = lock!(self.requests);
        let mut child_boxes = vec![];
        for (specifier,request) in requests.iter() {
            let secondary = specifier.arbitrator_horiz(arbitrator).or_else(|| secondary.clone()).unwrap_or(0);
            let child_allotment = AllotmentBox::new(AllotmentBoxBuilder::new(request.metadata(),request.max_used()));
            let transformer = LeafTransformer::new(&request.geometry(),secondary,&child_allotment,request.depth());
            request.set_allotment(Arc::new(transformer));
            child_boxes.push(child_allotment);
        }
        let mut builder = AllotmentBoxBuilder::empty();
        builder.overlay_all(child_boxes);
        let allot_box = AllotmentBox::new(builder);
        arbitrator.add_symbolic(&SymbolicAxis::ScreenVert, self.metadata.name(), allot_box.top_delayed());
        allot_box
    }
}

pub struct MainTrackLinearHelper;

impl LinearGroupHelper for MainTrackLinearHelper {
    type Key = String;
    type Value = MainTrackRequest;

    fn make_linear_group_entry(&self, geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<MainTrackRequest> {
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
