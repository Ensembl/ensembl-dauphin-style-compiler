use std::any::Any;
use std::cell::RefCell;
use std::collections::{ HashMap, BTreeSet };
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use super::core::{ VareaId, VareaWalker, VareaSearch };
use super::axis::{ VareaIndexItem, VareaIndex, VareaIndexRemover };
use super::walkers::{ NeverVareaWalker, OrVareaSearch };

#[cfg_attr(test, derive(Debug))]
pub struct Discrete<T> where T: Hash+PartialEq+Clone {
    value: Vec<T>
}

impl<T> Discrete<T> where T: Clone+Hash+PartialEq+Eq+'static {
    pub fn new(value: &[T]) -> Discrete<T> {
        Discrete { value: value.to_vec() }
    }
}

impl<T> VareaIndexItem for Discrete<T> where T: Clone+Hash+PartialEq+Eq+'static+Debug {
    fn factory_id(&self) -> &str { "discrete" }
    fn make_index(&self) -> Box<dyn VareaIndex> {
        Box::new(DiscreteIndex::<T>::new())
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}

pub(crate) struct DiscreteIndex<T> {
    index: HashMap<T,Rc<RefCell<BTreeSet<VareaId>>>>
}

impl<T> DiscreteIndex<T> {
    fn new() -> DiscreteIndex<T> {
        DiscreteIndex {
            index: HashMap::new()
        }
    }
}

impl<T> VareaIndex for DiscreteIndex<T> where T: Clone+Hash+PartialEq+Eq+'static {
    fn add(&mut self, id: &VareaId, item: Box<dyn VareaIndexItem>) -> Box<dyn VareaIndexRemover> {
        if let Ok(d) = item.into_any().downcast::<Discrete<_>>() {
            let mut all_indexes = vec![];
            for value in &d.value {
                if !self.index.contains_key(value) {
                    self.index.insert(value.clone(),Rc::new(RefCell::new(BTreeSet::new())));
                }
                let index = self.index.get_mut(value).unwrap();
                index.borrow_mut().insert(*id);
                all_indexes.push(index.clone());
            }
            Box::new(DiscreteIndexRemover(Some((all_indexes,*id))))
        } else {
            Box::new(DiscreteIndexRemover(None))
        }
    }

    fn lookup(&self, area: &Box<dyn VareaIndexItem>) -> VareaSearch {
        if let Some(d) = area.as_any().downcast_ref::<Discrete<_>>() {
            let mut walkers : Vec<VareaSearch> = vec![];
            for value in &d.value {
                if let Some(values) = self.index.get(value) {
                    walkers.push(Box::new(DiscreteVareaWalker(values.clone())));
                }
            }
            return OrVareaSearch::new(walkers)
        }
        Box::new(NeverVareaWalker())
    }
}

pub(crate) struct DiscreteIndexRemover(Option<(Vec<Rc<RefCell<BTreeSet<VareaId>>>>,VareaId)>);

impl VareaIndexRemover for DiscreteIndexRemover {
    fn remove(&mut self) {
        if let Some((trees,id)) = &self.0 {
            for tree in trees {
                tree.borrow_mut().remove(&id);
            }
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct DiscreteVareaWalker(Rc<RefCell<BTreeSet<VareaId>>>);

impl VareaWalker for DiscreteVareaWalker {
    fn next_from(&self, start: VareaId) -> Option<VareaId> {
        self.0.borrow().range(start..).next().cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn all(vs: VareaSearch) -> Vec<VareaId> {
        let mut out = vec![];
        let mut start = 0;
        while let Some(value) = vs.next_from(start) {
            out.push(value);
            start = value+1;
        }
        out
    }

    #[test]
    fn discrete() {
        let mut d = DiscreteIndex::<u32>::new();
        d.add(&0,Box::new(Discrete::new(&[0_u32])));
        d.add(&1,Box::new(Discrete::new(&[0_u32,10])));
        d.add(&10,Box::new(Discrete::new(&[1_u32,10])));
        let mut r = d.add(&11,Box::new(Discrete::new(&[1_u32,11])));
        let x : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[0_u32]));
        assert_eq!(vec![0,1],all(d.lookup(&x)));
        let x : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[1_u32]));
        assert_eq!(vec![10,11],all(d.lookup(&x)));
        let x : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[10_u32]));
        assert_eq!(vec![1,10],all(d.lookup(&x)));
        let x : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[10_u32,1]));
        assert_eq!(vec![1,10,11],all(d.lookup(&x)));
        let x : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[2_u32]));
        assert_eq!(Vec::<VareaId>::new(),all(d.lookup(&x)));
        r.remove();
        let x : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[1_u32]));
        assert_eq!(vec![10],all(d.lookup(&x)));
    }
}
