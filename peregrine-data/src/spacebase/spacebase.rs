use std::{marker::PhantomData, ops::{Add, Div}, sync::Arc};
use crate::util::ringarray::{ UniformData, DataFilter };

fn cycle<T>(data: &[T], index: usize) -> &T {
    &data[index%data.len()]
}

fn average<X: Clone + Add<Output=X> + Div<f64,Output=X>>(a: &[X], b: &[X]) -> Vec<X> {
    a.iter().zip(b.iter().cycle()).map(|(a,b)| (a.clone()+b.clone())/2.).collect()
}

pub struct SpaceBasePoint<X> {
    base: X,
    normal: X,
    tangent: X
}

#[derive(Debug)]
pub struct SpaceBasePointRef<'a,X> {
    pub base: &'a X,
    pub normal: &'a X,
    pub tangent: &'a X
}

impl<'a,X: Clone> SpaceBasePointRef<'a,X> {
    fn make(&self) -> SpaceBasePoint<X> {
        SpaceBasePoint {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone()
        }
    }
}

/* If any are empty, all are empty */

#[derive(Debug)]
pub struct SpaceBase<X> {
    pub(super) base: Arc<Vec<X>>,
    pub(super) normal: Arc<Vec<X>>,
    pub(super) tangent: Arc<Vec<X>>,
    pub(super) max_len: usize
}

pub trait ParametricType {
    type Location;
    type Value;

    fn replace(&mut self, replace: &[(Self::Location,Self::Value)]);
}

pub enum SpaceBaseParameterLocation {
    Base(usize),
    Normal(usize),
    Tangent(usize)
}

impl SpaceBaseParameterLocation {
}

impl<X: Clone> ParametricType for SpaceBase<X> {
    type Location = SpaceBaseParameterLocation;
    type Value = X;

    fn replace(&mut self, replace: &[(Self::Location,X)]) {
        let mut base = None;
        let mut normal = None;
        let mut tangent = None;  
        for (location,_) in replace {
            match location {
                SpaceBaseParameterLocation::Base(_) => { base = Some(Arc::make_mut(&mut self.base)); },
                SpaceBaseParameterLocation::Normal(_) => { normal = Some(Arc::make_mut(&mut self.normal)); },
                SpaceBaseParameterLocation::Tangent(_) => { tangent = Some(Arc::make_mut(&mut self.tangent));  },
            }
        }
        for (location,value) in replace {
            match location {
                SpaceBaseParameterLocation::Base(index) => { base.as_mut().unwrap()[*index] = value.clone(); },
                SpaceBaseParameterLocation::Normal(index) => { normal.as_mut().unwrap()[*index] = value.clone(); },
                SpaceBaseParameterLocation::Tangent(index) => { tangent.as_mut().unwrap()[*index] = value.clone(); },
            }
        }
    }
}

impl<X> Clone for SpaceBase<X> {
    fn clone(&self) -> Self {
        SpaceBase {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            max_len: self.max_len
        }
    }
}

impl<X: Clone> SpaceBase<X> {
    pub fn empty() -> SpaceBase<X> {
        SpaceBase {
            base: Arc::new(vec![]),
            normal: Arc::new(vec![]),
            tangent: Arc::new(vec![]),
            max_len: 0
        }
    }

    pub fn len(&self) -> usize { self.max_len }

    pub fn new(base: Vec<X>, normal: Vec<X>, tangent: Vec<X>) -> SpaceBase<X> {
        let max_len = base.len().max(normal.len()).max(tangent.len());
        if base.len() == 0 || normal.len() == 0 || tangent.len() == 0 {
            SpaceBase::empty()
        } else {
            SpaceBase {
                base: Arc::new(base),
                normal: Arc::new(normal),
                tangent: Arc::new(tangent),
                max_len
            }
        }
    }

    pub fn iter_len<'a>(&'a self, length: usize) -> SpaceBaseIterator<'a,X> {
        SpaceBaseIterator {
            spacebase: self,
            index: 0,
            length
        }
    }

    pub fn iter<'a>(&'a self) -> SpaceBaseIterator<'a,X> {
        SpaceBaseIterator {
            spacebase: self,
            index: 0,
            length: self.max_len
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBase<X> {
        SpaceBase {
            base: Arc::new(filter.filter(&self.base)),
            normal: Arc::new(filter.filter(&self.normal)),
            tangent: Arc::new(filter.filter(&self.tangent)),
            max_len: filter.count()
        }
    }

    pub fn replace_normal(&self, other: &SpaceBase<X>) -> SpaceBase<X> {
        SpaceBase {
            base: self.base.clone(),
            tangent: self.tangent.clone(),
            normal: other.normal.clone(),
            max_len: self.max_len
        }
    }
}

impl<X: Clone + PartialOrd> SpaceBase<X> {
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        let mut filter = DataFilter::new(&mut self.base.iter(),|base| {
            let exclude =  *base >= max_value || *base < min_value;
            !exclude
        });
        filter.set_size(self.max_len);
        filter
    }
}

impl<X: Clone + Add<Output=X>> SpaceBase<X> {
    pub fn delta(&self, x_size: &[X], y_size: &[X]) -> SpaceBase<X> {
        if x_size.len() == 0 || y_size.len() == 0 {
            return SpaceBase::empty()
        }
        let mut x_iter = x_size.iter().cycle();
        let mut y_iter = y_size.iter().cycle();
        SpaceBase {
            base: self.base.clone(),
            tangent: Arc::new(self.tangent.iter().map(|x| x.clone()+x_iter.next().unwrap().clone()).collect()),
            normal: Arc::new(self.normal.iter().map(|y| y.clone()+y_iter.next().unwrap().clone()).collect()),
            max_len: self.max_len
        }
    }

    pub fn nudge_normal(&self, amt: X) -> SpaceBase<X> {
        SpaceBase {
            base: self.base.clone(),
            tangent: self.tangent.clone(),
            normal: Arc::new(self.normal.iter().map(|x| x.clone()+amt.clone()).collect()),
            max_len: self.max_len
        }        
    }

    pub fn nudge_tangent(&self, amt: X) -> SpaceBase<X> {
        SpaceBase {
            base: self.base.clone(),
            tangent: Arc::new(self.tangent.iter().map(|x| x.clone()+amt.clone()).collect()),
            normal: self.normal.clone(),
            max_len: self.max_len
        }        
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>> SpaceBase<X> {
    pub fn middle_base(&self, other: &SpaceBase<X>) -> SpaceBase<X> {
        if self.max_len < other.max_len { return other.middle_base(self); }
        SpaceBase {
            base: Arc::new(average(&self.base,&other.base)),
            tangent: self.tangent.clone(),
            normal: self.normal.clone(),
            max_len: self.max_len
        }
    }
}

pub struct SpaceBaseIterator<'a,X> {
    spacebase: &'a SpaceBase<X>,
    index: usize,
    length: usize
}

impl<'a,X> Iterator for SpaceBaseIterator<'a,X> {
    type Item = SpaceBasePointRef<'a,X>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.length { return None; }
        let out = SpaceBasePointRef {
            base: cycle(&self.spacebase.base,self.index),
            normal: cycle(&self.spacebase.normal,self.index),
            tangent: cycle(&self.spacebase.tangent,self.index),
        };
        self.index += 1;
        Some(out)
    }
}
