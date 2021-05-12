use std::cmp::Ordering;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::hash::{ Hash, Hasher };
use keyed::{ keyed_handle, KeyedValues };

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

/*
pub(crate) struct ScreenBuilder {
    allotments: HashSet<AllotmentHandle>
}

impl ScreenBuilder {
    pub(crate) fn new() -> ScreenBuilder {
        ScreenBuilder {
            allotments: HashSet::new()
        }
    }

    fn set_active(&mut self, allotment: AllotmentHandle) {
        self.allotments.insert(allotment);
    }

    fn build(&self, allotter: &Allotter) -> Screen {
        Screen::new(self,allotter)
    }
}

fn order_allotments(builder: &ScreenBuilder, allotter: &Allotter) -> Vec<Allotment> {
    let mut allotments = builder.allotments.iter().map(|handle| {
        allotter.get(handle)
    }).collect::<Vec<_>>();
    allotments.sort_by_cached_key(|allotment| {
        (allotment.priority().unwrap_or(0),allotment.name())
    });
    allotments
}

pub(crate) struct Screen {
    allotments: Vec<Allotment>
}

impl Screen {
    fn new(builder: &ScreenBuilder, allotter: &Allotter) -> Screen {
        let allotments = order_allotments(builder,allotter);
        Screen {
            allotments
        }
    }
}

pub(crate) struct Allotter {
    store: Arc<Mutex<AllotmentStore>>
}

impl Allotter {
    pub(crate) fn new() -> Allotter {
        Allotter {
            store: Arc::new(Mutex::new(AllotmentStore::new()))
        }
    }

    pub(crate) fn lookup(&self, name: &str) -> AllotmentHandle {
        self.store.lock().unwrap().lookup(name)
    }

    fn get(&self, handle: &AllotmentHandle) -> Allotment { self.store.lock().unwrap().get(handle).clone() }
}
*/