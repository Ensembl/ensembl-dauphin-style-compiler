use std::{ops::{Add, Div}, sync::{Arc}};
use crate::{/*SpaceBaseSized,*/ util::ringarray::{ DataFilter }};

use super::parametric::{Flattenable, ParameterValue, ParametricType, Substitutions};

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

#[cfg_attr(debug_assertions,derive(Debug))]
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

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct SpaceBase<X> {
    pub(super) base: Arc<Vec<X>>,
    pub(super) normal: Arc<Vec<X>>,
    pub(super) tangent: Arc<Vec<X>>,
    pub(super) max_len: usize
}

pub enum SpaceBaseParameterLocation {
    Base(usize),
    Normal(usize),
    Tangent(usize)
}

impl<X: Clone> SpaceBase<ParameterValue<X>> {
    pub(crate) fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase<X> where F: Fn(SpaceBaseParameterLocation) -> L {
        SpaceBase {
            base: Arc::new(subs.flatten(&self.base,|x| cb(SpaceBaseParameterLocation::Base(x)))),
            normal: Arc::new(subs.flatten(&self.normal, |x| cb(SpaceBaseParameterLocation::Normal(x)))),
            tangent: Arc::new(subs.flatten(&self.tangent,|x| cb(SpaceBaseParameterLocation::Tangent(x)))),
            max_len: self.max_len
        }
    }
}

impl<X: Clone> ParametricType for SpaceBase<X> {
    type Location = SpaceBaseParameterLocation;
    type Value = X;

    fn replace(&mut self, replace: &[(&Self::Location,X)]) {
        let mut go_base = false;
        let mut go_normal = false;
        let mut go_tangent = false;
        for (location,_) in replace {
            match location {
                SpaceBaseParameterLocation::Base(_) => {go_base = true;; },
                SpaceBaseParameterLocation::Normal(_) => { go_normal = true; },
                SpaceBaseParameterLocation::Tangent(_) => { go_tangent = true;  },
            }
        }
        let mut base = if go_base { Some(Arc::make_mut(&mut self.base)) } else { None };
        let mut normal = if go_normal { Some(Arc::make_mut(&mut self.normal)) } else { None };
        let mut tangent = if go_tangent { Some(Arc::make_mut(&mut self.tangent)) } else { None };
        for (location,value) in replace {
            match location {
                SpaceBaseParameterLocation::Base(index) => { base.as_mut().unwrap()[*index] = value.clone(); },
                SpaceBaseParameterLocation::Normal(index) => { normal.as_mut().unwrap()[*index] = value.clone(); },
                SpaceBaseParameterLocation::Tangent(index) => { tangent.as_mut().unwrap()[*index] = value.clone(); },
            }
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum HoleySpaceBase {
    Simple(SpaceBase<f64>),
    Parametric(SpaceBase<ParameterValue<f64>>)
}

impl HoleySpaceBase {
    pub fn default_values(&self) -> SpaceBase<f64> {
        match self {
            HoleySpaceBase::Simple(x) => x.clone(),
            HoleySpaceBase::Parametric(x) => {
                x.clone().map_into(|x| *x.param_default())
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            HoleySpaceBase::Simple(x) => x.len(),
            HoleySpaceBase::Parametric(x) => x.len()
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> HoleySpaceBase {
        match self {
            HoleySpaceBase::Simple(x) => HoleySpaceBase::Simple(x.filter(filter)),
            HoleySpaceBase::Parametric(x) => HoleySpaceBase::Parametric(x.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min_value: f64, max_value: f64) -> DataFilter {
        match self {
            HoleySpaceBase::Simple(x) =>
                x.make_base_filter(min_value,max_value),
            HoleySpaceBase::Parametric(x) =>
                x.make_base_filter(ParameterValue::Constant(min_value),ParameterValue::Constant(max_value))
        }
    }
}

impl Flattenable for HoleySpaceBase {
    type Location = SpaceBaseParameterLocation;
    type Target = SpaceBase<f64>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase<f64> where F: Fn(Self::Location) -> L {
        match self {
            HoleySpaceBase::Simple(x) => x.clone(),
            HoleySpaceBase::Parametric(x) => x.flatten(subs,cb)
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

    // XXX WRONG! Consider
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

    pub fn map_into<F,Z>(&mut self, cb : F) -> SpaceBase<Z> where F: Fn(&X) -> Z {
        SpaceBase {
            base: Arc::new(self.base.iter().map(&cb).collect()),
            tangent: Arc::new(self.tangent.iter().map(&cb).collect()),
            normal: Arc::new(self.normal.iter().map(&cb).collect()),
            max_len: self.max_len
        }
    }

    pub fn update_tangent<'a,F>(&mut self, mut cb: F) where F: FnMut(&mut X) {
        for x in Arc::make_mut(&mut self.tangent) { cb(x); }
    }

    pub fn update_normal<F>(&mut self, mut cb: F) where F: FnMut(&mut X) {
        for x in Arc::make_mut(&mut self.normal) { cb(x); }
    }

    pub fn fold_tangent<F,Z>(&mut self, values: &[Z], cb: F) where F: Fn(&mut X,&Z) {
        if values.len() == 0 { return; }
        let mut values2 = values.iter().cycle();
        self.update_tangent(move |x| { cb(x,values2.next().unwrap()) });
    }

    pub fn fold_normal<F,Z>(&mut self, values: &[Z], cb: F) where F: Fn(&mut X,&Z) {
        if values.len() == 0 { return; }
        let mut values2 = values.iter().cycle();
        self.update_normal(move |x| { cb(x,values2.next().unwrap()) });
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
    pub fn delta(&mut self, x_size: &[X], y_size: &[X]) {
        self.fold_tangent(x_size,|v,d| { *v = v.clone() + d.clone(); });
        self.fold_normal(y_size,|v,d| { *v = v.clone() + d.clone(); });
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
