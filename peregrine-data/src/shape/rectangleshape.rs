use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentRequest, DataMessage, Patina, Shape, ShapeDemerge, ShapeDetails, shape::shape::ShapeCommon, util::{eachorevery::EachOrEveryFilter}, SpaceBaseArea, reactive::Observable, allotment::{transform_spacebasearea2, tree::allotmentbox::AllotmentBox, boxes::boxtraits::Transformable, transformers::transformers::{Transformer, TransformerVariety}, style::{pendingleaf::PendingLeaf, allotmentname::AllotmentNamePart}, stylespec::stylegroup::AllotmentStyleGroup}, EachOrEvery};
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
}

// XXX pass in coord-system
impl RectangleShape<PendingLeaf> {
    /*
    pub fn new2(area: SpaceBaseArea<f64,PendingLeaf>, style: &AllotmentStyleGroup, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Shape<PendingLeaf> {
        let len = area.len();
        let mut out = vec![];
        let demerge = area.top_left().allotments().demerge(|leaf| {
            style.get_leaf(&AllotmentNamePart::new(leaf.name().clone())).leaf.top_style.coord_system
        });
        for (coord_system,mut filter) in demerge {
            filter.set_size(len);
            let details = eoe_throw("add_rectangles",RectangleShape::new_details(area.filter(&filter),patina.clone(),wobble.clone()))?;
            out.push(Shape::new(
                eoe_throw("add_rectangles",ShapeCommon::new(coord_system, depth.filter(&filter)))?,
                ShapeDetails::SpaceBaseRect(details.clone().filter(&filter))
            ));
        }
        Ok(out)
    }
    */
}

impl<A> Clone for RectangleShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { area: self.area.clone(), patina: self.patina.clone(), wobble: self.wobble.clone() }
    }

}

impl<A: Clone> RectangleShape<A> {
    pub fn new(area: SpaceBaseArea<f64,AllotmentRequest>, depth: EachOrEvery<i8>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let len = area.len();
        let mut out = vec![];
        let demerge = area.top_left().allotments().demerge(len,|x| { x.coord_system() });
        for (coord_system,mut filter) in demerge {
            if let Some(details) = RectangleShape::new_details(area.filter(&filter),patina.clone(),wobble.clone()) {
                out.push(Shape::new(
                    ShapeCommon::new(coord_system, depth.filter(&filter)),
                    ShapeDetails::SpaceBaseRect(details.clone().filter(&filter))
                ));
            }
        }
        Ok(out)
    }

    pub fn new_details(area: SpaceBaseArea<f64,A>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Option<RectangleShape<A>> {
        if !patina.compatible(area.len()) { return None; }
        Some(RectangleShape {
            area, patina, wobble
        })
    }

    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn wobble(&self) -> &Option<SpaceBaseArea<Observable<'static,f64>,()>> { &self.wobble }
    pub fn area(&self) -> &SpaceBaseArea<f64,A> { &self.area }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> RectangleShape<A> {
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter),
            wobble: self.wobble.as_ref().map(|w| w.filter(filter))
        }
    }

    pub fn make_base_filter(&self, min: f64, max: f64) -> EachOrEveryFilter {
        self.area.make_base_filter(min,max)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,RectangleShape<A>)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = self.area.top_left().allotments().zip(&colours,|x,y| (x.clone(),y.clone()));
                allotments_and_colours.demerge(self.area.len(),|(_,c)| 
                    cat.categorise_with_colour(common_in.coord_system(),drawn_type,c)
                )
            },
            _ => {
                self.area.top_left().allotments().demerge(self.area.len(),|_| cat.categorise(common_in.coord_system()))
            }
        };
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&filter)));
        }
        out
    }
}

impl RectangleShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<(TransformerVariety,RectangleShape<Arc<dyn Transformer>>)> {
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

impl RectangleShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> RectangleShape<()> {
        RectangleShape {
            area: transform_spacebasearea2(solution,&common.coord_system(),&self.area),
            patina: self.patina.clone(),
            wobble: self.wobble.clone()
        }
    }
}

impl RectangleShape<Arc<dyn Transformer>> {
    pub fn make(&self, solution: &PuzzleSolution, common: &ShapeCommon) -> Vec<RectangleShape<()>> {
        let mut out = vec![];
        for (variety,rectangles) in self.demerge_by_variety() {
            out.push(RectangleShape {
                area: variety.spacebasearea_transform(&common.coord_system(),&self.area),
                patina: self.patina.clone(),
                wobble: self.wobble.clone()
            });
        }
        out
    }
}
