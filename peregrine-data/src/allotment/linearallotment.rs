use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{Allotment, AllotmentDirection, AllotmentGroup, AllotmentMetadata, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentRequest, DataMessage, Pitch, SpaceBasePointRef, spacebase::spacebase::SpaceBasePoint};

use super::{allotment::AllotmentImpl, allotmentrequest::AllotmentRequestImpl};


#[cfg_attr(debug_assertions,derive(Debug))]
pub struct LinearAllotment {
    metadata: AllotmentMetadata,
    direction: AllotmentDirection,
    offset: i64,
    size: i64
}

impl LinearAllotment {
    fn new(request: &LinearAllotmentRequest, offset: i64, size: i64) -> LinearAllotment {
        LinearAllotment {
            metadata: request.metadata.clone(),
            direction: request.direction(),
            offset,size
        }
    }

    fn max(&self) -> i64 { self.offset+self.size }

    fn add_metadata(&self, full_metadata: &mut AllotmentMetadataRequest) {
        full_metadata.add_pair("type","track");
        full_metadata.add_pair("offset",&self.offset.to_string());
        full_metadata.add_pair("height",&self.size.to_string());
    }
}

impl AllotmentImpl for LinearAllotment {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        output.normal += self.offset as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        let offset = self.offset as f64;
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }

    fn direction(&self) -> AllotmentDirection { self.direction.clone() }
}

pub struct LinearAllotmentRequest {
    metadata: AllotmentMetadata,
    allotment: Mutex<Option<Arc<LinearAllotment>>>
}

impl LinearAllotmentRequest {
    pub fn new(metadata: &AllotmentMetadata) -> LinearAllotmentRequest {
        LinearAllotmentRequest { metadata: metadata.clone(), allotment: Mutex::new(None) }
    }

    pub fn direction(&self) -> AllotmentDirection {
        self.metadata.allotment_group().direction()
    }

    pub fn make(&self, offset: i64, size: i64) {
        *self.allotment.lock().unwrap() = Some(Arc::new(LinearAllotment::new(&self,offset,size)));
    }

    pub fn linear_allotment(&self) -> Option<Arc<LinearAllotment>> {
        self.allotment.lock().unwrap().as_ref().cloned()
    }
}

impl AllotmentRequestImpl for LinearAllotmentRequest {
    fn name(&self) -> String { self.metadata.name().to_string() }
    fn allotment_group(&self) -> AllotmentGroup { self.metadata.allotment_group() }
    fn is_dustbin(&self) -> bool { false }
    fn priority(&self) -> i64 { self.metadata.priority() }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        Ok(Allotment::new(self.allotment.lock().unwrap().clone()
            .ok_or_else(|| DataMessage::AllotmentNotCreated(format!("name={}",self.metadata.name())))?))
    }
}

pub(super) struct LinearRequestGroup {
    requests: HashMap<String,Arc<LinearAllotmentRequest>>
}

impl LinearRequestGroup {
    pub(super) fn new() -> LinearRequestGroup {
        LinearRequestGroup {
            requests: HashMap::new()
        }
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if !self.requests.contains_key(name) {
            let metadata = allotment_metadata.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)));
            let request = Arc::new(LinearAllotmentRequest::new(&metadata));
            self.requests.insert(name.to_string(),request);
        }
        Some(AllotmentRequest::upcast(self.requests.get(name).unwrap().clone()))
    }

    pub(super) fn union(&mut self, other: &LinearRequestGroup) {
        for (name,request) in other.requests.iter() {
            if !self.requests.contains_key(name) {
                self.requests.insert(name.to_string(),request.clone());
            }
        }
    }

    pub(super) fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,request) in self.requests.iter() {
            if let Some(this_metadata) = allotment_metadata.get(&request.name()) {
                let mut full_metadata = AllotmentMetadataRequest::rebuild(&this_metadata);
                if let Some(allotment) = request.linear_allotment() {
                    allotment.add_metadata(&mut full_metadata);
                }
                out.push(AllotmentMetadata::new(full_metadata));
            }
        }
    }

    pub(super) fn apply_pitch(&self, pitch: &mut Pitch) {
        for (_,request) in &self.requests {
            if let Some(allotment) = request.allotment.lock().unwrap().as_ref() {
                pitch.set_limit(allotment.max());
            }
        }
    }

    pub(super) fn allot(&mut self) {
        let mut sorted_requests = self.requests.values().collect::<Vec<_>>();
        sorted_requests.sort_by_cached_key(|r| r.priority());
        let mut offset = 0;
        for request in sorted_requests {
            request.make(offset,64);
            offset += 64; // XXX
        }
    }
}
