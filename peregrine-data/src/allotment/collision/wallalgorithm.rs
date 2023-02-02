use std::{collections::{HashMap, HashSet}, ops::Range};
use peregrine_toolkit::lock;
use crate::allotment::core::allotmentname::AllotmentName;
use super::{bumprequest::{BumpRequest, BumpRequestSet}, bumpprocess::GenericBumpingAlgorithm, algorithmbuilder::BumpResponses};

pub(crate) struct WallAlgorithm {
    indexes: Option<Range<usize>>,
}

impl WallAlgorithm {
    fn to_range(indexes: HashSet<usize>) -> Option<Range<usize>> {
        let mut indexes = indexes.iter().cloned().collect::<Vec<_>>();
        indexes.sort();
        if indexes.len() == 0 { return None; }
        let mut prev = None;
        for index in &indexes {
            if let Some(prev) = prev {
                if prev != *index - 1 { return None; }
            }
            prev = Some(*index);
        }
        let start = indexes[0];
        Some(start..start+indexes.len())
    }

    pub(super) fn new(requests: HashMap<AllotmentName,BumpRequest>, request_order: Vec<AllotmentName>, indexes: HashSet<usize>) -> WallAlgorithm {
        WallAlgorithm {
            indexes: Self::to_range(indexes),
        }
    }

    fn separate_preexisting(&self, requests: &BumpRequestSet) -> (Vec<(BumpRequest,BumpRequest,f64)>,Vec<BumpRequest>) {
        let values = lock!(self.value);
        let (mut old,mut new) = (vec![],vec![]);
        for request in requests.values.iter() {
            if let Some(existing) = self.requests.get(&request.name) {
                let value = values.get(&request.name).copied().unwrap_or(0.);
                old.push((existing.clone(),request.clone(),value));
            } else {
                new.push(request.clone());
            }
        }
        (old,new)
    }

    fn in_range(&self, index: usize) -> bool {
        self.indexes.as_ref().map(|range| {
            index >= range.start && index < range.end
        }).unwrap_or(false)
    }

    fn update_range(&mut self, index: usize) -> bool {
        if self.in_range(index) { return true; }
        if let Some(range) = &mut self.indexes {
            if range.start == index+1 {
                range.start -= 1;
            } else if range.end == index {
                range.end += 1;
            } else {
                return false;
            }
        }
        true
    }
}

impl GenericBumpingAlgorithm for WallAlgorithm {
    fn add(&mut self, requests: &BumpRequestSet) -> bool {
        /* seen already */
        if self.in_range(requests.index) { return true; }
        /* 1. We cannot add in a bridging fashion, bail.*/
        if !self.update_range(requests.index) { return false; }
        /* 2. For everything with pre-existing value */
        let (old,new) = self.separate_preexisting(requests);
        todo!()
    }

    fn build(&self) -> BumpResponses {
        todo!()
    }
}    

