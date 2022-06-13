use std::hash::Hash;
use std::sync::Arc;
use peregrine_toolkit::eachorevery::EachOrEveryFilter;

use crate::{SpaceBaseArea, DataMessage, LeafRequest, Shape, allotment::transformers::transformers::{Transformer, TransformerVariety}, CoordinateSystem, LeafStyle, ShapeDemerge};

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
        let non_tracking = self.0.top_left().allotments().make_filter(self.0.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.0.make_base_filter(min_value,max_value);
        self.filter(&filter.or(&non_tracking))
    }
}

impl<A> Clone for EmptyShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl EmptyShape<LeafStyle> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,EmptyShape<LeafStyle>)> where D: ShapeDemerge<X=T> {
        let demerge = self.0.top_left().allotments().demerge(self.0.len(),|a| cat.categorise(&a.coord_system));
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            out.push((draw_group,self.filter(&filter)));
        }
        out
    }
}

impl EmptyShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<((TransformerVariety,CoordinateSystem),EmptyShape<Arc<dyn Transformer>>)> {
        let demerge = self.0.top_left().allotments().demerge(self.0.len(),|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,filter) in demerge {
            out.push((variety,self.filter(&filter)));
        }
        out
    }
}

impl EmptyShape<Arc<dyn Transformer>> {
    pub fn make(&self) -> Vec<EmptyShape<LeafStyle>> {
        let mut out = vec![];
        for ((variety,coord_system),rectangles) in self.demerge_by_variety() {
            out.push(EmptyShape(variety.spacebasearea_transform(&coord_system,&rectangles.0)));
        }
        out
    }
}
