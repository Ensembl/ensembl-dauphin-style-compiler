use std::collections::BTreeSet;
use crate::core::{ VareaId, VareaStore, VareaWalker, VareaSearch };
use std::cell::RefCell;

#[cfg_attr(test, derive(Debug))]
pub struct NeverVareaWalker();

impl VareaWalker for NeverVareaWalker {
    fn next_from(&self, _start: VareaId) -> Option<VareaId> {
        None
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct AndVareaSearch(RefCell<Vec<(VareaSearch,u128)>>);

impl AndVareaSearch {
    pub fn new(mut walkers: Vec<VareaSearch>) -> VareaSearch {
        assert!(walkers.len()>0);
        Box::new(AndVareaSearch(RefCell::new(walkers.drain(..).map(|k| (k,0)).collect())))
    }
}

impl VareaWalker for AndVareaSearch {
    fn next_from(&self, min_index: VareaId) -> Option<VareaId> {
        let mut candidate = min_index;
        loop {
            let mut hit = true;
            let mut data = self.0.borrow_mut();
            for (w,d) in data.iter_mut() {
                if let Some(c) = w.next_from(candidate) {
                    *d += (c-candidate) as u128*(c-candidate) as u128;
                    if c > candidate {
                        hit = false;
                        candidate = c;
                        break; // failed this index; re-loop with new candidate
                    } 
                    // else { passed this index; carry on in "for" checking this candidate in next index }
                } else {
                    return None; // no more candidates so we are done
                }
            }
            if hit {
                // only possible if for loop completed without incident ie passed all indexes
                if data.len() > 1 {
                    if data[0].1 * 3 < data[1].1 * 2 {
                        data.swap(0,1);
                    }
                }
                return Some(candidate);
            }
        }
    }    
}

#[cfg_attr(test, derive(Debug))]
pub struct OrVareaSearch(RefCell<Vec<(VareaSearch,u128)>>);

impl OrVareaSearch {
    pub fn new(mut walkers: Vec<VareaSearch>) -> VareaSearch {
        Box::new(OrVareaSearch(RefCell::new(walkers.drain(..).map(|k| (k,0)).collect())))
    }
}

impl VareaWalker for OrVareaSearch {
    fn next_from(&self, min_index: VareaId) -> Option<VareaId> {
        let mut candidate = None;
        let mut data = self.0.borrow_mut();
        for (w,d) in data.iter_mut() {
            if let Some(c) = w.next_from(min_index) {
                if c <= candidate.unwrap_or(c) {
                    *d += (c-min_index) as u128*(c-min_index) as u128;
                    candidate = Some(c);
                }
            }
        }
        if data.len() > 1 {
            if data[0].1 * 2 > data[1].1 * 3 {
                data.swap(0,1);
            }
        }
        candidate
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct AndNotVareaSearch(VareaSearch,VareaSearch);

impl AndNotVareaSearch {
    pub fn new(and: VareaSearch, not: VareaSearch) -> VareaSearch {
        Box::new(AndNotVareaSearch(and,not))
    }
}

impl AndNotVareaSearch {
    fn acceptable(&self, candidate: VareaId) -> bool {
        if let Some(next_not) = self.1.next_from(candidate) {
            return next_not != candidate;
        } else {
            return true;
        }
    }    
}

impl VareaWalker for AndNotVareaSearch {
    fn next_from(&self, min_index: VareaId) -> Option<VareaId> {
        let mut candidate = min_index;
        loop {
            if let Some(next_candidate) = self.0.next_from(candidate) {
                if self.acceptable(next_candidate) {
                    return Some(next_candidate);
                } else {
                    candidate = next_candidate+1;
                }
            } else {
                return None;
            }
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct AllVareaSearch(BTreeSet<VareaId>);

impl AllVareaSearch {
    pub fn new<T>(store: &VareaStore<T>) -> VareaSearch {
        Box::new(AllVareaSearch(store.all_ids().iter().cloned().collect()))
    }
}

impl VareaWalker for AllVareaSearch {
    fn next_from(&self, min_index: VareaId) -> Option<VareaId> {
        self.0.range(min_index..).next().cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::{ VareaItem, VareaStore };
    use crate::discrete::Discrete;

    fn all(vs: &VareaSearch) -> Vec<VareaId> {
        let mut out = vec![];
        let mut start = 0;
        while let Some(value) = vs.next_from(start) {
            out.push(value);
            start = value+1;
        }
        out
    }

    fn item(k:u32) -> VareaItem {
        let mut t = VareaItem::new();
        t.add("x",Discrete::new(&[k/10]));
        t.add("y",Discrete::new(&[k%2]));
        t
    }

    fn search_x(k:u32) -> VareaItem {
        let mut t = VareaItem::new();
        t.add("x",Discrete::new(&[k]));
        t
    }

    fn search_y(k:u32) -> VareaItem {
        let mut t = VareaItem::new();
        t.add("y",Discrete::new(&[k]));
        t
    }

    fn add(s: &mut VareaStore<u32>,v:u32) {
        s.add(item(v),v);
    }

    fn setup() -> VareaStore<u32> {
        let mut s = VareaStore::new();
        add(&mut s,0);
        add(&mut s,1);
        add(&mut s,10);
        add(&mut s,11);
        s
    }

    fn big_setup() -> VareaStore<u32> {
        let mut s = VareaStore::new();
        for i in 0..1000 {
            add(&mut s,i);
        }
        s
    }


    #[test]
    fn search_all() {
        let store = setup();
        let values = AllVareaSearch::new(&store);
        let values = store.lookup(values);
        assert_eq!(vec![0,1,10,11],values.cloned().collect::<Vec<_>>());
    }

    #[test]
    fn search_simple() {
        let store = setup();
        let values = store.search_item(&search_x(0));
        let values = store.lookup(values);
        assert_eq!(vec![0,1],values.cloned().collect::<Vec<_>>());
        let values = store.search_item(&search_x(1));
        let values = store.lookup(values);
        assert_eq!(vec![10,11],values.cloned().collect::<Vec<_>>());
        let values = store.search_item(&search_x(2));
        let values = store.lookup(values);
        assert_eq!(Vec::<u32>::new(),values.cloned().collect::<Vec<_>>());
    }

    #[test]
    fn search_and() {
        let store = setup();
        let x = store.search_item(&search_x(0));
        let y = store.search_item(&search_y(1));
        let values = store.lookup(AndVareaSearch::new(vec![x,y]));
        assert_eq!(vec![1],values.cloned().collect::<Vec<_>>());
        let x = store.search_item(&search_x(1));
        let y = store.search_item(&search_y(0));
        let values = store.lookup(AndVareaSearch::new(vec![x,y]));
        assert_eq!(vec![10],values.cloned().collect::<Vec<_>>());
        let x = store.search_item(&search_x(2));
        let y = store.search_item(&search_y(0));
        let values = store.lookup(AndVareaSearch::new(vec![x,y]));
        assert_eq!(Vec::<u32>::new(),values.cloned().collect::<Vec<_>>());
    }

    #[test]
    fn search_or() {
        let store = setup();
        let x0 = store.search_item(&search_x(0));
        let x2 = store.search_item(&search_x(2));
        let x = OrVareaSearch::new(vec![x0,x2]);
        let y0 = store.search_item(&search_y(0));
        let y1 = store.search_item(&search_y(1));
        let y = OrVareaSearch::new(vec![y0,y1]);
        let values = store.lookup(AndVareaSearch::new(vec![x,y]));
        assert_eq!(vec![0,1],values.cloned().collect::<Vec<_>>());
    }

    #[test]
    fn search_and_not() {
        let store = setup();
        let x0 = store.search_item(&search_x(0));
        let all = AllVareaSearch::new(&store);
        let z = AndNotVareaSearch::new(all,x0);
        let values = store.lookup(z);
        assert_eq!(vec![10,11],values.cloned().collect::<Vec<_>>());
    }

    #[test]
    fn and_promotion() {
        let store = big_setup();
        // x has a hundred entries with ten members; y has two entries with 500 members. And searches should priorisise x.
        let x = store.search_item(&search_x(30));
        let y = store.search_item(&search_y(1));
        let avs = AndVareaSearch::new(vec![y,x]);
        // we can't lookup because we'd lose avs
        let all = all(&avs);
        assert_eq!(vec![302,304,306,308,310],all); // we have only ids so +1, ie id 302 = value 301
        let debug = format!("{:?}",avs);
        assert!(debug.find("301, 302") < debug.find("2, 4"));
    }

    #[test]
    fn or_promotion() {
        let store = big_setup();
        // x has a hundred entries with ten members; y has two entries with 500 members. Or searches should priorisise y.
        let x = store.search_item(&search_x(30));
        let y = store.search_item(&search_y(1));
        let avs = OrVareaSearch::new(vec![y,x]);
        // we can't lookup because we'd lose avs
        let all = all(&avs);
        assert_eq!(505,all.len()); // we have only ids so +1, ie id 302 = value 301
        let debug = format!("{:?}",avs);
        assert!(debug.find("301, 302") > debug.find("2, 4"));
    }
}
