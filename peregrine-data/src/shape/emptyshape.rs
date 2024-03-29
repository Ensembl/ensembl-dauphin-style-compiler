use std::hash::Hash;
use eachorevery::EachOrEveryFilter;

use crate::{SpaceBaseArea, DataMessage, LeafRequest, Shape, allotment::{leafs::anchored::AnchoredLeaf}, ShapeDemerge, CoordinateSystem, AuxLeaf};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct EmptyShape<A>(SpaceBaseArea<f64,A>);

impl<A> EmptyShape<A> {
    pub fn new_details(area: SpaceBaseArea<f64,A>) -> Result<EmptyShape<A>,DataMessage> {
        Ok(EmptyShape(area))
    }

    pub fn map_new_allotment<F,B>(&self, cb: F) -> EmptyShape<B> where F: FnMut(&A) -> B {
        EmptyShape(self.0.map_allotments(cb))
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> EmptyShape<A> {
        EmptyShape(self.0.filter(filter))
    }

    pub fn len(&self) -> usize { self.0.len() }
    pub fn area(&self) -> &SpaceBaseArea<f64,A> { &self.0 }
}

impl EmptyShape<LeafRequest> {
    pub fn new(area: SpaceBaseArea<f64,LeafRequest>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = EmptyShape::new_details(area)?;
        Ok(Shape::Empty(details))
    }

    pub fn base_filter(&self, min_value: f64, max_value: f64) -> EmptyShape<LeafRequest> {
        let non_tracking = self.0.top_left().allotments().make_filter(self.0.len(),|a| !a.leaf_style().aux.coord_system.is_tracking());
        let filter = self.0.make_base_filter(min_value,max_value);
        self.filter(&filter.or(&non_tracking))
    }
}

impl<A> Clone for EmptyShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl EmptyShape<AuxLeaf> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,EmptyShape<AuxLeaf>)> where D: ShapeDemerge<X=T> {
        let demerge = self.0.top_left().allotments().demerge(self.0.len(),|a| cat.categorise(&a.coord_system,a.depth));
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            out.push((draw_group,self.filter(&filter)));
        }
        out
    }
}

impl EmptyShape<AnchoredLeaf> {
    fn demerge_by_variety(&self) -> Vec<(CoordinateSystem,EmptyShape<AnchoredLeaf>)> {
        let demerge = self.0.top_left().allotments().demerge(self.0.len(),|x| {
            x.coordinate_system().clone()
        });
        let mut out = vec![];
        for (coord,filter) in demerge {
            out.push((coord,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self) -> Vec<EmptyShape<AuxLeaf>> {
        let mut out = vec![];
        for (coord_system,rectangles) in self.demerge_by_variety() {
            out.push(EmptyShape(rectangles.0.spacebasearea_transform(&coord_system)));
        }
        out
    }
}
