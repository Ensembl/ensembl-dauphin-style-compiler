use std::{sync::{Arc, Mutex}, collections::{HashMap, HashSet}, mem};
use peregrine_toolkit::{lock};
use crate::allotment::{core::{allotmentname::AllotmentName}};
use super::{bumprequest::{BumpRequestSet, BumpRequest}, standardalgorithm::StandardAlgorithm};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct BumpResponses {
    pub(super) offset: f64,
    pub(super) total_height: f64,
    pub(super) value: Arc<Mutex<HashMap<AllotmentName,f64>>>
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

    fn make(&mut self) -> (Vec<AllotmentName>,HashMap<AllotmentName,BumpRequest>) {
        let mut request_order = vec![];
        let mut request_data = HashMap::new();
        let mut requests = mem::replace(&mut self.requests,vec![]);
        requests.sort_by_key(|r| r.index);
        for request in requests {
            self.real_add(&request,&mut request_data,&mut request_order);
        }
        (request_order,request_data)
    }

    pub(crate) fn make_standard(mut self, use_wall: bool) -> StandardAlgorithm {
        let (request_order,request_data) = self.make();
        StandardAlgorithm::new(request_data,request_order,self.indexes,use_wall)
    }
}
