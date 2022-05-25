use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet}, ops::Range, mem};
use peregrine_toolkit::{lock};
use peregrine_toolkit::skyline::Skyline;
use crate::allotment::{util::rangeused::RangeUsed, core::allotmentname::AllotmentName};
use super::bumppart::Part;

use lazy_static::lazy_static;
use identitynumber::{identitynumber, hashable, orderable};

/* Too avoid too much rearranging when we tidy up unused BumpRequestSets, we always
 * add their contents in the same order and the caller also add the BumpRequestSets
 * themselves in the same order. To allow this, an ordering is provided on BumpRequestSets.
 * 
 * This should mean that items tend to "staywhere they are" except when it is
 * advanatgeous to move them (down) due to now dead big teeting off-screen stacks.
 */

const PIXEL_PRECISION : f64 = 1000000.;

#[derive(Clone)]
pub struct BumpRequest {
    name: AllotmentName,
    range: RangeUsed<f64>,
    height: f64
}

impl BumpRequest {
    pub fn new(name: &AllotmentName, range: &RangeUsed<f64>, height: f64) -> BumpRequest {
        BumpRequest {
            name: name.clone(),
            range: range.clone(),
            height
        }
    }

    fn add_delta(&mut self, delta: f64) -> bool {
        match &mut self.range {
            RangeUsed::Part(a,b) => {
                *a += delta;
                *b += delta;
                *a *= PIXEL_PRECISION;
                *b *= PIXEL_PRECISION;
                true
            },
            RangeUsed::All => { true },
            RangeUsed::None => { false }
        }
    }
}

identitynumber!(IDS);
hashable!(BumpRequestSet,identity);
orderable!(BumpRequestSet,identity);

#[derive(Clone)]
pub struct BumpRequestSetFactory {
    index: usize
}

impl BumpRequestSetFactory {
    pub(crate) fn new(index: usize) -> BumpRequestSetFactory {
        BumpRequestSetFactory { index }
    }

    pub(crate) fn builder(&self) -> BumpRequestSetBuilder {
        BumpRequestSetBuilder::new(self.index)
    }
}

pub struct BumpRequestSetBuilder {
    members: Vec<BumpRequest>,
    index: usize
}

impl BumpRequestSetBuilder {
    fn new(index: usize) -> BumpRequestSetBuilder {
        BumpRequestSetBuilder { members: vec![], index }
    }

    pub(crate) fn add(&mut self, mut req: BumpRequest) {
        if req.add_delta(self.index as f64) {
            self.members.push(req);
        }
    }
}

#[derive(Clone)]
pub struct BumpRequestSet {
    values: Arc<Vec<BumpRequest>>,
    index: usize,
    identity: u64
}

impl BumpRequestSet {
    pub(crate) fn new(mut builder: BumpRequestSetBuilder) -> BumpRequestSet {
        builder.members.sort_by(|b,a| a.range.partial_cmp(&b.range).unwrap());
        BumpRequestSet {
            values: Arc::new(builder.members),
            identity: IDS.next(),
            index: builder.index
        }
    }

    pub(crate) fn index(&self) -> usize { self.index }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct BumpResponses {
    offset: f64,
    total_height: f64,
    value: Arc<Mutex<HashMap<AllotmentName,f64>>>
}

impl BumpResponses {
    pub(crate) fn get(&self, name: &AllotmentName) -> f64 {
        lock!(self.value).get(name).copied().map(|x| x+self.offset).unwrap_or(0.)
    }

    pub(crate) fn height(&self) -> f64 {
        self.total_height
    }
}

pub(crate) struct AlgorithmBuilder {
    indexes: HashSet<usize>,
    requests: Vec<BumpRequestSet>,
}

impl AlgorithmBuilder {
    pub(crate) fn new() -> AlgorithmBuilder {
        AlgorithmBuilder {
            indexes: HashSet::new(),
            requests: vec![],
        }
    }

    fn real_add(&self, requests: &BumpRequestSet, request_data: &mut HashMap<AllotmentName,BumpRequest>, request_order: &mut Vec<AllotmentName>) {
        for request in requests.values.iter() {
            if let Some(old) = request_data.get(&request.name) {
                let new_range = request.range.merge(&old.range);
                let new_height = request.height.max(old.height);
                let new_request = BumpRequest::new(&request.name,&new_range,new_height);
                request_data.insert(request.name.clone(),new_request);
            } else {
                request_order.push(request.name.clone());
                request_data.insert(request.name.clone(),request.clone());
            }
        }
    }

