use std::{collections::HashMap, sync::{Arc}};
use crate::{AllotmentMetadata, AllotmentMetadataStore, AllotmentRequest};
use super::{allotment::{AllotmentImpl}, allotmentrequest::{AllotmentRequestImpl}};

pub struct SecondaryPosition {
    pub offset: i64,
    pub size: i64,
    pub reverse: bool
}

pub struct SecondaryPositionStore {
    position: HashMap<String,SecondaryPosition>
}

impl SecondaryPositionStore {
    pub fn new() -> SecondaryPositionStore {
        SecondaryPositionStore {
            position: HashMap::new()
        }
    }

    pub fn add(&mut self, name: &str, position: SecondaryPosition) {
        self.position.insert(name.to_string(),position);
    }

    pub fn lookup(&self, name: &str) -> Option<&SecondaryPosition> {
        self.position.get(name)
    }
}

pub struct LinearOffsetBuilder {
    fwd: i64,
    rev: i64
}

impl LinearOffsetBuilder {
    pub fn new() -> LinearOffsetBuilder {
        LinearOffsetBuilder {
            fwd: 0,
            rev: 0
        }
    }

    pub fn advance_fwd(&mut self, amt: i64) {
        self.fwd += amt;
    }

    pub fn advance_rev(&mut self, amt: i64) {
        self.fwd += amt;
        self.rev += amt;
    }

    pub fn fwd(&self) -> i64 { self.fwd }
    pub fn rev(&self) -> i64 { self.rev }
}

pub trait LinearAllotmentImpl : AllotmentImpl {
    fn max(&self) -> i64;
    fn up(self: Arc<Self>) -> Arc<dyn LinearAllotmentImpl>;
}

pub trait LinearGroupEntry {
    fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>);
    fn make(&self, secondary: i64, offset: i64, secondary_store: &SecondaryPositionStore) -> i64;
    fn name(&self) -> &str;
    fn priority(&self) -> i64;
    fn make_request(&self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest>;
}

pub trait AsAllotmentRequestImpl {
    fn up(value: Arc<Self>) -> Arc<dyn AllotmentRequestImpl>;
}

pub trait LinearAllotmentRequestCreatorImpl {
    fn is_reverse(&self) -> bool;
    fn base(&self, name: &str) -> String;
    fn make(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry>;
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

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str, full_path: &str) -> Option<AllotmentRequest> {
        let base_name = self.creator.base(name);
        if !self.requests.contains_key(&base_name) {
            let request = self.creator.make(&allotment_metadata,&full_path);
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

    pub(super) fn allot(&mut self, secondary: i64, offset: &mut LinearOffsetBuilder, secondary_store: &mut SecondaryPositionStore) {
        let is_reverse = self.creator.is_reverse();
        let mut sorted_requests = self.requests.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        for request in sorted_requests {
            let name = request.name();
            if is_reverse {
                let more = request.make(secondary,offset.rev(),secondary_store);
                secondary_store.add(name,SecondaryPosition {
                    offset: offset.rev(),
                    size: more,
                    reverse: true
                });
                offset.advance_rev(more);
            } else {
                let more = request.make(secondary,offset.fwd(),secondary_store);
                secondary_store.add(name,SecondaryPosition {
                    offset: offset.fwd(),
                    size: more,
                    reverse: false
                });
                offset.advance_fwd(more);
            }
        }
        self.max = offset.fwd();
    }

    pub(super) fn max(&self) -> i64 { self.max }
}
