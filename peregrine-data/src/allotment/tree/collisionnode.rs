use std::{sync::{Arc, Mutex}, collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}};

use peregrine_toolkit::lock;

use crate::{allotment::{lineargroup::{lineargroup::{LinearGroupHelper, LinearGroupEntry}, secondary::{SecondaryPositionStore}}, core::allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}}, AllotmentMetadataStore, AllotmentMetadata, AllotmentMetadataRequest, AllotmentRequest};

use super::{leafboxtransformer::LeafBoxTransformer, maintrackspec::MTSpecifier, treeallotment::{tree_best_offset, tree_best_height}};

pub struct CollisionNodeRequest {
    metadata: AllotmentMetadata,
    group: Option<String>,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl<LeafBoxTransformer>>>>,
    reverse: bool
}

impl CollisionNodeRequest {
    fn new(metadata: &AllotmentMetadata, group: &Option<String>, reverse: bool) -> CollisionNodeRequest {
        CollisionNodeRequest {
            metadata: metadata.clone(),
            group: group.clone(),
            requests: Mutex::new(HashMap::new()),
            reverse
        }
    }
}

impl LinearGroupEntry for CollisionNodeRequest {
    fn allot(&self, secondary: &Option<i64>, offset: i64, secondary_store: &SecondaryPositionStore) -> i64 {
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
            let our_secondary = specifier.get_secondary(secondary_store).or_else(|| secondary.clone());
            request.set_allotment(Arc::new(LeafBoxTransformer::new(&request.coord_system(),&our_secondary,offset,best_offset_val,best_height_val,specifier.base().depth(),self.reverse)));
        }
        best_height_val
    }

    fn name_for_secondary(&self) -> &str { self.metadata.name() }

    fn priority(&self) -> i64 {
        let mut h = DefaultHasher::new();
        self.group.hash(&mut h);
        h.finish() as i64
    }

    fn make_request(&self, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<crate::AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            Arc::new(AllotmentRequestImpl::new(&self.metadata,&specifier.coord_system(false),specifier.base().depth()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }

    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, _out: &mut Vec<AllotmentMetadata>) {}
}

pub(crate) struct CollisionNodeLinearHelper(pub bool);

impl LinearGroupHelper for CollisionNodeLinearHelper {
    type Key = Option<String>;

    fn entry_key(&self, full_name: &str) -> Option<String> { MTSpecifier::new(&full_name).base().group().clone() }

    fn make_linear_group_entry(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(CollisionNodeRequest::new(&metadata,specifier.base().group(),self.0))
    }
}
