use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, Patina, Shape, ShapeDemerge, ShapeDetails, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, Allotment, HoleySpaceBaseArea, SpaceBaseArea};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape<A: Clone> {
    area: HoleySpaceBaseArea<f64,A>,
    patina: Patina
}

impl<A: Clone> RectangleShape<A> {
    pub fn new_details(area: HoleySpaceBaseArea<f64,A>, patina: Patina) -> Option<RectangleShape<A>> {
        if !patina.compatible(area.len()) { return None; }
        Some(RectangleShape {
            area, patina
        })
    }

    pub fn new(area: HoleySpaceBaseArea<f64,AllotmentRequest>, depth: EachOrEvery<i8>, patina: Patina) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let len = area.len();
        let mut out = vec![];
        let demerge = area.demerge_by_allotment(|x| { x.coord_system() });
        for (coord_system,mut filter) in demerge {
            filter.set_size(len);
            let details = eoe_throw("add_rectangles",RectangleShape::new_details(area.filter(&filter),patina.clone()))?;
            out.push(Shape::new(
                eoe_throw("add_rectangles",ShapeCommon::new(coord_system, depth.filter(&filter)))?,
                ShapeDetails::SpaceBaseRect(details.clone().filter(&filter))
            ));
        }
        Ok(out)
    }

    pub fn len(&self) -> usize { self.area.len() }
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn holey_area(&self) -> &HoleySpaceBaseArea<f64,A> { &self.area }
    pub fn area(&self) -> SpaceBaseArea<f64,A> { self.area.extract().0 }

    pub(super) fn filter(&self, filter: &DataFilter) -> RectangleShape<A> {
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter)
        }
    }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.area.make_base_filter(min,max)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,RectangleShape<A>)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = self.area.allotments().merge(&colours).unwrap();
                allotments_and_colours.demerge(|(a,c)| 
                    cat.categorise_with_colour(common_in.coord_system(),drawn_type,c)
                )
            },
            _ => {
                self.area.allotments().demerge(|a| cat.categorise(common_in.coord_system()))
            }
        };
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(self.area.len());
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&filter)));
        }
        out
    }
}

impl RectangleShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<RectangleShape<Allotment>,E> where F: Fn(&AllotmentRequest) -> Result<Allotment,E> {
        Ok(RectangleShape {
            area: self.area.map_allotments_results(&cb,&cb)?,
            patina: self.patina.clone()
        })
    }
}
