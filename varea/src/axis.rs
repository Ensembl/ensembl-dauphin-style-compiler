use crate::core::{ VareaId, VareaSearch };
use super::walkers::NeverVareaWalker;
use std::any::Any;
use std::collections::HashMap;
#[cfg(test)]
use std::fmt::Debug;

pub(crate) struct VareaAxis {
    indexes: HashMap<String,Box<dyn VareaIndex>>
}

impl VareaAxis {
    pub(crate) fn new() -> VareaAxis {
        VareaAxis {
            indexes: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, id: &VareaId, item: Box<dyn VareaIndexItem>) -> Box<dyn VareaIndexRemover> {
        let factory_id = item.factory_id().to_string();
        if !self.indexes.contains_key(&factory_id) {
            let id = factory_id.to_string();
            self.indexes.insert(id,item.make_index());
        }
        self.indexes.get_mut(&factory_id).as_mut().unwrap().add(id,item)
    }

    pub(crate) fn lookup<'a>(&'a self, region: &Box<dyn VareaIndexItem>) -> VareaSearch {
        let factory_id = region.factory_id();
        if !self.indexes.contains_key(factory_id) {
            return Box::new(NeverVareaWalker());
        }
        self.indexes.get(factory_id).as_ref().unwrap().lookup(region)
    }
}

#[cfg(test)]
pub trait VareaIndexItem : Debug {
    fn factory_id(&self) -> &str;
    fn make_index(&self) -> Box<dyn VareaIndex>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
}

#[cfg(not(test))]
pub trait VareaIndexItem {
    fn factory_id(&self) -> &str;
    fn make_index(&self) -> Box<dyn VareaIndex>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
}

pub trait VareaIndexRemover {
    fn remove(&mut self);
}

pub trait VareaIndex {
    fn add(&mut self, id: &VareaId, item: Box<dyn VareaIndexItem>) -> Box<dyn VareaIndexRemover>;
    fn lookup(&self, area: &Box<dyn VareaIndexItem>) -> VareaSearch;
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::discrete::Discrete;
    use super::super::range::RTreeRange;

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
    fn axis() {
        let mut axis = VareaAxis::new();
        axis.add(&1,Box::new(Discrete::new(&[0])));
        let r : Box<dyn VareaIndexItem> = Box::new(RTreeRange::new(0,100));
        let r = all(axis.lookup(&r));
        assert_eq!(Vec::<VareaId>::new(),r);
        let mut rm = axis.add(&2,Box::new(RTreeRange::new(2,25)));
        axis.add(&3,Box::new(RTreeRange::new(12,14)));
        let r : Box<dyn VareaIndexItem> = Box::new(Discrete::new(&[0]));
        let r = all(axis.lookup(&r));
        assert_eq!(vec![1],r);
        let r : Box<dyn VareaIndexItem> = Box::new(RTreeRange::new(2,8));
        let r = all(axis.lookup(&r));
        assert_eq!(vec![2],r);
        let r : Box<dyn VareaIndexItem> = Box::new(RTreeRange::new(2,13));
        let r = all(axis.lookup(&r));
        assert_eq!(vec![2,3],r);
        rm.remove();
        let r : Box<dyn VareaIndexItem> = Box::new(RTreeRange::new(2,13));
        let r = all(axis.lookup(&r));
        assert_eq!(vec![3],r);
    }
}