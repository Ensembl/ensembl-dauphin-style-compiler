use std::{collections::HashMap, hash::Hash, sync::{Arc}};

use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentMetadataStore, AllotmentMetadata, AllotmentRequest, allotment::{tree::{ allotmentbox::AllotmentBox}, core::arbitrator::Arbitrator}, CoordinateSystem};

pub trait LinearGroupEntry {
    fn get_entry_metadata(&self, solution: &PuzzleSolution, _allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>);
    fn bump(&self, arbitrator: &mut Arbitrator);
    fn allot(&self, arbitrator: &mut Arbitrator) -> AllotmentBox;
    fn priority(&self) -> i64;
    fn make_request(&self, geometry: &CoordinateSystem, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest>;
}

pub trait LinearGroupHelper {
    type Key : PartialEq + Eq + Hash + Clone;
    type Value: LinearGroupEntry + 'static;

    fn pre_bump<'a>(&self, _arbitrator: &'a Arbitrator<'a>) -> Option<Arbitrator<'a>> { None }
    fn bump(&self, arbitrator: &mut Arbitrator);
    fn entry_key(&self, full_name: &str) -> Self::Key;
    fn make_linear_group_entry(&self, geometry: &CoordinateSystem, metadata: &AllotmentMetadataStore, full_path: &str) -> Arc<Self::Value>;
}

pub(crate) struct LinearGroup<C: LinearGroupHelper> where C::Value: 'static {
    geometry: CoordinateSystem,
    entries: HashMap<C::Key,Arc<C::Value>>,
    creator: Box<C>
}

impl<C: LinearGroupHelper> LinearGroup<C> {
    pub(crate) fn new(geometry: &CoordinateSystem, creator: C) -> LinearGroup<C> {
        LinearGroup {
            geometry: geometry.clone(),
            entries: HashMap::new(),
            creator: Box::new(creator)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(&C::Key,&Arc<C::Value>)> { self.entries.iter() }

    fn get_entry_for(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> &Arc<C::Value> {
        let geometry = self.geometry.clone();
        let (creator,entries) = (&mut self.creator, &mut self.entries);
        entries.entry(creator.entry_key(name)).or_insert_with(|| {
            creator.make_linear_group_entry(&geometry,&allotment_metadata,&name)
        })
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let geometry = self.geometry.clone();
        self.get_entry_for(allotment_metadata,name).make_request(&geometry,allotment_metadata,name)
    }

    pub(crate) fn union(&mut self, other: &LinearGroup<C>) {
        for (base_name,request) in other.entries.iter() {
            if !self.entries.contains_key(base_name) {
                self.entries.insert(base_name.clone(),request.clone());
            }
        }
    }

    pub(crate) fn get_all_metadata(&self, solution: &PuzzleSolution, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,entry) in &self.entries {
            entry.get_entry_metadata(solution,allotment_metadata,out);
        }
    }

    fn real_bump(&mut self, arbitrator: &mut Arbitrator) {
        for (_,entry) in &self.entries {
            entry.bump(arbitrator);
        }
        self.creator.bump(arbitrator);
    }

    pub(crate) fn bump(&mut self, arbitrator: &mut Arbitrator) {
        if let Some(mut sub_arbitrator) = self.creator.pre_bump(arbitrator) {
            self.real_bump(&mut sub_arbitrator);
        } else {
            self.real_bump(arbitrator);
        }
    }

    pub(crate) fn allot(&mut self, arbitrator: &mut Arbitrator) -> Vec<AllotmentBox> {
        let mut out = vec![];
        let mut sorted_requests = self.entries.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        for entry in sorted_requests {
            out.push(entry.allot(arbitrator));
        }
        out
    }
}
