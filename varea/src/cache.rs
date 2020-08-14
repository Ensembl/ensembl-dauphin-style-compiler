use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::lrucache::Cache;
use super::core::{ VareaStore, VareaId, VareaItemRemover, VareaItem, VareaStoreMatches, VareaSearch };

pub struct VareaCache<T> {
    to_drop: Rc<RefCell<Vec<VareaId>>>,
    droppers: HashMap<VareaId,VareaItemRemover>,
    store: VareaStore<T>,
    cache: Rc<RefCell<Cache<VareaId,VareaId>>>
}

impl<T> VareaCache<T> {
    pub fn new(size: usize) -> VareaCache<T> {
        let mut cache = Cache::<VareaId,VareaId>::new(size);
        let to_drop = Rc::new(RefCell::new(vec![]));
        let to_drop_copy = to_drop.clone();
        cache.set_dropper(move |_,id| to_drop_copy.borrow_mut().push(id));
        VareaCache {
            to_drop,
            store: VareaStore::new(),
            droppers: HashMap::new(),
            cache: Rc::new(RefCell::new(cache))
        }
    }

    fn remove(&mut self, id: &VareaId) {
        if let Some(mut remover) = self.droppers.remove(id) {
            remover.remove(&mut self.store);
        }
    }

    pub fn add(&mut self, item: VareaItem, value: T) {
        let (id,remover) = self.store.add(item,value);
        self.droppers.insert(id,remover);
        self.cache.borrow_mut().put(&id,id);
        let ids = self.to_drop.borrow_mut().to_vec();
        for id in &ids {
            self.remove(&id);
        }
        *self.to_drop.borrow_mut() = vec![];
    }

    pub fn lookup<'a,'b>(&'a self, search: VareaSearch) -> VareaCacheMatches<'a,T> {
        VareaCacheMatches(self.store.lookup(search),self.cache.clone())
    }

    pub fn search_item(&self, item: &VareaItem) -> VareaSearch {
        self.store.search_item(item)
    }
}

pub struct VareaCacheMatches<'a,T>(VareaStoreMatches<'a,T>,Rc<RefCell<Cache<VareaId,VareaId>>>);

impl<'a,T> VareaCacheMatches<'a,T> {
    pub fn hit(&mut self) {
        self.1.borrow_mut().get(&self.0.get_id());
    }
}

impl<'a,T> Iterator for VareaCacheMatches<'a,T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.0.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::discrete::Discrete;
    use crate::walkers::OrVareaSearch;

    fn item(k:u32) -> VareaItem {
        let mut t = VareaItem::new();
        t.add("x",Discrete::new(k/10));
        t.add("y",Discrete::new(k%2));
        t
    }

    fn add(s: &mut VareaCache<u32>,v:u32) {
        s.add(item(v),v);
    }

    fn search_x(k:u32) -> VareaItem {
        let mut t = VareaItem::new();
        t.add("x",Discrete::new(k));
        t
    }

    #[test]
    fn cache() {
        let mut c = VareaCache::<u32>::new(16);
        add(&mut c,0);
        add(&mut c,1);
        add(&mut c,10);
        add(&mut c,11);
        let x0 = c.search_item(&search_x(0));
        let x1 = c.search_item(&search_x(1));
        let avs = OrVareaSearch::new(vec![x0,x1]);
        let values = c.lookup(avs);
        assert_eq!(vec![0,1,10,11],values.cloned().collect::<Vec<_>>());
        /* flood with other values, to force out 0, 1, 10, 11 at some stage */
        for i in 0..10000 {
            add(&mut c,i+100);
        }
        let x0 = c.search_item(&search_x(0));
        let x1 = c.search_item(&search_x(1));
        let avs = OrVareaSearch::new(vec![x0,x1]);
        let values = c.lookup(avs);
        assert_eq!(Vec::<u32>::new(),values.cloned().collect::<Vec<_>>());
        assert_eq!(0,c.to_drop.borrow().len());
        assert!(c.droppers.get(&10).is_none());
        assert!(c.droppers.get(&11).is_none());
    }

    #[test]
    fn hit() {
        let mut c = VareaCache::<u32>::new(16);
        add(&mut c,0);
        add(&mut c,1);
        add(&mut c,10);
        add(&mut c,11);
        for i in 0..100 {
            add(&mut c,i+100);
            let x0 = c.search_item(&search_x(0));
            let x1 = c.search_item(&search_x(1));
            let avs = OrVareaSearch::new(vec![x0,x1]);
            let mut values = c.lookup(avs);
            while let Some(_) = values.next() {
                values.hit();
            }
        }
        let x0 = c.search_item(&search_x(0));
        let x1 = c.search_item(&search_x(1));
        let avs = OrVareaSearch::new(vec![x0,x1]);
        let values = c.lookup(avs);
        assert_eq!(vec![0,1,10,11],values.cloned().collect::<Vec<_>>());
    }
}
