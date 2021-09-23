use std::{collections::HashMap};
use crate::{Allotment, AllotmentPosition, AllotmentGroup, AllotmentMetadata, DataMessage};
use super::{allotment::{AllotterMetadata, AllotmentImpl, OffsetSize}, pitch::Pitch};

struct RequestSorter {
    requests: Vec<AllotmentMetadata>
}

impl RequestSorter {
    fn new() -> RequestSorter {
        RequestSorter {
            requests: vec![]
        }
    }

    fn add(&mut self, request: &AllotmentMetadata) {
        if request.is_dustbin() { return; }
        self.requests.push(request.clone());
    }

    fn get(mut self) -> Vec<AllotmentMetadata> {
        self.requests.sort_by_cached_key(|r| {
            (r.priority(),r.name().to_string())
        });
        self.requests.iter().cloned().collect()
    }
}

fn make_allocator(kind: &AllotmentGroup) -> Box<dyn AllotmentPositionAllocator> {
    match kind {
        AllotmentGroup::Track => Box::new(LinearAllotmentPositionAllocator::new(64, |index,size| {
            AllotmentPosition::Track(OffsetSize(index*size,size))
        })), // XXX size
        AllotmentGroup::BaseLabel(priority) => {
            let priority = priority.clone();
            Box::new(LinearAllotmentPositionAllocator::new(64, move |index,size| {
                AllotmentPosition::BaseLabel(priority.clone(),OffsetSize(index*size,size))
            }))
        }, // XXX size
        AllotmentGroup::SpaceLabel(priority) => {
            let priority = priority.clone();
            Box::new(LinearAllotmentPositionAllocator::new(64, move |index,size| {
                AllotmentPosition::SpaceLabel(priority.clone(),OffsetSize(index*size,size))
            }))
        }, // XXX size
        AllotmentGroup::Overlay(p) => {
            Box::new(OverlayAllotmentPositionAllocator::new(*p)) as Box<dyn AllotmentPositionAllocator>
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

struct RunningAllotter {
    allocators: HashMap<AllotmentGroup,Box<dyn AllotmentPositionAllocator>>
}

impl RunningAllotter {
    fn new() -> RunningAllotter {
        RunningAllotter {
            allocators: HashMap::new()
        }
    }

    fn get_allocator(&mut self, kind: &AllotmentGroup) -> &mut Box<dyn AllotmentPositionAllocator> {
        self.allocators.entry(kind.clone()).or_insert_with(|| {
            make_allocator(kind)
        })
    }

    fn add(&mut self, request: &AllotmentMetadata) -> Allotment {
        let position = self.get_allocator(&request.allotment_group()).allocate();
        let metadata = request.update_metadata(&position);
        Allotment::new(position,&metadata)
    }
}

pub struct Allotter {
    allotments: HashMap<AllotmentMetadata,Allotment>,
    metadata: AllotterMetadata,
    pitch: Pitch
}

impl Allotter {
    pub fn empty() -> Allotter {
        Allotter {
            allotments: HashMap::new(),
            metadata: AllotterMetadata::new(vec![]),
            pitch: Pitch::new()
        }
    }

    pub fn new(requests: &[AllotmentMetadata]) -> Allotter {
        let mut pitch = Pitch::new();
        let mut sorter = RequestSorter::new();
        for request in requests {
            sorter.add(request);
        }
        let mut metadata = vec![];
        let mut allotments = HashMap::new();
        let mut running_allocator = RunningAllotter::new();
        for sorted_request in sorter.get() {
            let allotment = running_allocator.add(&sorted_request);
            allotment.apply_pitch(&mut pitch);
            metadata.push(allotment.metadata().clone());
            allotments.insert(sorted_request,allotment);
        }
        Allotter { allotments, metadata: AllotterMetadata::new(metadata), pitch }
    }

    pub fn get(&self, handle: &AllotmentMetadata) -> Result<Allotment,DataMessage> {
        self.allotments.get(handle).ok_or_else(|| DataMessage::NoSuchAllotment("request for unallocated allotment".to_string())).map(|r| r.clone())
    }

    pub fn metadata(&self) -> AllotterMetadata { self.metadata.clone() }
    pub fn pitch(&self) -> &Pitch { &self.pitch }
}
