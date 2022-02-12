use std::{ops::{Add, Div, Sub}, hash::Hash};
use crate::{util::{ringarray::{ DataFilter }, eachorevery::EachOrEveryGroupCompatible}, AllotmentRequest, EachOrEvery};
use super::{parametric::{Flattenable, ParameterValue, ParametricType, Substitutions}, spacebase::{SpaceBase, SpaceBaseNumericParameterLocation, SpaceBaseAllotmentParameterLocation, SpaceBaseIterator, SpaceBasePointRef, PartialSpaceBase}};

pub struct SpaceBaseArea<X,Y>(SpaceBase<X,Y>,SpaceBase<X,Y>,usize);

#[cfg(debug_assertions)]
impl<X: std::fmt::Debug, Y: std::fmt::Debug> std::fmt::Debug for SpaceBaseArea<X,Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"SpaceBaseArea({:?},{:?})",self.0,self.1)
    }
}

pub enum SpaceBaseAreaNumericParameterLocation {
    Left(SpaceBaseNumericParameterLocation),
    Right(SpaceBaseNumericParameterLocation)
}

impl<X: Clone, Y: Clone> ParametricType<SpaceBaseAreaNumericParameterLocation> for SpaceBaseArea<X,Y> {
    type Value = X;

    fn replace(&mut self, replace: &[(&SpaceBaseAreaNumericParameterLocation,X)]) {
        let mut left_replace = vec![];
        let mut right_replace = vec![];
        for (location,value) in replace.iter() {
            match location {
                SpaceBaseAreaNumericParameterLocation::Left(x) => { left_replace.push((x,value.clone())); },
                SpaceBaseAreaNumericParameterLocation::Right(x) => { right_replace.push((x,value.clone())); },
            }
        }
        self.0.replace(&left_replace);
        self.1.replace(&right_replace);
    }
}

pub enum SpaceBaseAreaAllotmentParameterLocation {
    Left(SpaceBaseAllotmentParameterLocation),
    Right(SpaceBaseAllotmentParameterLocation)
}

impl<X: Clone, Y: Clone> ParametricType<SpaceBaseAreaAllotmentParameterLocation> for SpaceBaseArea<X,Y> {
    type Value = Y;

