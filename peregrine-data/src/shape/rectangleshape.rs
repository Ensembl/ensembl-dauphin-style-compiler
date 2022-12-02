use peregrine_toolkit::eachorevery::EachOrEveryFilter;
use crate::{DataMessage, Patina, ShapeDemerge, Shape, SpaceBaseArea, reactive::Observable, allotment::{leafs::anchored::AnchoredLeaf}, LeafRequest, CoordinateSystem, AuxLeaf};
use std::{hash::Hash};

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

    pub(crate) fn len(&self) -> usize { self.area.len() }
    pub fn area(&self) -> &SpaceBaseArea<f64,A> { &self.area }
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn wobble(&self) -> &Option<SpaceBaseArea<Observable<'static,f64>,()>> { &self.wobble }
}

impl RectangleShape<LeafRequest> {
    pub fn new(area: SpaceBaseArea<f64,LeafRequest>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = RectangleShape::new_details(area,patina.clone(),wobble.clone())?;
        Ok(Shape::SpaceBaseRect(details))
    }

    pub fn base_filter(&self, min_value: f64, max_value: f64) -> RectangleShape<LeafRequest> {
        let non_tracking = self.area.top_left().allotments().make_filter(self.area.len(),|a| !a.leaf_style().aux.coord_system.is_tracking());
        let filter = self.area.make_base_filter(min_value,max_value);
        self.filter(&filter.or(&non_tracking))
    }
}

impl<A> Clone for RectangleShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { area: self.area.clone(), patina: self.patina.clone(), wobble: self.wobble.clone() }
    }
}

impl RectangleShape<AuxLeaf> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,RectangleShape<AuxLeaf>)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = self.area.top_left().allotments().zip(&colours,|x,y| (x.clone(),y.clone()));
                allotments_and_colours.demerge(self.area.len(),|(a,c)| 
                    cat.categorise_with_colour(&a.coord_system,a.depth,drawn_type,c)
                )
            },
            _ => {
                self.area.top_left().allotments().demerge(self.area.len(),|a| cat.categorise(&a.coord_system,a.depth))
            }
        };
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            if filter.count() > 0 {
                out.push((draw_group,self.filter(&filter)));
            }
        }
        out
    }
}

impl RectangleShape<AnchoredLeaf> {
    fn demerge_by_variety(&self) -> Vec<(CoordinateSystem,RectangleShape<AnchoredLeaf>)> {
        let demerge = self.area.top_left().allotments().demerge(self.area.len(),|x| {
            x.coordinate_system().clone()
        });
        let mut out = vec![];
        for (coordinate_system,filter) in demerge {
            out.push((coordinate_system,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self) -> Vec<RectangleShape<AuxLeaf>> {
        let mut out = vec![];
        for (coord_system,rectangles) in self.demerge_by_variety() {
            out.push(RectangleShape {
                area: rectangles.area.spacebasearea_transform(&coord_system),
                patina: rectangles.patina.clone(),
                wobble: rectangles.wobble.clone()
            });
        }
        out
    }
}
