use std::{sync::Arc, collections::HashMap};

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleSolution, PuzzleValue};

use crate::allotment::{util::rangeused::RangeUsed, style::allotmentname::AllotmentName};

use super::concretebump::{ConcreteRequests, ConcreteBump};

#[derive(Clone)]
pub(crate) struct BumpRequest {
    name: AllotmentName,
    range: PuzzleValueHolder<RangeUsed<f64>>,
    height: PuzzleValueHolder<f64>
}

impl BumpRequest {
    fn add_to_concrete_requests(&self, solution: &PuzzleSolution, requests: &mut ConcreteRequests) {
        match self.range.get(solution).as_ref() {
            RangeUsed::None =>
                {},
            RangeUsed::All =>
                requests.add_infinite(&self.name,self.height.get(solution).ceil() as i64),
            RangeUsed::Part(start,end) =>
                requests.add_finite(&self.name,start.floor().max(0.) as u64,end.ceil().max(0.) as u64,self.height.get(solution).ceil() as i64),
        }
    }
}

pub(crate) struct BumpRequests {
    requests: Vec<BumpRequest>,
    index: usize
}

impl BumpRequests {
    pub(crate) fn new(index: usize) -> BumpRequests {
        BumpRequests {
            requests: vec![],
            index
        }
    }

    pub(crate) fn add(&mut self, name: &AllotmentName, range: &PuzzleValueHolder<RangeUsed<f64>>, height: &PuzzleValueHolder<f64>) {
        self.requests.push(BumpRequest {
            name: name.clone(),
            range: range.clone(),
            height: height.clone()
        })
    }

    fn add_concrete(&self, solution: &PuzzleSolution, concrete: &mut ConcreteBump) -> bool {
        let mut requests = concrete.new_requests(self.index);
        for request in &*self.requests {
            request.add_to_concrete_requests(solution,&mut requests);
        }
        concrete.add(requests)
    }

}

pub(crate) struct BumpProcess {
    bp_per_carriage: u64,
    carriages: HashMap<usize,BumpRequests>
}

impl BumpProcess {
    pub(crate) fn new(bp_per_carriage: u64) -> BumpProcess {
        BumpProcess {
            bp_per_carriage,
            carriages: HashMap::new()
        }
    }

    pub(crate) fn add_carriage(&mut self, requests: BumpRequests) {
        if !self.carriages.contains_key(&requests.index) {
            self.carriages.insert(requests.index,requests);
        }
    }

    pub(crate) fn remove_carriage(&mut self, carriage_index: usize) {
        self.carriages.remove(&carriage_index);
    }
}
