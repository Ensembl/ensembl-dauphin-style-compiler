use std::{collections::HashMap, sync::{Arc}};
use crate::{AllotmentMetadata, AllotmentMetadataStore, AllotmentRequest, allotment::{allotment::AllotmentImpl}};

use super::{secondary::{SecondaryPositionStore}, offsetbuilder::LinearOffsetBuilder};

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

pub trait LinearGroupEntryCreator {
    fn is_reverse(&self) -> bool;
    fn base(&self, name: &str) -> String;
    fn make_linear_group_entry(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry>;
}

pub(crate) struct LinearGroup<C> {
    entries: HashMap<String,Arc<dyn LinearGroupEntry>>,
    creator: Box<C>,
    max: i64
}

impl<C: LinearGroupEntryCreator> LinearGroup<C> {
    pub(crate) fn new(creator: C) -> LinearGroup<C> {
        LinearGroup {
            entries: HashMap::new(),
            creator: Box::new(creator),
            max: 0
        }
    }

    fn get_entry_for(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str, full_path: &str) -> &Arc<dyn LinearGroupEntry> {
        let (creator,entries) = (&mut self.creator, &mut self.entries);
        entries.entry(creator.base(name)).or_insert_with(|| {
            creator.make_linear_group_entry(&allotment_metadata,&full_path)
        })
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str, full_path: &str) -> Option<AllotmentRequest> {
        self.get_entry_for(allotment_metadata,name,full_path).make_request(allotment_metadata,name)
    }

    pub(crate) fn union(&mut self, other: &LinearGroup<C>) {
        for (base_name,request) in other.entries.iter() {
            if !self.entries.contains_key(base_name) {
                self.entries.insert(base_name.clone(),request.clone());
            }
        }
    }

    pub(crate) fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,request) in self.entries.iter() {
            request.get_all_metadata(allotment_metadata,out);
        }
    }

    pub(crate) fn allot(&mut self, secondary: i64, offset: &mut LinearOffsetBuilder, secondary_store: &mut SecondaryPositionStore) {
        let mut sorted_requests = self.entries.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        for entry in sorted_requests {
            let offset_amt = if self.creator.is_reverse() { offset.rev() } else { offset.fwd() };
            let size = entry.make(secondary,offset_amt,secondary_store);
            secondary_store.add(entry.name(),offset_amt, size,self.creator.is_reverse());
            offset.advance_fwd(size);
            if self.creator.is_reverse() { offset.advance_rev(size); }
        }
        self.max = offset.fwd();
    }

    pub(crate) fn max(&self) -> i64 { self.max }
}
