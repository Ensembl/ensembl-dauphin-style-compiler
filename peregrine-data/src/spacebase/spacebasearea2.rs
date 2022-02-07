use std::{ops::{Add, Div, Sub}, sync::Arc};
use crate::{util::{ringarray::{ DataFilter }, eachorevery::EachOrEveryGroupCompatible}, AllotmentRequest};
use super::{parametric::{Flattenable, ParameterValue, ParametricType, Substitutions}, spacebase::{SpaceBase, SpaceBaseIterator, SpaceBaseParameterLocation, SpaceBasePointRef}, spacebase2::{SpaceBase2, SpaceBase2ParameterLocation, SpaceBase2NumericParameterLocation, SpaceBase2AllotmentParameterLocation, SpaceBase2Iterator, SpaceBase2PointRef, PartialSpaceBase2}};

pub struct SpaceBaseArea2<X,Y>(SpaceBase2<X,Y>,SpaceBase2<X,Y>,usize);

#[cfg(debug_assertions)]
impl<X: std::fmt::Debug, Y: std::fmt::Debug> std::fmt::Debug for SpaceBaseArea2<X,Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"SpaceBaseArea({:?},{:?})",self.0,self.1)
    }
}

pub enum SpaceBaseArea2NumericParameterLocation {
    Left(SpaceBase2NumericParameterLocation),
    Right(SpaceBase2NumericParameterLocation)
}

impl<X: Clone, Y: Clone> ParametricType<SpaceBaseArea2NumericParameterLocation> for SpaceBaseArea2<X,Y> {
    type Value = X;

    fn replace(&mut self, replace: &[(&SpaceBaseArea2NumericParameterLocation,X)]) {
        let mut left_replace = vec![];
        let mut right_replace = vec![];
        for (location,value) in replace.iter() {
            match location {
                SpaceBaseArea2NumericParameterLocation::Left(x) => { left_replace.push((x,value.clone())); },
                SpaceBaseArea2NumericParameterLocation::Right(x) => { right_replace.push((x,value.clone())); },
            }
        }
        self.0.replace(&left_replace);
        self.1.replace(&right_replace);
    }
}

pub enum SpaceBaseArea2AllotmentParameterLocation {
    Left(SpaceBase2AllotmentParameterLocation),
    Right(SpaceBase2AllotmentParameterLocation)
}

impl<X: Clone, Y: Clone> ParametricType<SpaceBaseArea2AllotmentParameterLocation> for SpaceBaseArea2<X,Y> {
    type Value = Y;

