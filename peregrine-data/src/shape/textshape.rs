use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBase, Pen, Shape, ShapeDemerge, ShapeDetails, SpaceBase, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, Allotment};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape {
    position: HoleySpaceBase<f64>,
    pen: Pen,
    text: EachOrEvery<String>
}

impl TextShape {
    pub fn new_details(position: HoleySpaceBase<f64>, pen: Pen, text: EachOrEvery<String>) -> Option<TextShape> {
        if !text.compatible(position.len()) { return None; }
        Some(TextShape {
            position, pen, text
        })
    }

    pub fn new(position: HoleySpaceBase<f64>, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let mut out = vec![];
        let len = position.len();
        let details = eoe_throw("new_text",TextShape::new_details(position,pen,text))?;
        for (coord_system,mut filter) in allotments.demerge(|x| { x.coord_system() }) {
            filter.set_size(len);
            out.push(Shape::new(
                eoe_throw("add_image",ShapeCommon::new(filter.count(),coord_system,allotments.filter(&filter)))?,
                ShapeDetails::Text(details.clone().filter(&mut filter))
            ));
        }
        Ok(out)        
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn holey_position(&self) -> &HoleySpaceBase<f64> { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.position.make_base_filter(min,max)
    }

    pub(super) fn filter(&self, filter: &DataFilter) -> TextShape {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            text: self.text.filter(filter)
        }
    }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon<Allotment>, cat: &D) -> Vec<(T,ShapeCommon<Allotment>,TextShape)> where D: ShapeDemerge<X=T> {
        let demerge = common_in.allotments().demerge(|a| cat.categorise(common_in.coord_system()));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(self.position.len());
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&mut filter)));
        }
        out
    }
}
