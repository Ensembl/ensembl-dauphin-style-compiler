use std::ops::{Add, Div};
use std::sync::Arc;

use crate::spacebase::parametric::Flattenable;
use crate::util::eachorevery::{EachOrEveryMut, EachOrEveryGroupCompatible};
use crate::{EachOrEvery, ParameterValue, Substitutions, DataFilter};

use super::parametric::{ParametricType};

pub struct SpaceBase2Point<X,Y> {
    pub base: X,
    pub normal: X,
    pub tangent: X,
    pub allotment: Y
}

impl<X,Y> SpaceBase2Point<X,Y> {
    pub fn as_ref(&self) -> SpaceBase2PointRef<X,Y> {
        SpaceBase2PointRef {
            base: &self.base,
            normal: &self.normal,
            tangent: &self.tangent,
            allotment: &self.allotment
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct SpaceBase2PointRef<'a,X,Y> {
    pub base: &'a X,
    pub normal: &'a X,
    pub tangent: &'a X,
    pub allotment: &'a Y
}

impl<'a,X: Clone,Y: Clone> SpaceBase2PointRef<'a,X,Y> {
    pub fn make(&self) -> SpaceBase2Point<X,Y> {
        SpaceBase2Point {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            allotment: self.allotment.clone()
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct SpaceBase2<X,Y> {
    pub(super) base: EachOrEvery<X>,
    pub(super) normal: EachOrEvery<X>,
    pub(super) tangent: EachOrEvery<X>,
    pub(super) allotment: EachOrEvery<Y>,
    len: usize
}

pub enum SpaceBase2NumericParameterLocation {
    Base(usize),
    Normal(usize),
    Tangent(usize),
}

pub enum SpaceBase2AllotmentParameterLocation {
    Allotment(usize)
}

pub enum SpaceBase2ParameterLocation {
    Numeric(SpaceBase2NumericParameterLocation),
    Allotment(SpaceBase2AllotmentParameterLocation)
}

impl<X: Clone, Y: Clone> SpaceBase2<ParameterValue<X>,ParameterValue<Y>> {
    pub(crate) fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase2<X,Y> where F: Fn(SpaceBase2ParameterLocation) -> L {
        SpaceBase2 {
            base: self.base.flatten(subs,|x| cb(SpaceBase2ParameterLocation::Numeric(SpaceBase2NumericParameterLocation::Base(x)))),
            normal: self.normal.flatten(subs,|x| cb(SpaceBase2ParameterLocation::Numeric(SpaceBase2NumericParameterLocation::Normal(x)))),
            tangent: self.tangent.flatten(subs,|x| cb(SpaceBase2ParameterLocation::Numeric(SpaceBase2NumericParameterLocation::Tangent(x)))),
            allotment: self.allotment.flatten(subs,|x| cb(SpaceBase2ParameterLocation::Allotment(SpaceBase2AllotmentParameterLocation::Allotment(x)))),
            len: self.len.clone()
        }
    }
}

impl<X: Clone,Y> ParametricType<SpaceBase2NumericParameterLocation> for SpaceBase2<X,Y> {
    type Value = X;

    fn replace(&mut self, replace: &[(&SpaceBase2NumericParameterLocation,X)]) {
        let mut go_base = false;
        let mut go_normal = false;
        let mut go_tangent = false;
        for (location,_) in replace {
            match location {
                SpaceBase2NumericParameterLocation::Base(_) => { go_base = true; },
                SpaceBase2NumericParameterLocation::Normal(_) => { go_normal = true; },
                SpaceBase2NumericParameterLocation::Tangent(_) => { go_tangent = true;  },
            }
        }
        let mut base = if go_base { Some(self.base.as_builder()) } else { None };
        let mut normal = if go_normal { Some(self.normal.as_builder()) } else { None };
        let mut tangent = if go_tangent { Some(self.tangent.as_builder()) } else { None };
        for (location,value) in replace {
            let (position,index) = match location {
                SpaceBase2NumericParameterLocation::Base(index) => { (base.as_mut().unwrap(),*index) },
                SpaceBase2NumericParameterLocation::Normal(index) => { (normal.as_mut().unwrap(),*index) },
                SpaceBase2NumericParameterLocation::Tangent(index) => { (tangent.as_mut().unwrap(),*index) }
            };
            match position.as_mut() {
                EachOrEveryMut::Each(location) => { location[index] = value.clone(); },
                EachOrEveryMut::Every(location) => { *location = value.clone(); }
            }
        }
        if go_base { self.base = base.unwrap().make(); }
        if go_normal { self.normal = normal.unwrap().make(); }
        if go_tangent { self.tangent = tangent.unwrap().make(); }
    }
}

impl<X,Y: Clone> ParametricType<SpaceBase2AllotmentParameterLocation> for SpaceBase2<X,Y> {
    type Value = Y;

    fn replace(&mut self, replace: &[(&SpaceBase2AllotmentParameterLocation,Y)]) {
        if replace.len() > 0 {
            let mut builder = self.allotment.as_builder();
            for (location,value) in replace {
                match location {
                    SpaceBase2AllotmentParameterLocation::Allotment(index) => {
                        match builder.as_mut() {
                            EachOrEveryMut::Each(location) => { location[*index] = value.clone(); },
                            EachOrEveryMut::Every(location) => { *location = value.clone(); }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum HoleySpaceBase2 {
    Simple(SpaceBase2<f64,String>),
    Parametric(SpaceBase2<ParameterValue<f64>,ParameterValue<String>>)
}

impl HoleySpaceBase2 {
    pub fn default_values(&self) -> SpaceBase2<f64,String> {
        match self {
            HoleySpaceBase2::Simple(x) => x.clone(),
            HoleySpaceBase2::Parametric(x) => {
                x.clone().map_all(|x| *x.param_default(),|y| y.param_default().clone())
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            HoleySpaceBase2::Simple(x) => x.len(),
            HoleySpaceBase2::Parametric(x) => x.len()
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> HoleySpaceBase2 {
        match self {
            HoleySpaceBase2::Simple(x) => HoleySpaceBase2::Simple(x.filter(filter)),
            HoleySpaceBase2::Parametric(x) => HoleySpaceBase2::Parametric(x.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min_value: f64, max_value: f64) -> DataFilter {
        match self {
            HoleySpaceBase2::Simple(x) =>
                x.make_base_filter(min_value,max_value),
            HoleySpaceBase2::Parametric(x) =>
                x.make_base_filter(ParameterValue::Constant(min_value),ParameterValue::Constant(max_value))
        }
    }
}

impl Flattenable for HoleySpaceBase2 {
    type Location = SpaceBase2ParameterLocation;
    type Target = SpaceBase2<f64,String>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase2<f64,String> where F: Fn(Self::Location) -> L {
        match self {
            HoleySpaceBase2::Simple(x) => x.clone(),
            HoleySpaceBase2::Parametric(x) => x.flatten(subs,cb)
        }
    }
}

pub struct SpaceBase2Iterator<'a,X,Y> {
    item: Box<dyn Iterator<Item=(((&'a X,&'a X),&'a X),&'a Y)> + 'a>,
}

impl<'a,X,Y> Iterator for SpaceBase2Iterator<'a,X,Y> {
    type Item = SpaceBase2PointRef<'a,X,Y>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some ((((base,normal),tangent),allotment)) = self.item.next() {
            Some(SpaceBase2PointRef { base, normal, tangent, allotment })
        } else {
            None
        }
    }
}

impl<X: Clone,Y: Clone> Clone for SpaceBase2<X,Y> {
    fn clone(&self) -> Self {
        SpaceBase2 {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            allotment: self.allotment.clone(),
            len: self.len
        }
    }
}

pub struct PartialSpaceBase2<X,Y>(SpaceBase2<X,Y>);



impl<X: Clone, Y: Clone> PartialSpaceBase2<X,Y> {
    pub fn new(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> PartialSpaceBase2<X,Y> {
        PartialSpaceBase2(SpaceBase2::new_unszied(base,normal,tangent,allotment))
    }

    pub fn compat(&self,compat: &mut EachOrEveryGroupCompatible) {
        self.0.compat(compat);
    }

    pub fn make(mut self, compat: &EachOrEveryGroupCompatible) -> Option<SpaceBase2<X,Y>> {
        let compat_len = if let Some(len) = compat.len() { len } else { return None; };
        self.0.len = compat_len;
        Some(self.0)
    }
}

impl<X: Clone, Y: Clone> SpaceBase2<X,Y> {
    pub fn len(&self) -> usize { self.len }

    fn compat(&self, compat: &mut EachOrEveryGroupCompatible) {
        compat.add(&self.base);
        compat.add(&self.normal);
        compat.add(&self.tangent);
        compat.add(&self.allotment);
    }

    fn new_unszied(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> SpaceBase2<X,Y> {
        SpaceBase2 {
            base: base.clone(),
            normal: normal.clone(),
            tangent: tangent.clone(),
            allotment: allotment.clone(),
            len: 0
        }
    }

    fn new(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> Option<SpaceBase2<X,Y>> {
        let mut out = Self::new_unszied(base,normal,tangent,allotment);
        let mut compat = EachOrEveryGroupCompatible::new(None);
        out.compat(&mut compat);
        out.len = if let Some(len) = compat.len() { len } else { return None; };
        Some(out)
    }

    pub fn iter<'a>(&'a self) -> SpaceBase2Iterator<'a,X,Y> {
        let base = self.base.iter(self.len).unwrap();
        let normal = self.normal.iter(self.len).unwrap();
        let tangent = self.tangent.iter(self.len).unwrap();
        let allotment = self.allotment.iter(self.len).unwrap();
        let item = base.zip(normal).zip(tangent).zip(allotment);
        SpaceBase2Iterator {
            item: Box::new(item)
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBase2<X,Y> {
        SpaceBase2 {
            base: self.base.filter(filter),
            normal: self.normal.filter(filter),
            tangent: self.tangent.filter(filter),
            allotment:self.allotment.filter(filter),
            len: filter.count()
        }
    }

    pub fn replace_normal(&self, other: &SpaceBase2<X,Y>) -> Option<SpaceBase2<X,Y>> {
        SpaceBase2::new(&self.base,&other.normal,&self.tangent,&self.allotment)
    }

    pub fn map_all<F,G,A: Clone,B: Clone>(&mut self, cb: F, cb2: G) -> SpaceBase2<A,B> where F: Fn(&X) -> A, G: Fn(&Y) -> B {
        SpaceBase2 {
            base: self.base.map(&cb),
            tangent: self.tangent.map(&cb),
            normal: self.normal.map(&cb),
            allotment: self.allotment.map(&cb2),
            len: self.len
        }
    }

    pub fn update_tangent<'a,F>(&mut self, cb: F) where F: FnMut(&mut X) {
        let mut builder = self.tangent.as_builder();
        builder.as_mut().map(cb);
        self.tangent = builder.make();
    }

    pub fn update_normal<'a,F>(&mut self, cb: F) where F: FnMut(&mut X) {
        let mut builder = self.normal.as_builder();
        builder.as_mut().map(cb);
        self.normal = builder.make();
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

impl<X: Clone + PartialOrd,Y> SpaceBase2<X,Y> {
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        let mut filter = DataFilter::new(&mut self.base.iter(self.len).unwrap(),|base| {
            let exclude =  *base >= max_value || *base < min_value;
            !exclude
        });
        filter.set_size(self.len);
        filter
    }
}

impl<X: Clone + Add<Output=X>,Y: Clone> SpaceBase2<X,Y> {
    pub fn delta(&mut self, x_size: &[X], y_size: &[X]) {
        self.fold_tangent(x_size,|v,d| { *v = v.clone() + d.clone(); });
        self.fold_normal(y_size,|v,d| { *v = v.clone() + d.clone(); });
    }

    pub fn nudge_normal(&self, amt: X) -> SpaceBase2<X,Y> {
        SpaceBase2 {
            base: self.base.clone(),
            tangent: self.tangent.clone(),
            normal: self.normal.map(|x| x.clone()+amt.clone()),
            allotment: self.allotment.clone(),
            len: self.len
        }        
    }

    pub fn nudge_tangent(&self, amt: X) -> SpaceBase2<X,Y> {
        SpaceBase2 {
            base: self.base.clone(),
            tangent: self.tangent.map(|x| x.clone()+amt.clone()),
            normal: self.normal.clone(),
            allotment: self.allotment.clone(),
            len: self.len
        }        
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>,Y: Clone> SpaceBase2<X,Y> {
    pub fn middle_base(&self, other: &SpaceBase2<X,Y>) -> Option<SpaceBase2<X,Y>> {
        self.base.zip(&other.base, |a,b| (a.clone()+b.clone())/2.).map(|base| 
            SpaceBase2 {
                base,
                tangent: self.tangent.clone(),
                normal: self.normal.clone(),
                allotment: self.allotment.clone(),
                len: self.len
            }
        )
    }
}
