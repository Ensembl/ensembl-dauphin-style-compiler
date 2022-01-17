use std::{sync::{Arc, Mutex}, collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}};

use peregrine_toolkit::lock;

use crate::{allotment::{lineargroup::{lineargroup::{LinearGroupHelper, LinearGroupEntry}, secondary::{SecondaryPositionResolver}, offsetbuilder::LinearOffsetBuilder}, core::allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}}, AllotmentMetadataStore, AllotmentMetadata, AllotmentMetadataRequest, AllotmentRequest};

use super::{leafboxtransformer::{LeafBoxTransformer, LeafGeometry}, maintrackspec::MTSpecifier, allotmentbox::AllotmentBox};

pub struct CollisionNodeRequest {
    metadata: AllotmentMetadata,
    group: Option<String>,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafBoxTransformer>>>>,
    geometry: LeafGeometry
}

impl CollisionNodeRequest {
    fn new(metadata: &AllotmentMetadata, group: &Option<String>, geometry: &LeafGeometry) -> CollisionNodeRequest {
        CollisionNodeRequest {
            metadata: metadata.clone(),
            group: group.clone(),
            requests: Mutex::new(HashMap::new()),
            geometry: geometry.clone()
        }
    }
}

impl LinearGroupEntry for CollisionNodeRequest {
    fn allot(&self, geometry: &LeafGeometry, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, secondary_store: &SecondaryPositionResolver) {
        let mut allot_box = AllotmentBox::empty();
        let requests = lock!(self.requests);
        for (specifier,request) in requests.iter() {
            if specifier.sized() {
                allot_box = allot_box.merge(&AllotmentBox::new(request.metadata(),request.max_used()));
            }
        }
        let total_offset = offset.size() + allot_box.top_space();
        for (specifier,request) in requests.iter() {
            let our_secondary = specifier.base().secondary().as_ref().and_then(|s| secondary_store.lookup(s));
            let transformer = LeafBoxTransformer::new(geometry,&our_secondary,total_offset,allot_box.height(),request.depth());
            request.set_allotment(Arc::new(transformer));
        }
        offset.advance(allot_box.height());
    }

    fn name_for_secondary(&self) -> &str { self.metadata.name() }

    fn priority(&self) -> i64 {
        let mut h = DefaultHasher::new();
        self.group.hash(&mut h);
        h.finish() as i64
    }

    fn make_request(&self, geometry: &LeafGeometry, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<crate::AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            Arc::new(AllotmentRequestImpl::new(&self.metadata,geometry,specifier.base().depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }

    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, _out: &mut Vec<AllotmentMetadata>) {}
}

pub(crate) struct CollisionNodeLinearHelper;

impl LinearGroupHelper for CollisionNodeLinearHelper {
    type Key = Option<String>;

    fn entry_key(&self, full_name: &str) -> Option<String> { MTSpecifier::new(&full_name).base().group().clone() }

    fn make_linear_group_entry(&self, geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(CollisionNodeRequest::new(&metadata,specifier.base().group(),geometry))
    }
}
