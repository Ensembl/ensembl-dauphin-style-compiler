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

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetSize(pub i64,pub(super) i64);

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
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

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Allotment {
    position: AllotmentPosition
}

impl Allotment {
    pub(super) fn new(position: AllotmentPosition) -> Allotment {
        Allotment {
            position
        }
    }

    pub fn position(&self) -> &AllotmentPosition { &self.position }
}
