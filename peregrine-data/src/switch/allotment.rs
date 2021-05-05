use std::sync::{ Arc, Mutex };
use std::collections::HashSet;
use keyed::{ keyed_handle, KeyedValues };

#[derive(Debug)]
struct AllotmentData {
    name: String,
    priority: i64
}

impl AllotmentData {
    fn new(name: &str, priority: i64) -> AllotmentData {
        AllotmentData {
            name: name.to_string(),
            priority
        }
    }
}

#[derive(Clone,Debug)]
pub struct Allotment {
    data: Arc<Mutex<AllotmentData>>
}

impl Allotment {
    pub fn new(name: &str, priority: i64) -> Allotment {
        Allotment {
            data: Arc::new(Mutex::new(AllotmentData::new(name,priority)))
        }
    }

    pub fn priority(&self) -> i64 { self.data.lock().unwrap().priority }
    pub fn name(&self) -> String { self.data.lock().unwrap().name.to_string() }
}

/*
keyed_handle!(AllotmentHandle);

struct AllotmentStore {
    allotments: KeyedValues<AllotmentHandle,Allotment>
}

impl AllotmentStore {
    fn new() -> AllotmentStore {
        AllotmentStore {
            allotments: KeyedValues::new()
        }
    }

    fn lookup(&mut self, name: &str) -> AllotmentHandle {
        let handle = self.allotments.get_handle(name).ok();
        if let Some(handle) = handle {
            handle
        } else {
            self.allotments.add(name,Allotment::new(name))
        }
    }

    fn get(&self, handle: &AllotmentHandle) -> &Allotment { self.allotments.data().get(handle) }
}

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