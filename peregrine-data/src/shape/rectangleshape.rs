use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBaseArea, Patina, Shape, ShapeDemerge, ShapeDetails, SpaceBaseArea, shape::shape::ShapeCommon, util::eachorevery::eoe_throw};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape {
    area: HoleySpaceBaseArea,
    patina: Patina
}

impl RectangleShape {
    pub fn new_details(area: HoleySpaceBaseArea, patina: Patina) -> Option<RectangleShape> {
        if !patina.compatible(area.len()) { return None; }
        Some(RectangleShape {
            area, patina
        })
    }

    pub fn new(area: HoleySpaceBaseArea, patina: Patina, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape>,DataMessage> {
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
    pub fn holey_area(&self) -> &HoleySpaceBaseArea { &self.area }
    pub fn area(&self) -> SpaceBaseArea<f64> { self.area.extract().0 }

    fn filter(&self, filter: &DataFilter) -> RectangleShape {
        RectangleShape {
            area: self.area.filter(filter),
            patina: self.patina.filter(filter)
        }
    }

    pub fn filter_by_minmax(&self, common: &mut ShapeCommon, min: f64, max: f64) -> RectangleShape {
        let mut filter = self.area.make_base_filter(min,max);
        filter.set_size(self.area.len());
        *common = common.filter(&filter);
        let x = self.filter(&filter);
        x
    }

    pub fn filter_by_allotment<F>(&self, common: &mut ShapeCommon, cb: F)  -> RectangleShape where F: Fn(&AllotmentRequest) -> bool {
        let mut filter = common.allotments().new_filter(self.area.len(),cb);
        filter.set_size(self.area.len());
        *common = common.filter(&filter);
        self.filter(&filter)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,RectangleShape)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = common_in.allotments().merge(&colours).unwrap();
                allotments_and_colours.demerge(|(a,c)| 
                    cat.categorise_with_colour(a,drawn_type,c)
                )
            },
            _ => {
                common_in.allotments().demerge(|a| cat.categorise(a))
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
