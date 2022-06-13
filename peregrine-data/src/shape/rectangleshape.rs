use peregrine_toolkit::eachorevery::EachOrEveryFilter;
use crate::{DataMessage, Patina, ShapeDemerge, Shape, SpaceBaseArea, reactive::Observable, allotment::{transformers::transformers::{Transformer, TransformerVariety}, style::{style::{LeafStyle}}}, CoordinateSystem, LeafRequest};
use std::{hash::Hash, sync::Arc};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape<A> {
    area: SpaceBaseArea<f64,A>,
    patina: Patina,
    wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>
}

impl<A> RectangleShape<A> {
    pub fn new_details(area: SpaceBaseArea<f64,A>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<RectangleShape<A>,DataMessage> {
        if !patina.compatible(area.len()) { return Err(DataMessage::LengthMismatch(format!("rectangle patina"))); }
        Ok(RectangleShape {
            area, patina, wobble
        })
    }

    pub fn map_new_allotment<F,B>(&self, cb: F) -> RectangleShape<B> where F: FnMut(&A) -> B {
        RectangleShape {
            area: self.area.map_allotments(cb),
            patina: self.patina.clone(),
            wobble: self.wobble.clone()
        }
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> RectangleShape<A> {
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter),
            wobble: self.wobble.as_ref().map(|w| w.filter(filter))
        }
    }

    pub fn len(&self) -> usize { self.area.len() }
    pub fn area(&self) -> &SpaceBaseArea<f64,A> { &self.area }
}

impl RectangleShape<LeafRequest> {
    pub fn new2(area: SpaceBaseArea<f64,LeafRequest>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = RectangleShape::new_details(area,patina.clone(),wobble.clone())?;
        Ok(Shape::SpaceBaseRect(details))
    }

    pub fn base_filter(&self, min_value: f64, max_value: f64) -> RectangleShape<LeafRequest> {
        let non_tracking = self.area.top_left().allotments().make_filter(self.area.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.area.make_base_filter(min_value,max_value);
        self.filter(&filter.or(&non_tracking))
    }
}

impl<A> Clone for RectangleShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { area: self.area.clone(), patina: self.patina.clone(), wobble: self.wobble.clone() }
    }
}

impl<A: Clone> RectangleShape<A> {
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn wobble(&self) -> &Option<SpaceBaseArea<Observable<'static,f64>,()>> { &self.wobble }
}

impl RectangleShape<LeafStyle> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,RectangleShape<LeafStyle>)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = self.area.top_left().allotments().zip(&colours,|x,y| (x.clone(),y.clone()));
                allotments_and_colours.demerge(self.area.len(),|(a,c)| 
                    cat.categorise_with_colour(&a.coord_system,drawn_type,c)
                )
            },
            _ => {
                self.area.top_left().allotments().demerge(self.area.len(),|a| cat.categorise(&a.coord_system))
            }
        };
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            out.push((draw_group,self.filter(&filter)));
        }
        out
    }
}

impl RectangleShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<((TransformerVariety,CoordinateSystem),RectangleShape<Arc<dyn Transformer>>)> {
        let demerge = self.area.top_left().allotments().demerge(self.area.len(),|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,filter) in demerge {
            out.push((variety,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self) -> Vec<RectangleShape<LeafStyle>> {
        let mut out = vec![];
        for ((variety,coord_system),rectangles) in self.demerge_by_variety() {
            out.push(RectangleShape {
                area: variety.spacebasearea_transform(&coord_system,&rectangles.area),
                patina: rectangles.patina.clone(),
                wobble: rectangles.wobble.clone()
            });
        }
        out
    }
}
