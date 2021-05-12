use std::sync::{ Arc, Mutex };
use std::hash::{ Hash };
use keyed::{ keyed_handle, KeyedValues, KeyedData };
use crate::util::DataMessage;

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct AllotmentRequest {
    name: String,
    priority: i64
}

impl AllotmentRequest {
    pub fn new(name: &str, priority: i64) -> AllotmentRequest {
        AllotmentRequest {
            name: name.to_string(),
            priority
        }
    }

    pub fn merge(&mut self, other: &AllotmentRequest) {
        if self.name != other.name { return; }
        if other.priority < self.priority {
            self.priority = other.priority;
        }
    }

    pub fn priority(&self) -> i64 { self.priority }
    pub fn name(&self) -> &str { &self.name }
}


keyed_handle!(AllotmentHandle);

#[derive(Clone)]
pub struct AllotmentPetitioner {
    allotments: Arc<Mutex<KeyedValues<AllotmentHandle,AllotmentRequest>>>
}

impl AllotmentPetitioner {
    pub fn new() -> AllotmentPetitioner {
        AllotmentPetitioner {
            allotments: Arc::new(Mutex::new(KeyedValues::new()))
        }
    }

    pub fn add(&mut self, request: AllotmentRequest) -> AllotmentHandle {
        if let Some(handle) = self.lookup(request.name()) {
            let mut data = self.allotments.lock().unwrap();
            let existing = data.data_mut().get_mut(&handle);
            existing.merge(&request);
            return handle;
        }
        self.allotments.lock().unwrap().add(request.name(),request.clone())
    }

    pub fn lookup(&mut self, name: &str) -> Option<AllotmentHandle> {
        self.allotments.lock().unwrap().get_handle(name).ok()
    }

    pub fn get(&self, handle: &AllotmentHandle) -> AllotmentRequest { self.allotments.lock().unwrap().data().get(handle).clone() }
}

pub struct Allotment {
    offset: i64,
    size: i64
}

impl Allotment {
    fn new(offset: i64, size: i64) -> Allotment {
        Allotment {
            offset, size
        }
    }
}

struct RequestSorter {
    requests: Vec<(AllotmentHandle,AllotmentRequest)>
}

impl RequestSorter {
    fn new() -> RequestSorter {
        RequestSorter {
            requests: vec![]
        }
    }

    fn add(&mut self, petitioner: &AllotmentPetitioner, handle: &AllotmentHandle) {
        let request = petitioner.get(handle);
        self.requests.push((handle.clone(),request));
    }

    fn get(mut self) -> Vec<AllotmentHandle> {
        self.requests.sort_by_cached_key(|(_,r)| {
            (r.priority(),r.name().to_string())
        });
        self.requests.iter().map(|(h,_)| h.clone()).collect()
    }
}

struct RunningAllotter {
    index: i64
}

impl RunningAllotter {
    fn new() -> RunningAllotter {
        RunningAllotter {
            index: 0
        }
    }

    fn add(&mut self, petitioner: &AllotmentPetitioner, handle: &AllotmentHandle) -> Allotment {
        let out = Allotment::new(self.index*64,64); // XXX wrong
        self.index +=1;
        out
    }
}

pub struct Allotter {
    allotments: KeyedData<AllotmentHandle,Option<Allotment>>
}

impl Allotter {
    pub fn empty() -> Allotter {
        Allotter {
            allotments: KeyedData::new()
        }
    }

    pub fn new(petitioner: &AllotmentPetitioner, handles: &[AllotmentHandle]) -> Allotter {
        let mut sorter = RequestSorter::new();
        for handle in handles {
            sorter.add(petitioner,handle);
        }
        let mut allotments = KeyedData::new();
        let mut running_allocator = RunningAllotter::new();
        for sorted_handle in sorter.get() {
            allotments.insert(&sorted_handle,running_allocator.add(petitioner,&sorted_handle));
        }
        Allotter {
            allotments
        }
    }

    pub fn get(&self, handle: &AllotmentHandle) -> Result<&Allotment,DataMessage> {
        self.allotments.get(handle).as_ref().ok_or_else(|| DataMessage::NoSuchAllotment("request for unallocated allotment".to_string()))
    }
}
