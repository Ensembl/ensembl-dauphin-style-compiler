use std::{collections::HashMap, sync::{Arc}};

use crate::{AllotmentDirection, AllotmentMetadata, AllotmentMetadataStore, AllotmentRequest};

use super::{allotment::{AllotmentImpl, CoordinateSystem}, allotmentrequest::{AllotmentRequestImpl}};

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
    pub(crate) fn direction(&self) -> AllotmentDirection {
        match self {
            LinearRequestGroupName::Track => AllotmentDirection::Forward,
            LinearRequestGroupName::OverlayTop => AllotmentDirection::Forward,
            LinearRequestGroupName::OverlayBottom => AllotmentDirection::Reverse,
            LinearRequestGroupName::OverlayLeft => AllotmentDirection::Forward,
            LinearRequestGroupName::OverlayRight => AllotmentDirection::Reverse,
            LinearRequestGroupName::Screen(i) => AllotmentDirection::Forward
        }
    }

    pub(crate) fn coord_system(&self) -> CoordinateSystem {
        match self {
            LinearRequestGroupName::Track => CoordinateSystem::Track,
            LinearRequestGroupName::OverlayTop => CoordinateSystem::Space,
            LinearRequestGroupName::OverlayBottom => CoordinateSystem::Space,
            LinearRequestGroupName::OverlayLeft => CoordinateSystem::Base,
            LinearRequestGroupName::OverlayRight => CoordinateSystem::Base,
            LinearRequestGroupName::Screen(i) => CoordinateSystem::Window
        }
    }
}

pub trait LinearAllotmentImpl : AllotmentImpl {
    fn max(&self) -> i64;
    fn up(self: Arc<Self>) -> Arc<dyn LinearAllotmentImpl>;
}

pub trait LinearGroupEntry {
    fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>);
    fn make(&self, offset: i64) -> i64;
    fn name(&self) -> &str;
    fn priority(&self) -> i64;
    fn make_request(&self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest>;
}

pub trait AsAllotmentRequestImpl {
    fn up(value: Arc<Self>) -> Arc<dyn AllotmentRequestImpl>;
}

pub trait LinearAllotmentRequestCreatorImpl {
    fn base(&self, name: &str) -> String;
    fn make(&self, metadata: &AllotmentMetadataStore, base: &str) -> Arc<dyn LinearGroupEntry>;
}

pub(super) struct LinearRequestGroup<C> {
    requests: HashMap<String,Arc<dyn LinearGroupEntry>>,
    creator: Box<C>,
    max: i64
}

impl<C: LinearAllotmentRequestCreatorImpl> LinearRequestGroup<C> {
    pub(super) fn new(creator: C) -> LinearRequestGroup<C> {
        LinearRequestGroup {
            requests: HashMap::new(),
            creator: Box::new(creator),
            max: 0
        }
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let base_name = self.creator.base(name);
        if !self.requests.contains_key(&base_name) {
            let request = self.creator.make(&allotment_metadata,&base_name);
            self.requests.insert(base_name.to_string(),request);
        }
        let entry = self.requests.get(&base_name);
        if entry.is_none() { return None; }
        let entry = entry.unwrap();
        entry.make_request(allotment_metadata,name)
    }

    pub(super) fn union(&mut self, other: &LinearRequestGroup<C>) {
        for (base_name,request) in other.requests.iter() {
            if !self.requests.contains_key(base_name) {
                self.requests.insert(base_name.clone(),request.clone());
            }
        }
    }

    pub(super) fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,request) in self.requests.iter() {
            request.get_all_metadata(allotment_metadata,out);
        }
    }

    pub(super) fn allot(&mut self) {
        let mut sorted_requests = self.requests.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        let mut offset = 0;
        for request in sorted_requests {
            offset += request.make(offset);
        }
        self.max = offset;
    }

    pub(super) fn max(&self) -> i64 { self.max }
}
