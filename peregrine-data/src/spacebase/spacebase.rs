use std::ops::{Add, Div};
use std::hash::Hash;

use crate::util::eachorevery::{EachOrEveryFilter, EachOrEveryGroupCompatible};
use crate::{ EachOrEvery };

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

impl<X,Y> SpaceBase<X,Y> {
    pub fn demerge_by_allotment<F,K: Hash+PartialEq+Eq>(&self, cb: F) -> Vec<(K,EachOrEveryFilter)> where F: Fn(&Y) -> K {
        self.allotment.demerge(self.len,cb)
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

impl<X,Y> Clone for SpaceBase<X,Y> {
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

impl<X,Y> SpaceBase<X,Y> {
    pub fn map_allotments<F,A>(&self, cb: F) -> SpaceBase<X,A> where F: Fn(&Y) -> A {
        SpaceBase {
            base: self.base.clone(),
            normal: self.normal.clone(),
            tangent: self.tangent.clone(),
            allotment: self.allotment.map(cb),
            len: self.len
        }
    }

    pub fn into_new_allotment<F,A>(self, cb: F) -> SpaceBase<X,A> where F: Fn(&Y) -> A {
        SpaceBase {
            base: self.base,
            normal: self.normal,
            tangent: self.tangent,
            allotment: self.allotment.clone().map(cb),
            len: self.len
        }
    }

    pub fn allotments(&self) -> &EachOrEvery<Y> { &self.allotment }

    pub fn len(&self) -> usize { self.len }
}

impl<X: Clone, Y: Clone> SpaceBase<X,Y> {
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

    pub fn merge<A,B,P: Clone,Q: Clone>(&self, other: SpaceBase<A,B>, cbs: SpaceBasePoint<&dyn (Fn(&X,&A) -> P),&dyn (Fn(&Y,&B) -> Q)>) -> SpaceBase<P,Q> {
        let base = self.base.zip(&other.base,cbs.base);
        let normal = self.normal.zip(&other.normal,cbs.normal);
        let tangent =self.tangent.zip(&other.tangent,cbs.tangent);
        let allotment = self.allotment.zip(&other.allotment,cbs.allotment);
        SpaceBase::new_unszied(&base,&normal,&tangent,&allotment)
    }

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

    pub fn filter(&self, filter: &EachOrEveryFilter) -> SpaceBase<X,Y> {
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

    pub fn map_all<F,A: Clone>(&self, cb: F) -> SpaceBase<A,Y> where F: Fn(&X) -> A {
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

    pub fn fullmap_allotments_results<F,A: Clone,E>(&self, mut cb: F) -> Result<SpaceBase<X,A>,E> 
                where F: FnMut(&Y) -> Result<A,E> {
        let allotment = if self.len>0 {
            self.allotment.to_each(self.len).unwrap().map_results(&mut cb)?
        } else {
            EachOrEvery::each(vec![])
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

    pub fn update_tangent_from_allotment<'a,F>(&mut self, cb: F) where F: Fn(&mut X,&Y) {
        self.tangent = self.tangent.zip(&self.allotment,|t,a| {
            let mut t2 = t.clone();
            cb(&mut t2,a);
            t2
        });
    }

    pub fn update_tangent<'a,F>(&mut self, cb: F) where F: Fn(&X) -> X {
        self.tangent.map_mut(cb);
    }

    pub fn update_normal<'a,F>(&mut self, cb: F) where F: Fn(&X) -> X {
        self.normal.map_mut(cb);
    }

    pub fn update_normal_from_allotment<'a,F>(&mut self, cb: F) where F: Fn(&mut X,&Y) {
        self.normal = self.normal.zip(&self.allotment,|t,a| {
            let mut t2 = t.clone();
            cb(&mut t2,a);
            t2
        });
    }

    pub fn fold_tangent<F,Z>(&mut self, values: &[Z], cb: F) -> bool where F: Fn(&X,&Z) -> X {
        self.tangent = if let Some(t) = self.tangent.to_each(values.len()) { t.clone() } else { return false; };
        self.tangent.fold_mut(values,cb);
        true
    }

    pub fn fold_normal<F,Z>(&mut self, values: &[Z], cb: F) -> bool where F: Fn(&X,&Z) -> X {
        self.normal = if let Some(n) = self.normal.to_each(values.len()) { n.clone() } else { return false; };
        self.normal.fold_mut(values,cb);
        true
    }
}

impl<X: Clone + PartialOrd,Y> SpaceBase<X,Y> {
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> EachOrEveryFilter {
        self.base.make_filter(self.len, |base| {
            let exclude =  *base >= max_value || *base < min_value;
            !exclude
        })
    }
}

impl<X: Clone + Add<Output=X>,Y: Clone> SpaceBase<X,Y> {
    pub fn delta(&mut self, x_size: &[X], y_size: &[X]) {
        self.fold_tangent(x_size,|v,d| { v.clone() + d.clone() });
        self.fold_normal(y_size,|v,d| { v.clone() + d.clone() });
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
    pub fn middle_base(&self, other: &SpaceBase<X,Y>) -> SpaceBase<X,Y> {
        let base =  self.base.zip(&other.base, |a,b| (a.clone()+b.clone())/2.);
        SpaceBase {
            base,
            tangent: self.tangent.clone(),
            normal: self.normal.clone(),
            allotment: self.allotment.clone(),
            len: self.len
        }
    }
}