    fn replace(&mut self, replace: &[(&SpaceBaseArea2AllotmentParameterLocation,Y)]) {
        let mut left_replace = vec![];
        let mut right_replace = vec![];
        for (location,value) in replace.iter() {
            match location {
                SpaceBaseArea2AllotmentParameterLocation::Left(x) => { left_replace.push((x,value.clone())); },
                SpaceBaseArea2AllotmentParameterLocation::Right(x) => { right_replace.push((x,value.clone())); },
            }
        }
        self.0.replace(&left_replace);
        self.1.replace(&right_replace);
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum HoleySpaceBaseArea2<X: Clone,Y: Clone> {
    Simple(SpaceBaseArea2<X,Y>),
    Parametric(SpaceBaseArea2<ParameterValue<X>,ParameterValue<Y>>)
}

impl<X: Clone + PartialOrd,Y: Clone> HoleySpaceBaseArea2<X,Y> {
    pub fn len(&self) -> usize {
        match self {
            HoleySpaceBaseArea2::Simple(x) => x.len(),
            HoleySpaceBaseArea2::Parametric(x) => x.len()
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> HoleySpaceBaseArea2<X,Y> {
        match self {
            HoleySpaceBaseArea2::Simple(x) => HoleySpaceBaseArea2::Simple(x.filter(filter)),
            HoleySpaceBaseArea2::Parametric(x) => HoleySpaceBaseArea2::Parametric(x.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        match self {
            HoleySpaceBaseArea2::Simple(x) =>
                x.make_base_filter(min_value,max_value),
            HoleySpaceBaseArea2::Parametric(x) =>
                x.make_base_filter(ParameterValue::Constant(min_value),ParameterValue::Constant(max_value))
        }
    }
}

pub enum SpaceBaseArea2ParameterLocation {
    Left(SpaceBase2ParameterLocation),
    Right(SpaceBase2ParameterLocation)
}

impl<X: Clone, Y: Clone> SpaceBaseArea2<ParameterValue<X>,ParameterValue<Y>> {
    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBaseArea2<X,Y> where F: Fn(SpaceBaseArea2ParameterLocation) -> L {
        let left = self.0.flatten(subs,|location| cb(SpaceBaseArea2ParameterLocation::Left(location)));
        let right = self.1.flatten(subs,|location| cb(SpaceBaseArea2ParameterLocation::Right(location)));
        SpaceBaseArea2(left,right,self.2)
    }
}

impl<X: Clone,Y: Clone> Flattenable for HoleySpaceBaseArea2<X,Y> {
    type Location = SpaceBaseArea2ParameterLocation;
    type Target = SpaceBaseArea2<X,Y>;

    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBaseArea2<X,Y> where F: Fn(Self::Location) -> L {
        match self {
            HoleySpaceBaseArea2::Simple(x) => x.clone(),
            HoleySpaceBaseArea2::Parametric(x) => x.flatten(subs,cb)
        }
    }
}

impl<X: Clone, Y: Clone> SpaceBaseArea2<X,Y> {
    pub fn new(top_left: PartialSpaceBase2<X,Y>, bottom_right: PartialSpaceBase2<X,Y>) -> Option<SpaceBaseArea2<X,Y>> {
        let mut compat = EachOrEveryGroupCompatible::new(None);
        top_left.compat(&mut compat);
        bottom_right.compat(&mut compat);
        let top_left = if let Some(b) = top_left.make(&compat) { b } else { return None; };
        let bottom_right = if let Some(b) = bottom_right.make(&compat) { b } else { return None; };
        let len = top_left.len();
        Some(SpaceBaseArea2(top_left,bottom_right,len))
    }

    pub fn len(&self) -> usize { self.2 }

    pub fn iter(&self) -> SpaceBaseArea2Iterator<X,Y> {
        SpaceBaseArea2Iterator {
            a: self.0.iter(),
            b: self.1.iter(),
        }
    }

    pub fn iter_other<'a,Z>(&self, other: &'a [Z]) -> impl Iterator<Item=&'a Z> {
        let len = self.len();
        other.iter().cycle().take(len)
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBaseArea2<X,Y> {
        SpaceBaseArea2(self.0.filter(filter),self.1.filter(filter),self.2)
    }

    pub fn top_left(&self) -> SpaceBase2<X,Y> { self.0.clone() }
    pub fn bottom_right(&self) -> SpaceBase2<X,Y> { self.1.clone() }
    pub fn bottom_left(&self) -> SpaceBase2<X,Y> { self.0.replace_normal(&self.1).unwrap() }
    pub fn top_right(&self) -> SpaceBase2<X,Y> { self.1.replace_normal(&self.0).unwrap() }
}

impl<X: Clone,Y: Clone> Clone for SpaceBaseArea2<X,Y> {
    fn clone(&self) -> Self {
        SpaceBaseArea2(self.0.clone(),self.1.clone(),self.2)
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>, Y: Clone> SpaceBaseArea2<X,Y> {
    pub fn middle_base(&self) -> SpaceBase2<X,Y> { self.0.middle_base(&self.1).unwrap() }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum HollowEdge<X> { Top(X), Left(X), Bottom(X), Right(X) }

impl<X: Clone + Add<Output=X> + Sub<Output=X>, Y: Clone> SpaceBaseArea2<X,Y> {
    pub fn new_from_sizes(points: &SpaceBase2<X,Y>, x_size: &[X], y_size: &[X]) -> SpaceBaseArea2<X,Y> {
        let mut far = points.clone();
        far.delta(x_size,y_size);
        SpaceBaseArea2(points.clone(),far,points.len())
    }

    pub fn hollow_edge(&self, edge: &HollowEdge<X>) -> SpaceBaseArea2<X,Y> {
        let mut out = self.clone();
        match edge {
            HollowEdge::Left(w) => {
                out.1.base = out.0.base.clone();
                out.1.tangent = out.0.tangent.map(|x| x.clone()+w.clone());
            },
            HollowEdge::Right(w) => {
                out.0.base = out.1.base.clone();
                out.1.tangent = out.0.tangent.map(|x| x.clone()+w.clone());
            },
            HollowEdge::Top(w) => {
                out.1.normal = out.0.normal.map(|x| x.clone()+w.clone());
            },
            HollowEdge::Bottom(w) => {
                out.1.normal = out.0.normal.map(|x| x.clone()-w.clone());
            }
        }
        out
    }
}

impl<X: Clone + PartialOrd, Y: Clone> SpaceBaseArea2<X,Y> {
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

pub struct SpaceBaseArea2Iterator<'a,X,Y> {
    a: SpaceBase2Iterator<'a,X,Y>,
    b: SpaceBase2Iterator<'a,X,Y>,
}

impl<'a,X,Y> Iterator for SpaceBaseArea2Iterator<'a,X,Y> {
    type Item = (SpaceBase2PointRef<'a,X,Y>,SpaceBase2PointRef<'a,X,Y>);

    fn next(&mut self) -> Option<Self::Item> {
        let (x,y) = (self.a.next(),self.b.next());
        if x.is_none() || y.is_none() { return None; }
        Some((x.unwrap(),y.unwrap()))
    }
}
