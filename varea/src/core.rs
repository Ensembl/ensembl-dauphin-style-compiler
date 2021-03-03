use std::collections::{ BTreeMap, HashMap };
use crate::axis::{ VareaIndexItem, VareaIndexRemover, VareaAxis };
use crate::walkers::{ AllVareaSearch, AndVareaSearch };
#[cfg(test)]
use std::fmt::Debug;

pub type VareaId = usize;

pub struct VareaItem {
    area: HashMap<String,Box<dyn VareaIndexItem>>
}

impl VareaItem {
    pub fn new() -> VareaItem {
        VareaItem {
            area: HashMap::new()
        }
    }

    pub fn add<T>(&mut self, axis: &str, value: T) where T: VareaIndexItem + 'static {
        self.area.insert(axis.to_string(),Box::new(value));
    }
}

pub struct VareaItemRemover(Vec<Box<dyn VareaIndexRemover>>,VareaId);

impl VareaItemRemover {
    pub fn remove<T>(&mut self, store: &mut VareaStore<T>) {
        for r in self.0.iter_mut() {
            r.remove();
        }
        store.remove(&self.1);
    }
}

#[cfg(test)]
pub trait VareaWalker : Debug {
    fn next_from(&self, start: VareaId) -> Option<VareaId>;
}

#[cfg(not(test))]
pub trait VareaWalker {
    fn next_from(&self, start: VareaId) -> Option<VareaId>;
}

pub type VareaSearch = Box<dyn VareaWalker>;

pub struct VareaStore<T> {
    next_id: VareaId,
    payloads: BTreeMap<VareaId,T>,
    axes: HashMap<String,VareaAxis>
}

impl<T> VareaStore<T> {
    pub fn new() -> VareaStore<T> {
        VareaStore {
            next_id: 1,
            payloads: BTreeMap::new(),
            axes: HashMap::new()
        }
    }

    pub fn add(&mut self, mut item: VareaItem, value: T) -> (VareaId,VareaItemRemover) {
        let id = self.next_id;
        self.next_id += 1;
        self.payloads.insert(id,value);
        let mut removers = vec![];
        for (axis,item) in item.area.drain() {
            let remover = self.axes.entry(axis.clone()).or_insert_with(|| {
                VareaAxis::new()
            }).add(&id,item);
            removers.push(remover);
        }
        (id,VareaItemRemover(removers,id))
    }

    pub(crate) fn all_ids(&self) -> Vec<VareaId> {
        self.payloads.keys().cloned().collect()
    }

    pub fn search_item(&self, item: &VareaItem) -> VareaSearch {
        let mut walkers = vec![];
        for (axis,index) in &self.axes {
            if let Some(axis_item) = item.area.get(axis) {
                walkers.push(index.lookup(axis_item));
            }
        }
        if walkers.len() > 0 {
            AndVareaSearch::new(walkers)
        } else {
            AllVareaSearch::new(self)
        }
    }

    pub fn lookup<'a>(&'a self, search: VareaSearch) -> VareaStoreMatches<'a,T> {
        VareaStoreMatches {
            payloads: &self.payloads,
            search,
            index: 0,
            prev_id: None
        }
    }

    fn remove(&mut self, id: &VareaId) {
        // rest of removal handled by VareaItemRemover
        self.payloads.remove(id);
    }
}

pub struct VareaStoreMatches<'a,T> {
    payloads: &'a BTreeMap<VareaId,T>,
    search: VareaSearch,
    index: usize,
    prev_id: Option<VareaId>
}

impl<'a,T> VareaStoreMatches<'a,T> {
    pub fn get_id(&self) -> VareaId {
        self.prev_id.unwrap()
    }
}

impl<'a,T> Iterator for VareaStoreMatches<'a,T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if let Some(id) = self.search.next_from(self.index) {
            self.index = id+1;
            self.prev_id = Some(id);
            Some(self.payloads.get(&id).unwrap())
        } else {
            None
        }
    }
}

// This file is effectively unit-tested by the tests in walker.