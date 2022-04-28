use std::{sync::{Arc, Mutex}, collections::{HashMap, btree_map::Range}, f32::consts::PI};
use peregrine_toolkit::{lock, log};
use peregrine_toolkit::skyline::Skyline;
use crate::allotment::{style::allotmentname::AllotmentName, util::rangeused::RangeUsed};
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
        BumpRequestSetBuilder::new(self.index as f64)
    }
}

pub struct BumpRequestSetBuilder {
    members: Vec<BumpRequest>,
    delta: f64
}

impl BumpRequestSetBuilder {
    fn new(delta: f64) -> BumpRequestSetBuilder {
        BumpRequestSetBuilder { members: vec![], delta }
    }

    pub(crate) fn add(&mut self, mut req: BumpRequest) {
        if req.add_delta(self.delta) {
            self.members.push(req);
        }
    }
}

#[derive(Clone)]
pub struct BumpRequestSet {
    values: Arc<BumpRequestSetBuilder>,
    identity: u64
}

impl BumpRequestSet {
    pub(crate) fn new(builder: BumpRequestSetBuilder) -> BumpRequestSet {
        BumpRequestSet {
            values: Arc::new(builder),
            identity: IDS.next()
        }
    }

    pub(crate) fn identity(&self) -> u64 { self.identity }
}

#[derive(Clone)]
pub struct BumpResponses {
    offset: f64,
    total_height: f64,
    value: Arc<HashMap<AllotmentName,f64>>
}

impl BumpResponses {
    pub(crate) fn get(&self, name: &AllotmentName) -> f64 {
        self.value.get(name).copied().map(|x| x+self.offset).unwrap_or(0.)
    }

    pub(crate) fn height(&self) -> f64 {
        self.total_height
    }
}

struct CollisionAlgorithm2 {
    requests: Vec<AllotmentName>,
    request_data: HashMap<AllotmentName,BumpRequest>,
    tiebreak: usize,
    skyline: Skyline,
    value: HashMap<AllotmentName,f64>,
    substrate: u64
}

impl CollisionAlgorithm2 {
    fn new() -> CollisionAlgorithm2 {
        CollisionAlgorithm2 {
            requests: vec![],
            request_data: HashMap::new(),
            tiebreak: 0,
            skyline: Skyline::new(),
            value: HashMap::new(),
            substrate: 0
        }
    }

    fn add(&mut self, requests: &BumpRequestSet) {
        for request in &requests.values.members {
            if let Some(old) = self.request_data.get(&request.name) {
                let new_range = request.range.merge(&old.range);
                let new_height = request.height.max(old.height);
                let new_request = BumpRequest::new(&request.name,&new_range,new_height);
                self.request_data.insert(request.name.clone(),new_request);
            } else {
                self.requests.push(request.name.clone());
                self.request_data.insert(request.name.clone(),request.clone());
            }
        }
    }

    fn bump(&mut self) {
        for request_name in &self.requests {
            let request = self.request_data.get(request_name).unwrap();
            match &request.range {
                RangeUsed::None => {
                    self.value.insert(request.name.clone(),0.);
                },
                RangeUsed::All => {
                    self.value.insert(request.name.clone(),self.substrate as f64);
                    self.substrate += request.height.round() as u64;
                },
                RangeUsed::Part(a,b) => {
                    log!("Part({},{})",a,b);
                    let interval = (*a as i64)..(*b as i64);
                    let part = Part::new(&request.name,&interval,request.height,self.tiebreak);
                    self.tiebreak += 1;
                    let height = part.watermark_add(&mut self.skyline);
                    log!("{:?}: Part({},{}) -> {}",part.name().sequence(),a,b,height);
                    self.value.insert(part.name().clone(),height);
                }
            }
        }
    }

    fn build(self) -> BumpResponses {
        //log!("built: {:?}",self.value);
        BumpResponses {
            offset: self.substrate as f64,
            total_height: self.substrate as f64 + self.skyline.max_height(),
            value: Arc::new(self.value)
        }
    }
}

pub(crate) fn bump(input: &[&BumpRequestSet]) -> BumpResponses {
    let mut input = input.iter().cloned().collect::<Vec<_>>();
    let mut bumper = CollisionAlgorithm2::new();
    input.sort();
    for set in input {
        bumper.add(&set);
    }
    bumper.bump();
    bumper.build()
}
