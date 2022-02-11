use std::ops::{Add, Div};
use std::sync::Arc;
use std::hash::Hash;

use crate::spacebase::parametric::Flattenable;
use crate::util::eachorevery::{EachOrEveryMut, EachOrEveryGroupCompatible};
use crate::{EachOrEvery, ParameterValue, Substitutions, DataFilter, AllotmentRequest, SpaceBase, HoleySpaceBase};

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

impl<X: Clone,Y: Clone> SpaceBase2<ParameterValue<X>,Y> {
    pub fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase2<X,Y> where F: Fn(SpaceBase2NumericParameterLocation) -> L {
        SpaceBase2 {
            base: self.base.flatten(subs,|x| cb(SpaceBase2NumericParameterLocation::Base(x))),
            normal: self.normal.flatten(subs,|x| cb(SpaceBase2NumericParameterLocation::Normal(x))),
            tangent: self.tangent.flatten(subs,|x| cb(SpaceBase2NumericParameterLocation::Tangent(x))),
            allotment: self.allotment.clone(),
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
pub enum HoleySpaceBase2<X: Clone,Y: Clone> {
    Simple(SpaceBase2<X,Y>),
    Parametric(SpaceBase2<ParameterValue<X>,Y>)
}

impl<X: Clone + PartialOrd,Y: Clone> HoleySpaceBase2<X,Y> {
    pub fn default_values(&self) -> SpaceBase2<X,Y> {
        match self {
            HoleySpaceBase2::Simple(x) => x.clone(),
            HoleySpaceBase2::Parametric(x) => {
                x.clone().map_all(|x| x.param_default().clone())
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            HoleySpaceBase2::Simple(x) => x.len(),
            HoleySpaceBase2::Parametric(x) => x.len()
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> HoleySpaceBase2<X,Y> {
        match self {
            HoleySpaceBase2::Simple(x) => HoleySpaceBase2::Simple(x.filter(filter)),
            HoleySpaceBase2::Parametric(x) => HoleySpaceBase2::Parametric(x.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        match self {
            HoleySpaceBase2::Simple(x) =>
                x.make_base_filter(min_value,max_value),
            HoleySpaceBase2::Parametric(x) =>
                x.make_base_filter(ParameterValue::Constant(min_value),ParameterValue::Constant(max_value))
        }
    }

    pub fn demerge_by_allotment<F,K: Hash+PartialEq+Eq>(&self, cb: F) -> Vec<(K,DataFilter)> where F: Fn(&Y) -> K {
        match self {
            HoleySpaceBase2::Simple(x) => x.allotment.demerge(cb),
            HoleySpaceBase2::Parametric(x) => x.allotment.demerge(cb)
        }
    }

    pub fn map_allotments_results<F,E,Z: Clone>(&self, cb: F) -> Result<HoleySpaceBase2<X,Z>,E> where F: Fn(&Y) -> Result<Z,E> {
        Ok(match self {
            HoleySpaceBase2::Simple(x) =>
                HoleySpaceBase2::Simple(x.map_allotments_results(cb)?),
            HoleySpaceBase2::Parametric(x) =>
                HoleySpaceBase2::Parametric(x.map_allotments_results(cb)?)
        })
    }

    // XXX should allow parameterisable Allotments, as long as they don't change CoordSystem.
    pub fn allotments(&self) -> EachOrEvery<Y> {
        match self {
            HoleySpaceBase2::Simple(x) => x.allotments().clone(),
            HoleySpaceBase2::Parametric(x) => x.allotments().clone()
        }
    }
}

impl<X: Clone,Y: Clone> Flattenable<SpaceBase2NumericParameterLocation> for HoleySpaceBase2<X,Y> {
    type Target = SpaceBase2<X,Y>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase2<X,Y> where F: Fn(SpaceBase2NumericParameterLocation) -> L {
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

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct PartialSpaceBase2<X,Y>(SpaceBase2<X,Y>);

impl<X: Clone, Y: Clone> PartialSpaceBase2<X,Y> {
    pub fn new(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> PartialSpaceBase2<X,Y> {
        PartialSpaceBase2(SpaceBase2::new_unszied(base,normal,tangent,allotment))
    }

    pub fn from_spacebase(spacebase: SpaceBase2<X,Y>) -> PartialSpaceBase2<X,Y> {
        PartialSpaceBase2(spacebase)
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

fn xxx_to_eoe<X: PartialEq>(mut input: Vec<X>) -> EachOrEvery<X> {
    let mut single_value = None;
    for v in &input {
        if let Some(old) = single_value {
            if old != v { return EachOrEvery::Each(Arc::new(input)); }
        } else {
            single_value = Some(v);
        }
    }
    if let Some(v) = input.pop() {
        EachOrEvery::Every(v)
    } else {
        EachOrEvery::Each(Arc::new(vec![]))
    }
}

fn xxx_from_eoe<X: Clone>(input: EachOrEvery<X>) -> Vec<X> {
    match input {
        EachOrEvery::Each(mut x) => Arc::make_mut(&mut x).clone(),
        EachOrEvery::Every(x) => vec![x]
    }
}

impl<X: Clone + PartialEq, Y: Clone> SpaceBase2<X,Y> {
    pub fn xxx_from_original(mut positions: SpaceBase<X>, allotments: EachOrEvery<Y>) -> SpaceBase2<X,Y> {
        let a_len = allotments.len().unwrap_or(0);
        let mut out = SpaceBase2 {
            base: xxx_to_eoe(Arc::make_mut(&mut positions.base).clone()),
            normal: xxx_to_eoe(Arc::make_mut(&mut positions.normal).clone()),
            tangent: xxx_to_eoe(Arc::make_mut(&mut positions.tangent).clone()),
            allotment: allotments,
            len: a_len.max(positions.len())
        };
        if out.base.len().is_none() && out.normal.len().is_none() && out.tangent.len().is_none() {
            out.base = out.base.to_each(1).unwrap();
        }
        out
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

    pub fn new(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> Option<SpaceBase2<X,Y>> {
        let mut out = Self::new_unszied(base,normal,tangent,allotment);
        let mut compat = EachOrEveryGroupCompatible::new(None);
        out.compat(&mut compat);
        out.len = if let Some(len) = compat.len() { len } else { return None; };
        Some(out)
    }

    pub fn allotments(&self) -> &EachOrEvery<Y> { &self.allotment }

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

    pub fn map_all<F,A: Clone>(&mut self, cb: F) -> SpaceBase2<A,Y> where F: Fn(&X) -> A {
        SpaceBase2 {
            base: self.base.map(&cb),
            tangent: self.tangent.map(&cb),
            normal: self.normal.map(&cb),
            allotment: self.allotment.clone(),
            len: self.len
        }
    }

    pub fn map_all_results<F,G,A: Clone,B: Clone,E>(&mut self, cb: F, cb2: G) -> Result<SpaceBase2<A,B>,E> 
                where F: Fn(&X) -> Result<A,E>, G: Fn(&Y) -> Result<B,E> {
        Ok(SpaceBase2 {
            base: self.base.map_results(&cb)?,
            tangent: self.tangent.map_results(&cb)?,
            normal: self.normal.map_results(&cb)?,
            allotment: self.allotment.map_results(&cb2)?,
            len: self.len
        })
    }


    pub fn map_allotments_results<F,A: Clone,E>(&self, cb: F) -> Result<SpaceBase2<X,A>,E> 
                where F: Fn(&Y) -> Result<A,E> {
        Ok(SpaceBase2 {
            base: self.base.clone(),
            tangent: self.tangent.clone(),
            normal: self.normal.clone(),
            allotment: self.allotment.map_results(&cb)?,
            len: self.len
        })
    }
    // XXX not bool, result.

    pub fn update_tangent_from_allotment<'a,F>(&mut self, cb: F) -> bool where F: Fn(&mut X,&Y) {
        let tangent = self.tangent.zip(&self.allotment,|t,a| {
            let mut t2 = t.clone();
            cb(&mut t2,a);
            t2
        });
        if let Some(tangent) = tangent {
            self.tangent = tangent;
            true
        } else {
            false
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

    pub fn update_normal_from_allotment<'a,F>(&mut self, cb: F) -> bool where F: Fn(&mut X,&Y) {
        let normal = self.normal.zip(&self.allotment,|t,a| {
            let mut t2 = t.clone();
            cb(&mut t2,a);
            t2
        });
        if let Some(normal) = normal {
            self.normal = normal;
            true
        } else {
            false
        }
    }

    pub fn fold_tangent<F,Z>(&mut self, values: &[Z], cb: F) -> bool where F: Fn(&mut X,&Z) {
        self.tangent = if let Some(t) = self.tangent.to_each(values.len()) { t.clone() } else { return false; };        
        let mut values2 = values.iter();
        self.update_tangent(move |x| { cb(x,values2.next().unwrap()) });
        true
    }

    pub fn fold_normal<F,Z>(&mut self, values: &[Z], cb: F) -> bool where F: Fn(&mut X,&Z) {
        self.normal = if let Some(t) = self.normal.to_each(values.len()) { t.clone() } else { return false; };        
        let mut values2 = values.iter();
        self.update_normal(move |x| { cb(x,values2.next().unwrap()) });
        true
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