    pub(crate) fn add(&mut self, requests: &BumpRequestSet) {
        self.indexes.insert(requests.index);
        self.requests.push(requests.clone());
    }

    pub(crate) fn make(mut self) -> Algorithm {
        let mut request_order = vec![];
        let mut request_data = HashMap::new();
        let mut requests = mem::replace(&mut self.requests,vec![]);
        requests.sort_by_key(|r| r.index);
        for request in requests {
            self.real_add(&request,&mut request_data,&mut request_order);
        }
        Algorithm::new(request_data,request_order,self.indexes)
    }
}

pub(crate) struct Algorithm {
    indexes: Option<Range<usize>>,
    requests: HashMap<AllotmentName,BumpRequest>,
    value: Arc<Mutex<HashMap<AllotmentName,f64>>>,
    skyline: Skyline,
    substrate: u64
}

impl Algorithm {
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

    fn new(requests: HashMap<AllotmentName,BumpRequest>, request_order: Vec<AllotmentName>, indexes: HashSet<usize>) -> Algorithm {
        let mut out = Algorithm {
            requests: HashMap::new(),
            indexes: Self::to_range(indexes),
            skyline: Skyline::new(),
            value: Arc::new(Mutex::new(HashMap::new())),
            substrate: 0
        };
        for name in &request_order {
            let request = requests.get(name).unwrap();
            out.requests.insert(name.clone(),request.clone());
            out.bump_one(request);
        }
        out
    }

    fn bump_one(&mut self, request: &BumpRequest) {
        let mut value = lock!(self.value);
        match &request.range {
            RangeUsed::None => {
                value.insert(request.name.clone(),0.);
            },
            RangeUsed::All => {
                value.insert(request.name.clone(),self.substrate as f64);
                self.substrate += request.height.round() as u64;
            },
            RangeUsed::Part(a,b) => {
                let interval = (*a as i64)..(*b as i64);
                let part = Part::new(&request.name,&interval,request.height);
                let height = part.watermark_add(&mut self.skyline);
                value.insert(part.name().clone(),height);
            }
        }
    }

    /* With care we can often extend an existing Algorithm with a new carriage. This is of
     * practical significance because it prevents a TrainState change which means that an
     * awful lot of layout and rendering code need not be rerun in these cases.
     * 
     * 1. We cannot add in a bridging fashion, bail.
     * 2. For everything with pre-existing value:
     *    a. if height is increased, bail;
     *    b. if finite/infinite mismatch, bail;
     *    c. adjust skyline to at least reach point.
     * 3. For everything else 
     *    a. if any new infinite, bail;
     *    b. proceed adding as normal.
     *
     */
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

    fn add_old(&mut self, old: &[(BumpRequest,BumpRequest,f64)]) -> bool {
        for (existing_req,incoming_req,offset) in old {
            /* 2a. if height is increased, bail */
            if incoming_req.height > existing_req.height { return false; }
            /* 2b. if finite/infinite mismatch, bail */
            match (&incoming_req.range,&existing_req.range) {
                (RangeUsed::All,RangeUsed::All) => {},
                (RangeUsed::All,_) => { return false; },
                (_,RangeUsed::All) => { return false; },
                (_,_) => {}
            }
            /* 2c. adjust skyline to at least reach point */
            if let RangeUsed::Part(start,end) = incoming_req.range {
                self.skyline.set_min(start as i64,end as i64,*offset+existing_req.height);
            }
        }
        true
    }

    fn add_new(&mut self, new: &[BumpRequest]) -> bool {
        for req in new {
            /* 3a. if any new infinite, bail */
            match req.range {
                RangeUsed::All => { return false; },
                _ => {}
            }
            /* 3b. proceed adding as normal */
            self.requests.insert(req.name.clone(),req.clone());
            self.bump_one(req);
        }
        true
    }

    pub(crate) fn add(&mut self, requests: &BumpRequestSet) -> bool {
        /* seen already */
        if self.in_range(requests.index) { return true; }
        /* 1. We cannot add in a bridging fashion, bail.*/
        if !self.update_range(requests.index) { return false; }
        /* 2. For everything with pre-existing value */
        let (old,new) = self.separate_preexisting(requests);
        if !self.add_old(&old) { return false; }
        /* 3. For everything else */
        if !self.add_new(&new) { return false; }
        true
    }

    pub(crate) fn build(&self) -> BumpResponses {
        BumpResponses {
            offset: self.substrate as f64,
            total_height: self.substrate as f64 + self.skyline.max_height(),
            value: self.value.clone()
        }
    }
}
