use std::{ops::{Add, Div, Sub}, sync::Arc};
use crate::util::ringarray::{ DataFilter };
use super::{parametric::{ParameterValue, ParametricType, Substitutions}, spacebase::{SpaceBase, SpaceBaseIterator, SpaceBaseParameterLocation, SpaceBasePointRef}};

#[derive(Debug)]
pub struct SpaceBaseArea<X>(SpaceBase<X>,SpaceBase<X>);

pub enum SpaceBaseAreaParameterLocation {
    Left(SpaceBaseParameterLocation),
    Right(SpaceBaseParameterLocation)
}

impl<X: Clone + Default> SpaceBaseArea<ParameterValue<X>> {
    fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBaseArea<X> where F: Fn(SpaceBaseAreaParameterLocation) -> L {
        let left = self.0.flatten(subs,|location| cb(SpaceBaseAreaParameterLocation::Left(location)));
        let right = self.1.flatten(subs,|location| cb(SpaceBaseAreaParameterLocation::Left(location)));
        SpaceBaseArea(left,right)
    }
}

impl<X: Clone> ParametricType for SpaceBaseArea<X> {
    type Location = SpaceBaseAreaParameterLocation;
    type Value = X;

    fn replace(&mut self, replace: &[(&Self::Location,X)]) {
        let mut left_replace = vec![];
        let mut right_replace = vec![];
        for (location,value) in replace.iter() {
            match location {
                SpaceBaseAreaParameterLocation::Left(x) => { left_replace.push((x,value.clone())); },
                SpaceBaseAreaParameterLocation::Right(x) => { right_replace.push((x,value.clone())); },
            }
        }
        self.0.replace(&left_replace);
        self.1.replace(&right_replace);
    }
}

pub enum HoleySpaceBaseArea {
    Simple(SpaceBaseArea<f64>),
    Parametric(SpaceBaseArea<ParameterValue<f64>>)
}

impl HoleySpaceBaseArea {
    pub(crate) fn flatten<F,L>(&self, subs: &mut Substitutions<L>, cb: F) -> SpaceBaseArea<f64> where F: Fn(SpaceBaseAreaParameterLocation) -> L {
        match self {
            HoleySpaceBaseArea::Simple(x) => x.clone(),
            HoleySpaceBaseArea::Parametric(x) => x.flatten(subs,cb)
        }
    }
}

impl<X: Clone> SpaceBaseArea<X> {
    pub fn new(top_left: SpaceBase<X>, bottom_right: SpaceBase<X>) -> SpaceBaseArea<X> {
        SpaceBaseArea(top_left,bottom_right)
    }

    pub fn len(&self) -> usize {  self.0.max_len.max(self.1.max_len) }

    pub fn iter(&self) -> SpaceBaseAreaIterator<X> {
        let len = self.0.max_len.max(self.1.max_len);
        SpaceBaseAreaIterator {
            a: self.0.iter_len(len),
            b: self.1.iter_len(len),
        }
    }

    pub fn iter_other<'a,Z>(&self, other: &'a [Z]) -> impl Iterator<Item=&'a Z> {
        let len = self.len();
        other.iter().cycle().take(len)
    }

    pub fn filter(&self, filter: &DataFilter) -> SpaceBaseArea<X> {
        SpaceBaseArea(self.0.filter(filter),self.1.filter(filter))
    }

    pub fn top_left(&self) -> SpaceBase<X> { self.0.clone() }
    pub fn bottom_right(&self) -> SpaceBase<X> { self.1.clone() }
    pub fn bottom_left(&self) -> SpaceBase<X> { self.0.replace_normal(&self.1) }
    pub fn top_right(&self) -> SpaceBase<X> { self.1.replace_normal(&self.0) }
}

impl<X> Clone for SpaceBaseArea<X> {
    fn clone(&self) -> Self {
        SpaceBaseArea(self.0.clone(),self.1.clone())
    }
}

impl<X: Clone + Add<Output=X> + Div<f64,Output=X>> SpaceBaseArea<X> {
    pub fn middle_base(&self) -> SpaceBase<X> { self.0.middle_base(&self.1) }
}

impl<X: Clone + Add<Output=X> + Sub<Output=X>> SpaceBaseArea<X> {
    pub fn new_from_sizes(points: &SpaceBase<X>, x_size: &[X], y_size: &[X]) -> SpaceBaseArea<X> {
        SpaceBaseArea(points.clone(),points.delta(x_size,y_size))
    }

    pub fn hollow(&self, w: X) -> (SpaceBaseArea<X>,SpaceBaseArea<X>,SpaceBaseArea<X>,SpaceBaseArea<X>) {
        let mut left = self.clone();
        left.1.base = left.0.base.clone();
        left.1.tangent = Arc::new(left.0.tangent.iter().map(|x| x.clone()+w.clone()).collect());
        /**/
        let mut right = self.clone();
        right.0.base = right.1.base.clone();
        right.0.tangent = Arc::new(right.1.tangent.iter().map(|x| x.clone()-w.clone()).collect());
        /**/
        let mut top = self.clone();
        top.1.normal = Arc::new(top.0.normal.iter().map(|x| x.clone()+w.clone()).collect());
        /**/
        let mut bottom = self.clone();
        bottom.0.normal = Arc::new(bottom.1.normal.iter().map(|x| x.clone()-w.clone()).collect());
        /**/
        (left,right,top,bottom)
    }
}

impl<X: Clone + PartialOrd> SpaceBaseArea<X> {
    pub fn make_base_filter(&self, min_value: X, max_value: X) -> DataFilter {
        let top_left = DataFilter::new(&mut self.0.base.iter(),|base| {
            *base <= max_value
        });
        let bottom_right = DataFilter::new(&mut self.1.base.iter(),|base| {
            *base >= min_value
        });
        top_left.and(&bottom_right)
    }
}

pub struct SpaceBaseAreaIterator<'a,X> {
    a: SpaceBaseIterator<'a,X>,
    b: SpaceBaseIterator<'a,X>,
}

impl<'a,X> Iterator for SpaceBaseAreaIterator<'a,X> {
    type Item = (SpaceBasePointRef<'a,X>,SpaceBasePointRef<'a,X>);

    fn next(&mut self) -> Option<Self::Item> {
        let (x,y) = (self.a.next(),self.b.next());
        if x.is_none() || y.is_none() { return None; }
        Some((x.unwrap(),y.unwrap()))
    }
}
