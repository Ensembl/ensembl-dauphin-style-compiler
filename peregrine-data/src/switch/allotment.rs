use std::collections::HashMap;
use std::{io::LineWriter, sync::{ Arc, Mutex }};
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

    pub fn kind(&self) -> AllotmentPositionKind { AllotmentPositionKind::Paper }
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


#[derive(Clone,PartialEq,Eq,Hash)]
pub enum AllotmentPositionKind {
    Paper,
    Top,
    Bottom
}

impl AllotmentPositionKind {
    fn make_allocator(&self) -> Box<dyn AllotmentPositionAllocator> {
        Box::new(match self {
            AllotmentPositionKind::Paper => LinearAllotmentPositionAllocator::new(64, |index,size| {
                AllotmentPosition::Paper(index*size,size)
            }), // XXX size
            AllotmentPositionKind::Top => LinearAllotmentPositionAllocator::new(64, |index,size| {
                AllotmentPosition::Top(index*size,size)
            }), // XXX size
            AllotmentPositionKind::Bottom => LinearAllotmentPositionAllocator::new(64, |index,size| {
                AllotmentPosition::Bottom(index*size,size)
            }), // XXX size
        })
    }
}

#[derive(Clone)]
pub enum AllotmentPosition {
    Paper(i64,i64),
    Top(i64,i64),
    Bottom(i64,i64)
}

impl AllotmentPosition {
    fn to_kind(&self) -> AllotmentPositionKind {
        match self {
            AllotmentPosition::Paper(_,_) => AllotmentPositionKind::Paper,
            AllotmentPosition::Top(_,_) => AllotmentPositionKind::Top,
            AllotmentPosition::Bottom(_,_) => AllotmentPositionKind::Bottom,
        }
    }

    pub fn offset(&self) -> i64 { // XXX shouldn't exist. SHould magic shapes instead
        *match self {
            AllotmentPosition::Paper(x,_) => x,
            AllotmentPosition::Top(x,_) => x,
            AllotmentPosition::Bottom(x,_) => x,
        }
    }
}


trait AllotmentPositionAllocator {
    fn allocate(&mut self) -> AllotmentPosition;
}

struct LinearAllotmentPositionAllocator {
    index: i64,
    size: i64,
    ctor: Box<dyn Fn(i64,i64) -> AllotmentPosition>
}

impl LinearAllotmentPositionAllocator {
    fn new<F>(size: i64, ctor: F) -> LinearAllotmentPositionAllocator where F: Fn(i64,i64) -> AllotmentPosition + 'static {
        LinearAllotmentPositionAllocator {
            index: 0,
            size,
            ctor: Box::new(ctor)
        }
    }
}

impl AllotmentPositionAllocator for LinearAllotmentPositionAllocator {
    fn allocate(&mut self) -> AllotmentPosition {
        let out = (self.ctor)(self.index,self.size);
        self.index += 1;
        out
    }
}

#[derive(Clone)]
pub struct Allotment {
    position: AllotmentPosition
}

impl Allotment {
    fn new(position: AllotmentPosition) -> Allotment {
        Allotment {
            position
        }
    }

    pub fn position(&self) -> &AllotmentPosition { &self.position }
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
    allocators: HashMap<AllotmentPositionKind,Box<dyn AllotmentPositionAllocator>>
}

impl RunningAllotter {
    fn new() -> RunningAllotter {
        RunningAllotter {
            allocators: HashMap::new()
        }
    }

    fn get_allocator(&mut self, kind: &AllotmentPositionKind) -> &mut Box<dyn AllotmentPositionAllocator> {
        self.allocators.entry(kind.clone()).or_insert_with(|| {
            kind.make_allocator()
        })
    }

    fn add(&mut self, petitioner: &AllotmentPetitioner, handle: &AllotmentHandle) -> Allotment {
        let position = self.get_allocator(&petitioner.get(handle).kind()).allocate();
        Allotment::new(position)
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
