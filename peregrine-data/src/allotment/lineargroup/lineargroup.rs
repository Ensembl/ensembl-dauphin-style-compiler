use std::{collections::HashMap, hash::Hash, sync::{Arc}};

use crate::{AllotmentMetadataStore, AllotmentMetadata, AllotmentRequest, allotment::{tree::leaftransformer::LeafGeometry, core::arbitrator::Arbitrator}};

use super::{offsetbuilder::{LinearOffsetBuilder}};

pub trait LinearGroupEntry {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>);
    fn allot(&self, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, arbitrator: &mut Arbitrator);
    fn priority(&self) -> i64;
    fn make_request(&self, geometry: &LeafGeometry, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest>;
}

pub trait LinearGroupHelper {
    type Key : PartialEq + Eq + Hash + Clone;

    fn entry_key(&self, full_name: &str) -> Self::Key;
    fn make_linear_group_entry(&self, geometry: &LeafGeometry, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry>;
}

pub(crate) struct LinearGroup<C: LinearGroupHelper> {
    geometry: LeafGeometry,
    entries: HashMap<C::Key,Arc<dyn LinearGroupEntry>>,
    creator: Box<C>
}

impl<C: LinearGroupHelper> LinearGroup<C> {
    pub(crate) fn new(geometry: &LeafGeometry, creator: C) -> LinearGroup<C> {
        LinearGroup {
            geometry: geometry.clone(),
            entries: HashMap::new(),
            creator: Box::new(creator)
        }
    }

    fn get_entry_for(&mut self, allotment_metadata: &AllotmentMetadataStore, full_name: &str, full_path: &str) -> &Arc<dyn LinearGroupEntry> {
        let geometry = self.geometry.clone();
        let (creator,entries) = (&mut self.creator, &mut self.entries);
        entries.entry(creator.entry_key(full_name)).or_insert_with(|| {
            creator.make_linear_group_entry(&geometry,&allotment_metadata,&full_path)
        })
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str, full_path: &str) -> Option<AllotmentRequest> {
        let geometry = self.geometry.clone();
        self.get_entry_for(allotment_metadata,name,full_path).make_request(&geometry,allotment_metadata,name)
    }

    pub(crate) fn union(&mut self, other: &LinearGroup<C>) {
        for (base_name,request) in other.entries.iter() {
            if !self.entries.contains_key(base_name) {
                self.entries.insert(base_name.clone(),request.clone());
            }
        }
    }

    pub(crate) fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,entry) in &self.entries {
            entry.get_entry_metadata(allotment_metadata,out);
        }
    }

    pub(crate) fn allot(&mut self, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, arbitrator: &mut Arbitrator) {
        let mut sorted_requests = self.entries.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        for entry in sorted_requests {
            entry.allot(secondary,offset,arbitrator);
        }
    }
}
