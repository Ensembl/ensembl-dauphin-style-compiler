use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet}};
use peregrine_toolkit::{skyline::Skyline, lock, boom::Boom};
use crate::allotment::core::{allotmentname::AllotmentName, rangeused::RangeUsed};
use super::{bumprequest::{BumpRequest, BumpRequestSet}, bumppart::Part, algorithmbuilder::BumpResponses, bumpprocess::GenericBumpingAlgorithm, wall::Wall};

enum AlgorithmDetails {
    Bumper(Skyline),
    Wall(Wall)
}

impl AlgorithmDetails {
    fn verify(&mut self, requests: &[BumpRequest]) -> bool {
        match self {
            AlgorithmDetails::Bumper(_) => true,
            AlgorithmDetails::Wall(w) => w.verify(requests)
        }
    }
    
    fn total_height(&self) -> f64 {
        match self {
            AlgorithmDetails::Bumper(b) => b.max_height(),
            AlgorithmDetails::Wall(w) => w.total_height()
        }
    }

    fn renew(&mut self, start: i64, end: i64, offset: f64, height: f64) {
        match self {
            AlgorithmDetails::Bumper(b) => b.set_min(start,end,offset+height),
            AlgorithmDetails::Wall(w) => w.renew(start,end,offset)
        }
    }

    fn allocate(&mut self, start: i64, end: i64, height: f64) -> f64 {
        match self {
            AlgorithmDetails::Bumper(b) => b.add(start,end,height),
            AlgorithmDetails::Wall(w) => w.allocate(start,end)
        }
    }
}

pub(crate) struct StandardAlgorithm {
    indexes: HashSet<usize>,
    requests: HashMap<AllotmentName,BumpRequest>,
    value: Arc<Mutex<HashMap<AllotmentName,f64>>>,
    details: AlgorithmDetails,
    substrate: u64
}

impl StandardAlgorithm {
    fn good_index(&self, requests: &BumpRequestSet) -> bool {
        self.indexes.len() == 0 ||
        self.indexes.contains(&(requests.index+1)) ||
        self.indexes.contains(&(requests.index-1))
    }

    pub(super) fn new(requests: HashMap<AllotmentName,BumpRequest>, request_order: Vec<AllotmentName>, indexes: HashSet<usize>, use_wall: bool) -> StandardAlgorithm {
        let details = if use_wall {
            AlgorithmDetails::Wall(Wall::new())
        } else {
            AlgorithmDetails::Bumper(Skyline::new())
        };
        let mut out = StandardAlgorithm {
            requests: HashMap::new(),
            indexes,
            details,
            value: Arc::new(Mutex::new(HashMap::new())),
            substrate: 0
        };
        let values = requests.values().cloned().collect::<Vec<_>>();
        out.details.verify(&values);
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
                let (start,end,height) = part.shape();
                let height = self.details.allocate(start,end,height);
                value.insert(part.name().clone(),height);
            }
        }
    }

    /* With care we can often extend an existing StandardAlgorithm with a new carriage. This is of
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
                self.details.renew(start as i64,end as i64,*offset,existing_req.height);
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
}

impl GenericBumpingAlgorithm for StandardAlgorithm {
    fn add(&mut self, requests: &BumpRequestSet) -> bool {
        /* seen already */
        if self.indexes.contains(&requests.index) { return true; }
        /* 1. We cannot add in a bridging fashion, bail.*/
        if !self.good_index(requests) { return false; }
        if !self.details.verify(&requests.values) { return false; }
        self.indexes.insert(requests.index);
        /* 1. Algorithm-specific rejection criteria */
        /* 2. For everything with pre-existing value */
        let (old,new) = self.separate_preexisting(requests);
        if !self.add_old(&old) { return false; }
        /* 3. For everything else */
        if !self.add_new(&new) { return false; }
        true
    }

    fn build(&self) -> BumpResponses {
        BumpResponses {
            offset: self.substrate as f64,
            total_height: self.substrate as f64 + self.details.total_height(),
            value: self.value.clone()
        }
    }
}    

