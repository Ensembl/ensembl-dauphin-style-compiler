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
    pub(crate) fn to_allotment_group(&self) -> AllotmentGroup {
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
    fn max(&self) -> i64;
    fn up(self: Arc<Self>) -> Arc<dyn LinearAllotmentImpl>;
}

pub trait LinearGroupEntry {
    fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>);
    fn make(&self, offset: i64);
    fn max(&self) -> i64;
    fn name(&self) -> &str;
    fn priority(&self) -> i64;
    fn make_request(&self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest>;
}

pub trait AsAllotmentRequestImpl {
    fn up(value: Arc<Self>) -> Arc<dyn AllotmentRequestImpl>;
}

pub trait LinearAllotmentRequestCreatorImpl {
    fn hash(&self, name: &str) -> u64;
    fn make(&self, metadata: &AllotmentMetadata) -> Arc<dyn LinearGroupEntry>;
}

pub(super) struct LinearRequestGroup<C> {
    requests: HashMap<u64,Arc<dyn LinearGroupEntry>>,
    creator: Box<C>
}

impl<C: LinearAllotmentRequestCreatorImpl> LinearRequestGroup<C> {
    pub(super) fn new(creator: C) -> LinearRequestGroup<C> {
        LinearRequestGroup {
            requests: HashMap::new(),
            creator: Box::new(creator)
        }
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let hash = self.creator.hash(name);
        if !self.requests.contains_key(&hash) {
            let metadata = allotment_metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
            let request = self.creator.make(&metadata);
            self.requests.insert(hash,request);
        }
        let entry = self.requests.get(&hash);
        if entry.is_none() { return None; }
        let entry = entry.unwrap();
        entry.make_request(allotment_metadata,name)
    }

    pub(super) fn union(&mut self, other: &LinearRequestGroup<C>) {
        for (hash,request) in other.requests.iter() {
            if !self.requests.contains_key(hash) {
                self.requests.insert(*hash,request.clone());
            }
        }
    }

    pub(super) fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,request) in self.requests.iter() {
            request.get_all_metadata(allotment_metadata,out);
        }
    }

    pub(super) fn apply_pitch(&self, pitch: &mut Pitch) {
        for (_,request) in &self.requests {
            pitch.set_limit(request.max());
        }
    }

    pub(super) fn allot(&mut self) {
        let mut sorted_requests = self.requests.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        let mut offset = 0;
        for request in sorted_requests {
            request.make(offset);
            offset = request.max();
        }
    }
}