    fn replace(&mut self, replace: &[(&SpaceBaseAreaAllotmentParameterLocation,Y)]) {
        let mut left_replace = vec![];
        let mut right_replace = vec![];
        for (location,value) in replace.iter() {
            match location {
                SpaceBaseAreaAllotmentParameterLocation::Left(x) => { left_replace.push((x,value.clone())); },
                SpaceBaseAreaAllotmentParameterLocation::Right(x) => { right_replace.push((x,value.clone())); },
            }
        }
        self.0.replace(&left_replace);
        self.1.replace(&right_replace);
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum HoleySpaceBaseArea<X: Clone,Y: Clone> {
    Simple(SpaceBaseArea<X,Y>),
    Parametric(SpaceBaseArea<ParameterValue<X>,Y>)
}

impl<X: Clone + PartialOrd,Y: Clone> HoleySpaceBaseArea<X,Y> {
    pub fn map_allotments_results<F,G,E,Z: Clone>(&self, mut cb: F, mut cb2: G) -> Result<HoleySpaceBaseArea<X,Z>,E> 
            where F: FnMut(&Y) -> Result<Z,E>, G: FnMut(&Y) -> Result<Z,E> {
        Ok(match self {
            HoleySpaceBaseArea::Simple(x) =>
                HoleySpaceBaseArea::Simple(x.map_allotments_results(cb,cb2)?),
            HoleySpaceBaseArea::Parametric(x) =>
                HoleySpaceBaseArea::Parametric(x.map_allotments_results(cb,cb2)?)
        })
    }

    pub fn len(&self) -> usize {
        match self {
            HoleySpaceBaseArea::Simple(x) => x.len(),
            HoleySpaceBaseArea::Parametric(x) => x.len()
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> HoleySpaceBaseArea<X,Y> {
        match self {
            HoleySpaceBaseArea::Simple(x) => {
                HoleySpaceBaseArea::Simple(x.filter(filter))
            },
            HoleySpaceBaseArea::Parametric(x) => HoleySpaceBaseArea::Parametric(x.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        match self {
            HoleySpaceBaseArea::Simple(x) =>
                x.make_base_filter(min_value,max_value),
            HoleySpaceBaseArea::Parametric(x) =>
                x.make_base_filter(ParameterValue::Constant(min_value),ParameterValue::Constant(max_value))
        }
    }

    pub fn allotments(&self) -> EachOrEvery<Y> {
        match self {
            HoleySpaceBaseArea::Simple(x) => x.0.allotments().clone(),
            HoleySpaceBaseArea::Parametric(x) => x.0.allotments().clone()
        }
    }

    pub fn demerge_by_allotment<F,K: Hash+PartialEq+Eq>(&self, cb: F) -> Vec<(K,DataFilter)> where F: Fn(&Y) -> K {
        match self {
            HoleySpaceBaseArea::Simple(x) => x.0.allotments().demerge(cb),
            HoleySpaceBaseArea::Parametric(x) => x.0.allotments().demerge(|x| cb(x))
        }
    }
}

impl<X: Clone, Y: Clone> SpaceBaseArea<ParameterValue<X>,Y> {
    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBaseArea<X,Y> where F: Fn(SpaceBaseAreaNumericParameterLocation) -> L {
        let left = self.0.flatten(subs,|location: SpaceBaseNumericParameterLocation| cb(SpaceBaseAreaNumericParameterLocation::Left(location)));
        let right = self.1.flatten(subs,|location: SpaceBaseNumericParameterLocation| cb(SpaceBaseAreaNumericParameterLocation::Right(location)));
        SpaceBaseArea(left,right,self.2)
    }
}

impl<X: Clone,Y: Clone> Flattenable<SpaceBaseAreaNumericParameterLocation> for HoleySpaceBaseArea<X,Y> {
    type Target = SpaceBaseArea<X,Y>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBaseArea<X,Y> where F: Fn(SpaceBaseAreaNumericParameterLocation) -> L {
        match self {
            HoleySpaceBaseArea::Simple(x) => x.clone(),
            HoleySpaceBaseArea::Parametric(x) => x.flatten(subs,cb)
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

    pub fn len(&self) -> usize { self.2 }

    pub fn iter(&self) -> SpaceBaseAreaIterator<X,Y> {
        SpaceBaseAreaIterator {
            a: self.0.iter(),
            b: self.1.iter(),
        }
    }

    pub fn iter_other<'a,Z>(&self, other: &'a [Z]) -> impl Iterator<Item=&'a Z> {
        let len = self.len();
        other.iter().cycle().take(len)
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBaseArea<X,Y> {
        SpaceBaseArea(self.0.filter(filter),self.1.filter(filter),filter.count())
    }

    pub fn map_allotments_results<F,G,A: Clone,E>(&self, cb: F, cb2: G) -> Result<SpaceBaseArea<X,A>,E> 
                where F: FnMut(&Y) -> Result<A,E>, G: FnMut(&Y) -> Result<A,E> {
        Ok(SpaceBaseArea(
            self.0.map_allotments_results(cb)?,
            self.1.map_allotments_results(cb2)?,
            self.2
        ))
    }

    pub fn top_left(&self) -> &SpaceBase<X,Y> { &self.0 }
    pub fn bottom_right(&self) -> &SpaceBase<X,Y> { &self.1 }
    pub fn bottom_left(&self) -> SpaceBase<X,Y> { self.0.replace_normal(&self.1).unwrap() }
    pub fn top_right(&self) -> SpaceBase<X,Y> { self.1.replace_normal(&self.0).unwrap() }
}

impl<X: Clone,Y: Clone> Clone for SpaceBaseArea<X,Y> {
    fn clone(&self) -> Self {
        SpaceBaseArea(self.0.clone(),self.1.clone(),self.2)
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>, Y: Clone> SpaceBaseArea<X,Y> {
    pub fn middle_base(&self) -> SpaceBase<X,Y> { self.0.middle_base(&self.1).unwrap() }
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
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        let top_left = DataFilter::new(&mut self.0.base.iter(self.2).unwrap(),|base| {
            *base <= max_value
        });
        let bottom_right = DataFilter::new(&mut self.1.base.iter(self.2).unwrap(),|base| {
            *base >= min_value
        });
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
