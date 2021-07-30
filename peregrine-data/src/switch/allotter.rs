use std::collections::HashMap;
use keyed::KeyedData;
use crate::{Allotment, AllotmentPetitioner, AllotmentPosition, AllotmentPositionKind, AllotmentRequest, DataMessage};
use super::allotment::{ OffsetSize, AllotmentHandle };

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

fn make_allocator(kind: &AllotmentPositionKind) -> Box<dyn AllotmentPositionAllocator> {
    match kind {
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
            make_allocator(kind)
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
