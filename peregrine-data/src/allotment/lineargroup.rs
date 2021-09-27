use std::{collections::HashMap, sync::{Arc}};
use crate::{AllotmentDirection, AllotmentGroup, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, Pitch};

use super::{allotment::AllotmentImpl, allotmentrequest::{AllotmentRequestImpl}};

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub(super) enum LinearRequestGroupName {
    Track,
    OverlayTop,
    OverlayBottom,
    OverlayLeft,
    OverlayRight,
    Screen(i8) // XXX
}

impl LinearRequestGroupName {
    fn to_allotment_group(&self) -> AllotmentGroup {
        match self {
            LinearRequestGroupName::Track => AllotmentGroup::Track,
            LinearRequestGroupName::OverlayTop => AllotmentGroup::BaseLabel(AllotmentDirection::Forward),
            LinearRequestGroupName::OverlayBottom => AllotmentGroup::BaseLabel(AllotmentDirection::Reverse),
            LinearRequestGroupName::OverlayLeft => AllotmentGroup::SpaceLabel(AllotmentDirection::Forward),
            LinearRequestGroupName::OverlayRight => AllotmentGroup::SpaceLabel(AllotmentDirection::Reverse),
            LinearRequestGroupName::Screen(i) => AllotmentGroup::Overlay(*i as i64)

        }
    }
}

pub trait LinearAllotmentImpl : AllotmentImpl {
    fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest);
    fn max(&self) -> i64;
    fn up(self: Arc<Self>) -> Arc<dyn LinearAllotmentImpl>;
}

pub trait LinearAllotmentRequestImpl : AllotmentRequestImpl {
    fn linear_allotment_impl(&self) -> Option<Arc<dyn LinearAllotmentImpl>>;
    fn make(&self, offset: i64, size: i64);
}

pub trait AsAllotmentRequestImpl {
    fn up(value: Arc<Self>) -> Arc<dyn AllotmentRequestImpl>;
}

pub trait LinearAllotmentRequestCreatorImpl {
    fn make(&self, metadata: &AllotmentMetadata, group: &AllotmentGroup) -> Arc<dyn LinearAllotmentRequestImpl>;
}

pub(super) struct LinearRequestGroup<C> {
    requests: HashMap<String,Arc<dyn LinearAllotmentRequestImpl>>,
    group: AllotmentGroup,
    creator: Box<C>
}

impl<C: LinearAllotmentRequestCreatorImpl> LinearRequestGroup<C> {
    pub(super) fn new(group: &LinearRequestGroupName, creator: C) -> LinearRequestGroup<C> {
        LinearRequestGroup {
            requests: HashMap::new(),
            group: group.to_allotment_group(),
            creator: Box::new(creator)
        }
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if !self.requests.contains_key(name) {
            let metadata = allotment_metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
            let request = self.creator.make(&metadata,&self.group);
            self.requests.insert(name.to_string(),request);
        }
        Some(AllotmentRequest::upcast(self.requests.get(name).unwrap().clone()))
    }

    pub(super) fn union(&mut self, other: &LinearRequestGroup<C>) {
        for (name,request) in other.requests.iter() {
            if !self.requests.contains_key(name) {
                self.requests.insert(name.to_string(),request.clone());
            }
        }
    }

    pub(super) fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,request) in self.requests.iter() {
            if let Some(this_metadata) = allotment_metadata.get(&request.name()) {
                let mut full_metadata = AllotmentMetadataRequest::rebuild(&this_metadata);
                if let Some(allotment) = request.linear_allotment_impl() {
                    allotment.add_metadata(&mut full_metadata);
                }
                out.push(AllotmentMetadata::new(full_metadata));
            }
        }
    }

    pub(super) fn apply_pitch(&self, pitch: &mut Pitch) {
        for (_,request) in &self.requests {
            if let Some(allotment) = request.linear_allotment_impl() {
                pitch.set_limit(allotment.max());
            }
        }
    }

    pub(super) fn allot(&mut self) {
        let mut sorted_requests = self.requests.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        let mut offset = 0;
        for request in sorted_requests {
            request.make(offset,64);
            offset += 64; // XXX
        }
    }
}
