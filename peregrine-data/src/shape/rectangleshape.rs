use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBaseArea, Patina, Shape, ShapeDemerge, ShapeDetails, SpaceBaseArea, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, Allotment};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape {
    area: HoleySpaceBaseArea<f64>,
    patina: Patina
}

impl RectangleShape {
    pub fn new_details(area: HoleySpaceBaseArea<f64>, patina: Patina) -> Option<RectangleShape> {
        if !patina.compatible(area.len()) { return None; }
        Some(RectangleShape {
            area, patina
        })
    }

    pub fn new(area: HoleySpaceBaseArea<f64>, patina: Patina, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let len = area.len();
        let mut out = vec![];
        let details = eoe_throw("add_rectangles",RectangleShape::new_details(area,patina))?;
        for (coord_system,mut filter) in allotments.demerge(|x| { x.coord_system() }) {
            filter.set_size(len);
            out.push(Shape::new(
                eoe_throw("add_rectangles",ShapeCommon::new(filter.count(),coord_system, allotments.filter(&filter)))?,
                ShapeDetails::SpaceBaseRect(details.clone().filter(&filter))
            ));
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.area.len() }
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn holey_area(&self) -> &HoleySpaceBaseArea<f64> { &self.area }
    pub fn area(&self) -> SpaceBaseArea<f64> { self.area.extract().0 }

    pub(super) fn filter(&self, filter: &DataFilter) -> RectangleShape {
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter)
        }
    }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.area.make_base_filter(min,max)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon<Allotment>, cat: &D) -> Vec<(T,ShapeCommon<Allotment>,RectangleShape)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = common_in.allotments().merge(&colours).unwrap();
                allotments_and_colours.demerge(|(a,c)| 
                    cat.categorise_with_colour(common_in.coord_system(),drawn_type,c)
                )
            },
            _ => {
                common_in.allotments().demerge(|a| cat.categorise(common_in.coord_system()))
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
