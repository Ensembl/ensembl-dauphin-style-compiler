use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentRequest, DataMessage, Patina, ShapeDemerge, Shape, util::{eachorevery::EachOrEveryFilter}, SpaceBaseArea, reactive::Observable, allotment::{transform_spacebasearea2, tree::allotmentbox::AllotmentBox, boxes::boxtraits::Transformable, transformers::transformers::{Transformer, TransformerVariety}, style::{pendingleaf::PendingLeaf, allotmentname::AllotmentNamePart, style::{LeafCommonStyle, LeafAllotmentStyle}}, stylespec::stylegroup::AllotmentStyleGroup}, EachOrEvery, CoordinateSystem};
use std::{hash::Hash, sync::Arc};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape<A> {
    area: SpaceBaseArea<f64,A>,
    patina: Patina,
    wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>
}

impl<A> RectangleShape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> RectangleShape<B> where F: Fn(&A) -> B {
        RectangleShape {
            area: self.area.map_allotments(cb),
            patina: self.patina.clone(),
            wobble: self.wobble.clone()
        }
    }

    pub fn len(&self) -> usize { self.area.len() }
    pub fn area(&self) -> &SpaceBaseArea<f64,A> { &self.area }
}

impl RectangleShape<PendingLeaf> {
    pub fn new2(area: SpaceBaseArea<f64,PendingLeaf>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Shape<PendingLeaf>,DataMessage> {
        let details = RectangleShape::new_details(area,patina.clone(),wobble.clone())?;
        Ok(Shape::SpaceBaseRect(details))
    }

    pub fn base_filter(&self, min_value: f64, max_value: f64) -> RectangleShape<PendingLeaf> {
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
    pub fn new(area: SpaceBaseArea<f64,AllotmentRequest>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let len = area.len();
        let mut out = vec![];
        let demerge = area.top_left().allotments().demerge(len,|x| { x.coord_system() });
        for (coord_system,mut filter) in demerge {
            if let Ok(details) = RectangleShape::new_details(area.filter(&filter),patina.clone(),wobble.clone()) {
                out.push(Shape::SpaceBaseRect(details.clone().filter(&filter)));
            }
        }
        Ok(out)
    }

    pub fn new_details(area: SpaceBaseArea<f64,A>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<RectangleShape<A>,DataMessage> {
        if !patina.compatible(area.len()) { return Err(DataMessage::LengthMismatch(format!("rectangle patina"))); }
        Ok(RectangleShape {
            area, patina, wobble
        })
    }

    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn wobble(&self) -> &Option<SpaceBaseArea<Observable<'static,f64>,()>> { &self.wobble }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> RectangleShape<A> {
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter),
            wobble: self.wobble.as_ref().map(|w| w.filter(filter))
        }
    }
}

impl RectangleShape<LeafCommonStyle> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,RectangleShape<LeafCommonStyle>)> where D: ShapeDemerge<X=T> {
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
        for (draw_group,mut filter) in demerge {
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
        for (variety,mut filter) in demerge {
            out.push((variety,self.filter(&filter)));
        }
        out
    }
}

impl RectangleShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<RectangleShape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(RectangleShape {
            area: self.area.fullmap_allotments_results(&cb,&cb)?,
            patina: self.patina.clone(),
            wobble: self.wobble.clone()
        })
    }
}

/*
impl RectangleShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> RectangleShape<()> {
        RectangleShape {
            area: transform_spacebasearea2(solution,&common.coord_system(),&self.area),
            patina: self.patina.clone(),
            wobble: self.wobble.clone()
        }
    }
}
*/

impl RectangleShape<Arc<dyn Transformer>> {
    pub fn make(&self, solution: &PuzzleSolution) -> Vec<RectangleShape<LeafCommonStyle>> {
        let mut out = vec![];
        for ((variety,coord_system),rectangles) in self.demerge_by_variety() {
            out.push(RectangleShape {
                area: variety.spacebasearea_transform(&coord_system,&self.area),
                patina: self.patina.clone(),
                wobble: self.wobble.clone()
            });
        }
        out
    }
}
