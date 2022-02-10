use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Flattenable, HoleySpaceBase, Pen, Shape, ShapeDemerge, ShapeDetails, SpaceBase, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, Allotment};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape<A: Clone> {
    position: HoleySpaceBase<f64>,
    pen: Pen,
    text: EachOrEvery<String>,
    allotments: EachOrEvery<A>
}

impl<A: Clone> TextShape<A> {
    pub fn new_details(position: HoleySpaceBase<f64>, allotments: EachOrEvery<A>, pen: Pen, text: EachOrEvery<String>) -> Option<TextShape<A>> {
        if !text.compatible(position.len()) { return None; }
        Some(TextShape {
            position, pen, text, allotments
        })
    }

    pub fn new(position: HoleySpaceBase<f64>, depth: EachOrEvery<i8>, pen: Pen, text: EachOrEvery<String>, allotments: EachOrEvery<AllotmentRequest>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let mut out = vec![];
        let len = position.len();
        let demerge = allotments.demerge(|x| { x.coord_system() });
        let details = eoe_throw("new_text",TextShape::new_details(position,allotments,pen,text))?;
        for (coord_system,mut filter) in demerge {
            filter.set_size(len);
            out.push(Shape::new(
                eoe_throw("add_image",ShapeCommon::new(filter.count(),coord_system,depth.clone()))?,
                ShapeDetails::Text(details.clone().filter(&mut filter))
            ));
        }
        Ok(out)
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn holey_position(&self) -> &HoleySpaceBase<f64> { &self.position }
    pub fn position(&self) -> SpaceBase<f64> { self.position.extract().0 }
    pub fn allotments(&self) -> &EachOrEvery<A> { &self.allotments }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.position.make_base_filter(min,max)
    }

    pub(super) fn filter(&self, filter: &DataFilter) -> TextShape<A> {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            text: self.text.filter(filter),
            allotments: self.allotments.filter(filter)
        }
    }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,TextShape<A>)> where D: ShapeDemerge<X=T> {
        let demerge = self.allotments.demerge(|a| cat.categorise(common_in.coord_system()));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(self.position.len());
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&mut filter)));
        }
        out
    }

    pub fn iter_allotments(&self, len: usize) -> impl Iterator<Item=&A> {
        self.allotments.iter(len).unwrap()
    }
}

impl TextShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<TextShape<Allotment>,E> where F: Fn(&AllotmentRequest) -> Result<Allotment,E> {
        Ok(TextShape {
            position: self.position.clone(),
            allotments: self.allotments.map_results(cb)?,
            pen: self.pen.clone(),
            text: self.text.clone(),
        })
    }
}