use std::{collections::HashMap, hash::Hash, sync::{Arc}};

use crate::{AllotmentMetadataStore, AllotmentMetadata, AllotmentRequest};

use super::{secondary::{SecondaryPositionStore}, offsetbuilder::{LinearOffsetBuilder}};

/* A LinearGroup organises multiple requests along a linear axis and presents a single interface to the Universe.
 *
 * A LinearGroup has an associated LinearGroupHelper to allow type-specific behaviours. Specifically a LinearGroupHelper:
 * 1. can map from a spec to a key representing the corresponding entry at this level;
 * 2. can create new LinearGroupEntries with behaviour specific for the type;
 * 3. specifies whether the coordinates are top-to-bottom or bottom-to-top.
 * 
 * When requested the LinearGroupHelper creates something which implemenrs LinearGroupEntry. This object is stored
 * inside the linear group and:
 * 1. Can accept delegated allotment request creation decisions;
 * 2. Can return metadata to satisfy the universe's demands for metadata;
 * 3. Will be called via allot() with information concerning its final position once allotment takes place;
 * 4. Can return a priority for ordering decisions;
 * 5. Can return a name for use in secondaty axis allignment.
 */

pub trait LinearGroupEntry {
    fn get_entry_metadata(&self, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>);
    fn allot(&self, secondary: &Option<i64>, offset: i64, secondary_store: &SecondaryPositionStore) -> i64;
    fn name_for_secondary(&self) -> &str;
    fn priority(&self) -> i64;
    fn make_request(&self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest>;
}

pub trait LinearGroupHelper {
    type Key : PartialEq + Eq + Hash + Clone;

    fn entry_key(&self, full_name: &str) -> Self::Key;
    fn make_linear_group_entry(&self, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<dyn LinearGroupEntry>;
}

pub(crate) struct LinearGroup<C: LinearGroupHelper> {
    entries: HashMap<C::Key,Arc<dyn LinearGroupEntry>>,
    creator: Box<C>
}

impl<C: LinearGroupHelper> LinearGroup<C> {
    pub(crate) fn new(creator: C) -> LinearGroup<C> {
        LinearGroup {
            entries: HashMap::new(),
            creator: Box::new(creator)
        }
    }

    fn get_entry_for(&mut self, allotment_metadata: &AllotmentMetadataStore, full_name: &str, full_path: &str) -> &Arc<dyn LinearGroupEntry> {
        let (creator,entries) = (&mut self.creator, &mut self.entries);
        entries.entry(creator.entry_key(full_name)).or_insert_with(|| {
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
        for (_,entry) in &self.entries {
            entry.get_entry_metadata(allotment_metadata,out);
        }
    }

    pub(crate) fn allot(&mut self, secondary: &Option<i64>, offset: &mut LinearOffsetBuilder, secondary_store: &mut SecondaryPositionStore) {
        let mut sorted_requests = self.entries.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        for entry in sorted_requests {
            let offset_amt = offset.size();
            let size = entry.allot(secondary,offset_amt,secondary_store);
            secondary_store.add(entry.name_for_secondary(),offset_amt);
            offset.advance(size);
        }
    }
}
