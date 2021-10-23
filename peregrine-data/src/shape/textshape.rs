use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBase, Pen, Shape, ShapeDemerge, ShapeDetails, SpaceBase, shape::shape::ShapeCommon, util::eachorevery::eoe_throw};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape {
    position: HoleySpaceBase,
    pen: Pen,
    text: EachOrEvery<String>
}

impl TextShape {
    pub fn new_details(position: HoleySpaceBase, pen: Pen, text: EachOrEvery<String>) -> Option<TextShape> {
        if !text.compatible(position.len()) { return None; }
        Some(TextShape {
            position, pen, text
        })
    }

    pub fn new(position: HoleySpaceBase, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape>,DataMessage> {
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
    pub fn holey_position(&self) -> &HoleySpaceBase { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }

    fn filter(&self, filter: &DataFilter) -> TextShape {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            text: self.text.filter(filter)
        }
    }


    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    pub fn filter_by_minmax(&self, common: &mut ShapeCommon, min: f64, max: f64) -> TextShape {
        let mut filter = self.position.make_base_filter(min,max);
        filter.set_size(self.position.len());
        *common = common.filter(&filter);
        self.filter(&filter)
    }

    pub fn filter_by_allotment<F>(&self, common: &mut ShapeCommon, cb: F)  -> TextShape where F: Fn(&AllotmentRequest) -> bool {
        let mut filter = common.allotments().new_filter(self.position.len(),cb);
        filter.set_size(self.position.len());
        *common = common.filter(&filter);
        self.filter(&filter)
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,TextShape)> where D: ShapeDemerge<X=T> {
        let demerge = common_in.allotments().demerge(|a| cat.categorise(a));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(self.position.len());
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&mut filter)));
        }
        out
    }
}
