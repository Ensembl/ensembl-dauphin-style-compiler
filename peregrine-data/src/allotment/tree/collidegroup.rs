use std::{collections::{HashMap}, sync::{Arc, Mutex}};
use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, allotment::{lineargroup::{lineargroup::{LinearGroupEntry, LinearGroupHelper}}, core::{allotmentrequest::{AllotmentRequestImpl, GenericAllotmentRequestImpl}, arbitrator::{Arbitrator}, rangeused::RangeUsed}}, CoordinateSystem};

use super::{allotmentbox::{AllotmentBox, AllotmentBoxBuilder}, maintrackspec::MTSpecifier, collisionalgorithm::{CollisionToken}};

pub struct CollideGroupRequest {
    metadata: AllotmentMetadata,
    requests: Mutex<HashMap<MTSpecifier,Arc<AllotmentRequestImpl>>>,
    bump_token: Mutex<Option<CollisionToken>>
}

impl CollideGroupRequest {
    fn new(metadata: &AllotmentMetadata) -> CollideGroupRequest {
        CollideGroupRequest {
            metadata: metadata.clone(),
            requests: Mutex::new(HashMap::new()),
            bump_token: Mutex::new(None)
        }
    }

    pub(crate) fn add_allotment_metadata_values(&self, out: &mut AllotmentMetadataRequest) {
        let requests = lock!(self.requests);
        for request in requests.values() {
            request.add_allotment_metadata_values(out);
        }
    }

    fn make_content_box(&self, specifier: &MTSpecifier, request: &AllotmentRequestImpl, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let box_builder = AllotmentBoxBuilder::empty(request.max_y(),&specifier.arbitrator_horiz(arbitrator));
        let content_box = AllotmentBox::new(box_builder);
        request.set_allotment(Arc::new(content_box.clone()));
        content_box
    }

    fn make_child_box(&self, specifier: &MTSpecifier, request: &AllotmentRequestImpl, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let mut builder = AllotmentBoxBuilder::empty(0,&None);
        builder.add_padding_top(lock!(self.bump_token).as_ref().map(|x| x.get()).unwrap_or(0.) as i64);
        builder.append(self.make_content_box(specifier,request,arbitrator));
        AllotmentBox::new(builder)
    }
}

impl LinearGroupEntry for CollideGroupRequest {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, _out: &mut Vec<AllotmentMetadata>) {}

    fn priority(&self) -> i64 { self.metadata.priority() }

    fn make_request(&self, geometry: &CoordinateSystem, _allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let specifier = MTSpecifier::new(name);
        let mut requests = lock!(self.requests);
        let req_impl = requests.entry(specifier.clone()).or_insert_with(|| {
            let our_geometry = specifier.our_geometry(geometry);
            Arc::new(AllotmentRequestImpl::new(&self.metadata,&our_geometry,specifier.base().depth(),!specifier.sized()))
        });
        Some(AllotmentRequest::upcast(req_impl.clone()))
    }

    fn bump(&self, arbitrator: &mut Arbitrator) {
        let requests = lock!(self.requests);
        let mut range_used = RangeUsed::None;
        let mut max_height = 0_f64;
        for request in requests.values() {
            let full_range = if request.coord_system().is_tracking() {
                arbitrator.full_pixel_range(&request.base_range(),&request.pixel_range())
            } else {
                request.pixel_range()
            };
            max_height = max_height.max(request.max_y() as f64);
            range_used = range_used.merge(&full_range);
        }
        *lock!(self.bump_token) = Some(arbitrator.bumper().add_entry(&range_used,max_height));
    }

    fn allot(&self, arbitrator: &mut Arbitrator) -> AllotmentBox {
        let requests = lock!(self.requests);
        let mut child_boxes = vec![];
        for (specifier,request) in requests.iter() {
            let child_box = self.make_child_box(specifier,request,arbitrator);
            child_boxes.push(child_box);
        }
        let mut builder = AllotmentBoxBuilder::empty(0,&None);
        builder.overlay_all(child_boxes);
        AllotmentBox::new(builder)
    }
}

pub struct CollideGroupLinearHelper;

impl LinearGroupHelper for CollideGroupLinearHelper {
    type Key = Option<String>;
    type Value = CollideGroupRequest;

    fn make_linear_group_entry(&self, _geometry: &CoordinateSystem, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<CollideGroupRequest> {
        let specifier = MTSpecifier::new(full_path);
        let name = specifier.base().name();
        let metadata = metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
        Arc::new(CollideGroupRequest::new(&metadata))
    }

    fn entry_key(&self, name: &str) -> Option<String> {
        let specifier = MTSpecifier::new(name);
        specifier.base().group().clone()
    }

    fn pre_bump<'a>(&self, arbitrator: &'a Arbitrator<'a>) -> Option<Arbitrator<'a>> {
        Some(arbitrator.make_sub_arbitrator())
    }

    fn bump(&self, arbitrator: &mut Arbitrator) {
        arbitrator.bumper().bump();
    }
}
