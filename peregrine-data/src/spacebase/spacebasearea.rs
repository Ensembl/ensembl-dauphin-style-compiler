use std::{ops::{Add, Div, Sub}};
use crate::{util::{eachorevery::{EachOrEveryGroupCompatible, EachOrEveryFilter}}};
use super::{spacebase::{SpaceBase, SpaceBaseIterator, SpaceBasePointRef, PartialSpaceBase}};
use std::hash::Hash;

pub struct SpaceBaseArea<X,Y>(SpaceBase<X,Y>,SpaceBase<X,Y>,usize);

#[cfg(debug_assertions)]
impl<X: std::fmt::Debug, Y: std::fmt::Debug> std::fmt::Debug for SpaceBaseArea<X,Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"SpaceBaseArea({:?},{:?})",self.0,self.1)
    }
}

impl <X,Y> SpaceBaseArea<X,Y> {
    pub fn top_left(&self) -> &SpaceBase<X,Y> { &self.0 }
    pub fn bottom_right(&self) -> &SpaceBase<X,Y> { &self.1 }

    pub fn iter(&self) -> SpaceBaseAreaIterator<X,Y> {
        SpaceBaseAreaIterator {
            a: self.0.iter(),
            b: self.1.iter(),
        }
    }
}

impl<X: Clone, Y: Clone> SpaceBaseArea<X,Y> {
    pub fn new(top_left: PartialSpaceBase<X,Y>, bottom_right: PartialSpaceBase<X,Y>) -> Option<SpaceBaseArea<X,Y>> {
        let mut compat = EachOrEveryGroupCompatible::new(None);
        top_left.compat(&mut compat);
        bottom_right.compat(&mut compat);
        let top_left = if let Some(b) = top_left.make(&compat) { b } else { return None; };
        let bottom_right = if let Some(b) = bottom_right.make(&compat) { b } else { return None; };
        let len = top_left.len();
        Some(SpaceBaseArea(top_left,bottom_right,len))
    }

    pub fn iter_other<'a,Z>(&self, other: &'a [Z]) -> impl Iterator<Item=&'a Z> {
        let len = self.len();
        other.iter().cycle().take(len)
    }

    pub fn filter(&self, filter: &EachOrEveryFilter) -> SpaceBaseArea<X,Y> {
        SpaceBaseArea(self.0.filter(filter),self.1.filter(filter),filter.count())
    }

    pub fn fullmap_allotments_results<F,G,A: Clone,E>(&self, cb: F, cb2: G) -> Result<SpaceBaseArea<X,A>,E> 
                where F: FnMut(&Y) -> Result<A,E>, G: FnMut(&Y) -> Result<A,E> {
        Ok(SpaceBaseArea(
            self.0.fullmap_allotments_results(cb)?,
            self.1.fullmap_allotments_results(cb2)?,
            self.2
        ))
    }

    pub fn bottom_left(&self) -> SpaceBase<X,Y> { self.0.replace_normal(&self.1).unwrap() }
    pub fn top_right(&self) -> SpaceBase<X,Y> { self.1.replace_normal(&self.0).unwrap() }
}

impl<X: Clone,Y: Clone> Clone for SpaceBaseArea<X,Y> {
    fn clone(&self) -> Self {
        SpaceBaseArea(self.0.clone(),self.1.clone(),self.2)
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>, Y: Clone> SpaceBaseArea<X,Y> {
    pub fn middle_base(&self) -> SpaceBase<X,Y> { self.0.middle_base(&self.1) }
}

impl<X,Y> SpaceBaseArea<X,Y> {
    pub fn len(&self) -> usize { self.2 }

    pub fn demerge_by_allotment<F,K>(&self, cb: F) -> Vec<(K,EachOrEveryFilter)> where F: Fn(&Y) -> K, K: Hash+PartialEq+Eq {
        self.0.allotment.demerge(self.2,cb)
    }

    pub fn map_allotments<F,A>(&self, cb: F) -> SpaceBaseArea<X,A> where F: Fn(&Y) -> A {
        SpaceBaseArea(self.0.map_allotments(&cb),self.1.map_allotments(cb),self.2)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum HollowEdge2<X> { Top(X), Left(X), Bottom(X), Right(X) }

impl<X: Clone + Add<Output=X> + Sub<Output=X>, Y: Clone> SpaceBaseArea<X,Y> {
    pub fn new_from_sizes(points: &SpaceBase<X,Y>, x_size: &[X], y_size: &[X]) -> SpaceBaseArea<X,Y> {
        let mut far = points.clone();
        far.delta(x_size,y_size);
        SpaceBaseArea(points.clone(),far,points.len())
    }

    pub fn hollow_edge(&self, edge: &HollowEdge2<X>) -> SpaceBaseArea<X,Y> {
        let mut out = self.clone();
        match edge {
            HollowEdge2::Left(w) => {
                out.1.base = out.0.base.clone();
                out.1.tangent = out.0.tangent.map(|x| x.clone()+w.clone());
            },
            HollowEdge2::Right(w) => {
                out.0.base = out.1.base.clone();
                out.0.tangent = out.1.tangent.map(|x| x.clone()-w.clone());
            },
            HollowEdge2::Top(w) => {
                out.1.normal = out.0.normal.map(|x| x.clone()+w.clone());
            },
            HollowEdge2::Bottom(w) => {
                out.0.normal = out.1.normal.map(|x| x.clone()-w.clone());
            }
        }
        out
    }
}

impl<X: Clone + PartialOrd, Y: Clone> SpaceBaseArea<X,Y> {
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> EachOrEveryFilter {
        let top_left = self.0.base.make_filter(self.2, |base|
            *base <= max_value
        );
        let bottom_right = self.1.base.make_filter(self.2, |base|
            *base >= min_value
        );
        top_left.and(&bottom_right)
    }
}

pub struct SpaceBaseAreaIterator<'a,X,Y> {
    a: SpaceBaseIterator<'a,X,Y>,
    b: SpaceBaseIterator<'a,X,Y>,
}

impl<'a,X,Y> Iterator for SpaceBaseAreaIterator<'a,X,Y> {
    type Item = (SpaceBasePointRef<'a,X,Y>,SpaceBasePointRef<'a,X,Y>);

    fn next(&mut self) -> Option<Self::Item> {
        let (x,y) = (self.a.next(),self.b.next());
        if x.is_none() || y.is_none() { return None; }
        Some((x.unwrap(),y.unwrap()))
    }
}
