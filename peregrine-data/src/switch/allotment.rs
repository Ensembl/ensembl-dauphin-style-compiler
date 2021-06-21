use std::collections::HashMap;
use std::{sync::{ Arc, Mutex }};
use std::hash::{ Hash };
use keyed::{ keyed_handle, KeyedValues, KeyedData, KeyedHandle };
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

    pub fn kind(&self) -> AllotmentPositionKind { 
        if self.name.starts_with("window:") {
            AllotmentPositionKind::Overlay(if self.name.ends_with("-over") { 1 } else { 0 })
        } else {
            AllotmentPositionKind::Track
        }
    }
}


keyed_handle!(AllotmentHandle);

impl AllotmentHandle {
    pub fn is_null(&self) -> bool { self.get() == 0 }
}

#[derive(Clone)]
pub struct AllotmentPetitioner {
    allotments: Arc<Mutex<KeyedValues<AllotmentHandle,AllotmentRequest>>>
}

impl AllotmentPetitioner {
    pub fn new() -> AllotmentPetitioner {
        let mut out = AllotmentPetitioner {
            allotments: Arc::new(Mutex::new(KeyedValues::new()))
        };
        out.add(AllotmentRequest::new("",0)); // null gets slot 0
        out
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

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum PositionVariant {
    HighPriority,
    LowPriority
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentPositionKind {
    Track,
    Overlay(i64),
    BaseLabel(PositionVariant),
    SpaceLabel(PositionVariant)
}

impl AllotmentPositionKind {
    fn make_allocator(&self) -> Box<dyn AllotmentPositionAllocator> {
        match self {
            AllotmentPositionKind::Track => Box::new(LinearAllotmentPositionAllocator::new(64, |index,size| {
                AllotmentPosition::Track(OffsetSize(index*size,size))
            })), // XXX size
            AllotmentPositionKind::BaseLabel(priority) => {
                let priority = priority.clone();
                Box::new(LinearAllotmentPositionAllocator::new(64, move |index,size| {
                    AllotmentPosition::BaseLabel(priority.clone(),OffsetSize(index*size,size))
                }))
            }, // XXX size
            AllotmentPositionKind::SpaceLabel(priority) => {
                let priority = priority.clone();
                Box::new(LinearAllotmentPositionAllocator::new(64, move |index,size| {
                    AllotmentPosition::SpaceLabel(priority.clone(),OffsetSize(index*size,size))
                }))
            }, // XXX size
            AllotmentPositionKind::Overlay(p) => {
                Box::new(OverlayAllotmentPositionAllocator::new(*p)) as Box<dyn AllotmentPositionAllocator>
            }
        }
    }
}

#[derive(Clone,Debug)]
pub struct OffsetSize(pub i64,i64);

#[derive(Clone,Debug)]
pub enum AllotmentPosition {
    Track(OffsetSize),
    Overlay(i64),
    BaseLabel(PositionVariant,OffsetSize),
    SpaceLabel(PositionVariant,OffsetSize)
}

impl AllotmentPosition {
    pub fn kind(&self) -> AllotmentPositionKind {
        match self {
            AllotmentPosition::Track(_) => AllotmentPositionKind::Track,
            AllotmentPosition::Overlay(p) => AllotmentPositionKind::Overlay(*p),
            AllotmentPosition::BaseLabel(p,_) => AllotmentPositionKind::BaseLabel(p.clone()),
            AllotmentPosition::SpaceLabel(p,_) => AllotmentPositionKind::SpaceLabel(p.clone()),
        }
    }

    pub fn offset(&self) -> i64 { // XXX shouldn't exist. SHould magic shapes instead
        match self {
            AllotmentPosition::Track(x) => x.0,
            AllotmentPosition::BaseLabel(_,x) => x.0,
            AllotmentPosition::SpaceLabel(_,x) => x.0,
            AllotmentPosition::Overlay(x) => *x,
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

struct OverlayAllotmentPositionAllocator {
    priority: i64
}

impl OverlayAllotmentPositionAllocator {
    fn new(priority: i64) -> OverlayAllotmentPositionAllocator {
        OverlayAllotmentPositionAllocator {
            priority
        }
    }
}

impl AllotmentPositionAllocator for OverlayAllotmentPositionAllocator {
    fn allocate(&mut self) -> AllotmentPosition {
        let out = AllotmentPosition::Overlay(self.priority);
        out
    }
}

#[derive(Clone,Debug)]
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
        if request.name() == "" { return; }
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
