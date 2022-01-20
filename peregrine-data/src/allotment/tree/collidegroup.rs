use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::{Arc, Mutex}, hash::{Hash, Hasher}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper}}, core::{allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}, arbitrator::{Arbitrator, SymbolicAxis}}}};

use super::{leaftransformer::{LeafTransformer, LeafGeometry}, allotmentbox::{AllotmentBox, AllotmentBoxBuilder}, maintrackspec::MTSpecifier};

pub struct CollideGroupRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafTransformer>>>>
}

impl CollideGroupRequest {
    fn new(metadata: &AllotmentMetadata) -> CollideGroupRequest {
        CollideGroupRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new())
        }
    }

    pub(crate) fn add_allotment_metadata_values(&self, out: &mut AllotmentMetadataRequest) {
        let requests = lock!(self.requests);
        for request in requests.values() {
            request.add_allotment_metadata_values(out);
        }
    }

    fn calculate_offset(&self, specifier: &MTSpecifier) -> i64 {
        let group = specifier.base().group();
        let mut hasher = DefaultHasher::new();
        group.hash(&mut hasher);
        if hasher.finish() % 2 == 0 { 32 } else { 0 }
    }

    fn make_content_box(&self, specifier: &MTSpecifier, request: &AllotmentRequestImpl<LeafTransformer>, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let mut box_builder = AllotmentBoxBuilder::empty(request.max_used());
        if let Some(indent) =  specifier.arbitrator_horiz(arbitrator) {
            box_builder.set_self_indent(Some(&indent));
        }
        let content_box = AllotmentBox::new(box_builder);
        let transformer = LeafTransformer::new(&request.geometry(),&content_box,request.depth());
        request.set_allotment(Arc::new(transformer));
        content_box
    }

    fn make_child_box(&self, specifier: &MTSpecifier, request: &AllotmentRequestImpl<LeafTransformer>, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let mut builder = AllotmentBoxBuilder::empty(0);
        builder.add_padding_top(self.calculate_offset(specifier));
        builder.append(self.make_content_box(specifier,request,arbitrator));
        AllotmentBox::new(builder)
    }
}

impl LinearGroupEntry for CollideGroupRequest {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, _out: &mut Vec<AllotmentMetadata>) {}

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

    fn allot(&self, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let requests = lock!(self.requests);
        let mut child_boxes = vec![];
        for (specifier,request) in requests.iter() {
            let child_box = self.make_child_box(specifier,request,arbitrator);
            child_boxes.push(child_box);
        }
        let mut builder = AllotmentBoxBuilder::empty(0);
        builder.overlay_all(child_boxes);
        AllotmentBox::new(builder)
    }
}

pub struct CollideGroupLinearHelper;

impl LinearGroupHelper for CollideGroupLinearHelper {
    type Key = Option<String>;
    type Value = CollideGroupRequest;

    fn make_linear_group_entry(&self, _geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<CollideGroupRequest> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(CollideGroupRequest::new(&metadata))
    }

    fn entry_key(&self, name: &str) -> Option<String> {
        let specifier = MTSpecifier::new(name);
        specifier.base().group().clone()
    }
}
