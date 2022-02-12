use std::ops::{Add, Div};
use std::sync::Arc;
use std::hash::Hash;

use crate::spacebase::parametric::Flattenable;
use crate::util::eachorevery::{EachOrEveryMut, EachOrEveryGroupCompatible, eoe_throw};
use crate::{EachOrEvery, ParameterValue, Substitutions, DataFilter, AllotmentRequest};

use super::parametric::{ParametricType};

pub struct SpaceBasePoint<X,Y> {
    pub base: X,
    pub normal: X,
    pub tangent: X,
    pub allotment: Y
}

impl<X,Y> SpaceBasePoint<X,Y> {
    pub fn as_ref(&self) -> SpaceBasePointRef<X,Y> {
        SpaceBasePointRef {
            base: &self.base,
            normal: &self.normal,
            tangent: &self.tangent,
            allotment: &self.allotment
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct SpaceBasePointRef<'a,X,Y> {
    pub base: &'a X,
    pub normal: &'a X,
    pub tangent: &'a X,
    pub allotment: &'a Y
}

impl<'a,X: Clone,Y: Clone> SpaceBasePointRef<'a,X,Y> {
    pub fn make(&self) -> SpaceBasePoint<X,Y> {
        SpaceBasePoint {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            allotment: self.allotment.clone()
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct SpaceBase<X,Y> {
    pub(super) base: EachOrEvery<X>,
    pub(super) normal: EachOrEvery<X>,
    pub(super) tangent: EachOrEvery<X>,
    pub(super) allotment: EachOrEvery<Y>,
    len: usize
}

pub enum SpaceBaseNumericParameterLocation {
    Base(usize),
    Normal(usize),
    Tangent(usize),
}

pub enum SpaceBaseAllotmentParameterLocation {
    Allotment(usize)
}

impl<X: Clone,Y: Clone> SpaceBase<ParameterValue<X>,Y> {
    pub fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase<X,Y> where F: Fn(SpaceBaseNumericParameterLocation) -> L {
        SpaceBase {
            base: self.base.flatten(subs,|x| cb(SpaceBaseNumericParameterLocation::Base(x))),
            normal: self.normal.flatten(subs,|x| cb(SpaceBaseNumericParameterLocation::Normal(x))),
            tangent: self.tangent.flatten(subs,|x| cb(SpaceBaseNumericParameterLocation::Tangent(x))),
            allotment: self.allotment.clone(),
            len: self.len.clone()
        }
    }
}

impl<X: Clone,Y> ParametricType<SpaceBaseNumericParameterLocation> for SpaceBase<X,Y> {
    type Value = X;

    fn replace(&mut self, replace: &[(&SpaceBaseNumericParameterLocation,X)]) {
        let mut go_base = false;
        let mut go_normal = false;
        let mut go_tangent = false;
        for (location,_) in replace {
            match location {
                SpaceBaseNumericParameterLocation::Base(_) => { go_base = true; },
                SpaceBaseNumericParameterLocation::Normal(_) => { go_normal = true; },
                SpaceBaseNumericParameterLocation::Tangent(_) => { go_tangent = true;  },
            }
        }
        let mut base = if go_base { Some(self.base.as_builder()) } else { None };
        let mut normal = if go_normal { Some(self.normal.as_builder()) } else { None };
        let mut tangent = if go_tangent { Some(self.tangent.as_builder()) } else { None };
        for (location,value) in replace {
            let (position,index) = match location {
                SpaceBaseNumericParameterLocation::Base(index) => { (base.as_mut().unwrap(),*index) },
                SpaceBaseNumericParameterLocation::Normal(index) => { (normal.as_mut().unwrap(),*index) },
                SpaceBaseNumericParameterLocation::Tangent(index) => { (tangent.as_mut().unwrap(),*index) }
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

impl<X,Y: Clone> ParametricType<SpaceBaseAllotmentParameterLocation> for SpaceBase<X,Y> {
    type Value = Y;

    fn replace(&mut self, replace: &[(&SpaceBaseAllotmentParameterLocation,Y)]) {
        if replace.len() > 0 {
            let mut builder = self.allotment.as_builder();
            for (location,value) in replace {
                match location {
                    SpaceBaseAllotmentParameterLocation::Allotment(index) => {
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
pub enum HoleySpaceBase<X: Clone,Y: Clone> {
    Simple(SpaceBase<X,Y>),
    Parametric(SpaceBase<ParameterValue<X>,Y>)
}

impl<X: Clone + PartialOrd,Y: Clone> HoleySpaceBase<X,Y> {
    pub fn default_values(&self) -> SpaceBase<X,Y> {
        match self {
            HoleySpaceBase::Simple(x) => x.clone(),
            HoleySpaceBase::Parametric(x) => {
                x.clone().map_all(|x| x.param_default().clone())
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            HoleySpaceBase::Simple(x) => x.len(),
            HoleySpaceBase::Parametric(x) => x.len()
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> HoleySpaceBase<X,Y> {
        match self {
            HoleySpaceBase::Simple(x) => HoleySpaceBase::Simple(x.filter(filter)),
            HoleySpaceBase::Parametric(x) => HoleySpaceBase::Parametric(x.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        match self {
            HoleySpaceBase::Simple(x) =>
                x.make_base_filter(min_value,max_value),
            HoleySpaceBase::Parametric(x) =>
                x.make_base_filter(ParameterValue::Constant(min_value),ParameterValue::Constant(max_value))
        }
    }

    pub fn demerge_by_allotment<F,K: Hash+PartialEq+Eq>(&self, cb: F) -> Vec<(K,DataFilter)> where F: Fn(&Y) -> K {
        match self {
            HoleySpaceBase::Simple(x) => x.allotment.demerge(cb),
            HoleySpaceBase::Parametric(x) => x.allotment.demerge(cb)
        }
    }

    pub fn map_allotments_results<F,E,Z: Clone>(&self, cb: F) -> Result<HoleySpaceBase<X,Z>,E> where F: FnMut(&Y) -> Result<Z,E> {
        Ok(match self {
            HoleySpaceBase::Simple(x) =>
                HoleySpaceBase::Simple(x.map_allotments_results(cb)?),
            HoleySpaceBase::Parametric(x) =>
                HoleySpaceBase::Parametric(x.map_allotments_results(cb)?)
        })
    }

    // XXX should allow parameterisable Allotments, as long as they don't change CoordSystem.
    pub fn allotments(&self) -> EachOrEvery<Y> {
        match self {
            HoleySpaceBase::Simple(x) => x.allotments().clone(),
            HoleySpaceBase::Parametric(x) => x.allotments().clone()
        }
    }
}

impl<X: Clone,Y: Clone> Flattenable<SpaceBaseNumericParameterLocation> for HoleySpaceBase<X,Y> {
    type Target = SpaceBase<X,Y>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBase<X,Y> where F: Fn(SpaceBaseNumericParameterLocation) -> L {
        match self {
            HoleySpaceBase::Simple(x) => x.clone(),
            HoleySpaceBase::Parametric(x) => x.flatten(subs,cb)
        }
    }
}

pub struct SpaceBaseIterator<'a,X,Y> {
    item: Box<dyn Iterator<Item=(((&'a X,&'a X),&'a X),&'a Y)> + 'a>,
}

impl<'a,X,Y> Iterator for SpaceBaseIterator<'a,X,Y> {
    type Item = SpaceBasePointRef<'a,X,Y>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some ((((base,normal),tangent),allotment)) = self.item.next() {
            Some(SpaceBasePointRef { base, normal, tangent, allotment })
        } else {
            None
        }
    }
}

impl<X: Clone,Y: Clone> Clone for SpaceBase<X,Y> {
    fn clone(&self) -> Self {
        SpaceBase {
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
pub struct PartialSpaceBase<X,Y>(SpaceBase<X,Y>);

impl<X: Clone, Y: Clone> PartialSpaceBase<X,Y> {
    pub fn new(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> PartialSpaceBase<X,Y> {
        PartialSpaceBase(SpaceBase::new_unszied(base,normal,tangent,allotment))
    }

    pub fn from_spacebase(spacebase: SpaceBase<X,Y>) -> PartialSpaceBase<X,Y> {
        PartialSpaceBase(spacebase)
    }

    pub fn compat(&self,compat: &mut EachOrEveryGroupCompatible) {
        self.0.compat(compat);
    }

    pub fn make(mut self, compat: &EachOrEveryGroupCompatible) -> Option<SpaceBase<X,Y>> {
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

impl<X: Clone, Y: Clone> SpaceBase<X,Y> {
    pub fn len(&self) -> usize { self.len }

    fn compat(&self, compat: &mut EachOrEveryGroupCompatible) {
        compat.add(&self.base);
        compat.add(&self.normal);
        compat.add(&self.tangent);
        compat.add(&self.allotment);
    }

    fn new_unszied(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> SpaceBase<X,Y> {
        SpaceBase {
            base: base.clone(),
            normal: normal.clone(),
            tangent: tangent.clone(),
            allotment: allotment.clone(),
            len: 0
        }
    }

    pub fn new(base: &EachOrEvery<X>, normal: &EachOrEvery<X>, tangent: &EachOrEvery<X>, allotment: &EachOrEvery<Y>) -> Option<SpaceBase<X,Y>> {
        let mut out = Self::new_unszied(base,normal,tangent,allotment);
        let mut compat = EachOrEveryGroupCompatible::new(None);
        out.compat(&mut compat);
        out.len = if let Some(len) = compat.len() { len } else { return None; };
        Some(out)
    }

    pub fn allotments(&self) -> &EachOrEvery<Y> { &self.allotment }

    pub fn iter<'a>(&'a self) -> SpaceBaseIterator<'a,X,Y> {
        let base = self.base.iter(self.len).unwrap();
        let normal = self.normal.iter(self.len).unwrap();
        let tangent = self.tangent.iter(self.len).unwrap();
        let allotment = self.allotment.iter(self.len).unwrap();
        let item = base.zip(normal).zip(tangent).zip(allotment);
        SpaceBaseIterator {
            item: Box::new(item)
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBase<X,Y> {
        SpaceBase {
            base: self.base.filter(filter),
            normal: self.normal.filter(filter),
            tangent: self.tangent.filter(filter),
            allotment:self.allotment.filter(filter),
            len: filter.count()
        }
    }

    pub fn replace_normal(&self, other: &SpaceBase<X,Y>) -> Option<SpaceBase<X,Y>> {
        SpaceBase::new(&self.base,&other.normal,&self.tangent,&self.allotment)
    }

    pub fn map_all<F,A: Clone>(&mut self, cb: F) -> SpaceBase<A,Y> where F: Fn(&X) -> A {
        SpaceBase {
            base: self.base.map(&cb),
            tangent: self.tangent.map(&cb),
            normal: self.normal.map(&cb),
            allotment: self.allotment.clone(),
            len: self.len
        }
    }

    pub fn map_all_results<F,G,A: Clone,B: Clone,E>(&mut self, cb: F, cb2: G) -> Result<SpaceBase<A,B>,E> 
                where F: Fn(&X) -> Result<A,E>, G: Fn(&Y) -> Result<B,E> {
        Ok(SpaceBase {
            base: self.base.map_results(&cb)?,
            tangent: self.tangent.map_results(&cb)?,
            normal: self.normal.map_results(&cb)?,
            allotment: self.allotment.map_results(&cb2)?,
            len: self.len
        })
    }


    pub fn map_allotments_results<F,A: Clone,E>(&self, mut cb: F) -> Result<SpaceBase<X,A>,E> 
                where F: FnMut(&Y) -> Result<A,E> {
        let allotment = if self.len>0 {
            self.allotment.to_each(self.len).unwrap().map_results(&mut cb)?
        } else {
            EachOrEvery::Each(Arc::new(vec![]))
        };
        Ok(SpaceBase {
            base: self.base.clone(),
            tangent: self.tangent.clone(),
            normal: self.normal.clone(),
            allotment,
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

impl<X: Clone + PartialOrd,Y> SpaceBase<X,Y> {
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        let mut filter = DataFilter::new(&mut self.base.iter(self.len).unwrap(),|base| {
            let exclude =  *base >= max_value || *base < min_value;
            !exclude
        });
        filter.set_size(self.len);
        filter
    }
}

impl<X: Clone + Add<Output=X>,Y: Clone> SpaceBase<X,Y> {
    pub fn delta(&mut self, x_size: &[X], y_size: &[X]) {
        self.fold_tangent(x_size,|v,d| { *v = v.clone() + d.clone(); });
        self.fold_normal(y_size,|v,d| { *v = v.clone() + d.clone(); });
    }

    pub fn nudge_normal(&self, amt: X) -> SpaceBase<X,Y> {
        SpaceBase {
            base: self.base.clone(),
            tangent: self.tangent.clone(),
            normal: self.normal.map(|x| x.clone()+amt.clone()),
            allotment: self.allotment.clone(),
            len: self.len
        }        
    }

    pub fn nudge_tangent(&self, amt: X) -> SpaceBase<X,Y> {
        SpaceBase {
            base: self.base.clone(),
            tangent: self.tangent.map(|x| x.clone()+amt.clone()),
            normal: self.normal.clone(),
            allotment: self.allotment.clone(),
            len: self.len
        }        
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>,Y: Clone> SpaceBase<X,Y> {
    pub fn middle_base(&self, other: &SpaceBase<X,Y>) -> Option<SpaceBase<X,Y>> {
        self.base.zip(&other.base, |a,b| (a.clone()+b.clone())/2.).map(|base| 
            SpaceBase {
                base,
                tangent: self.tangent.clone(),
                normal: self.normal.clone(),
                allotment: self.allotment.clone(),
                len: self.len
            }
        )
    }
}
