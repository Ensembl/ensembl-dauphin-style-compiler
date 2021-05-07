use std::cmp::Ordering;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::hash::{ Hash, Hasher };
use keyed::{ keyed_handle, KeyedValues };

#[derive(Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
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

impl PartialEq for Allotment {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.data,&other.data) {
            return true;
        }
        self.data.lock().unwrap().eq(&other.data.lock().unwrap())
    }
}
impl Eq for Allotment {}

impl Hash for Allotment {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.data.lock().unwrap().hash(state);
    }
}

impl PartialOrd for Allotment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if Arc::ptr_eq(&self.data,&other.data) {
            return Some(Ordering::Equal);
        }
        self.data.lock().unwrap().partial_cmp(&other.data.lock().unwrap())
    }
}

impl Ord for Allotment {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
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

#[derive(Clone)]
pub struct StickAllotments {
    allotments: Arc<Vec<Allotment>>,
    names: Arc<HashMap<String,usize>>
}

impl StickAllotments {
    pub(crate) fn new(allotments: &[Allotment]) -> StickAllotments {
        let mut allotments = allotments.to_vec();
        allotments.sort_by_cached_key(|a| {
            (a.priority(),a.name())
        });
        let mut names = HashMap::new();
        for (i,allotment) in allotments.iter().enumerate() {
            names.insert(allotment.name(),i);
        }
        StickAllotments {
            allotments: Arc::new(allotments),
            names: Arc::new(names)
        }
    }

    pub fn allotments(&self) -> &[Allotment] { &self.allotments }
}

#[derive(Clone)]
pub struct AllotmentList {
    allotments: Arc<Vec<Allotment>>
}

impl AllotmentList {
    pub fn new(mut allotments: Vec<Allotment>) -> AllotmentList {
        allotments.sort_by_cached_key(|a| {
            (a.priority(),a.name())
        });
        AllotmentList {
            allotments: Arc::new(allotments)
        }
    }

    pub fn allotments(&self) -> &[Allotment] { &self.allotments }
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