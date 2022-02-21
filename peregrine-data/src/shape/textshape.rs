use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Pen, Shape, ShapeDemerge, ShapeDetails, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, SpaceBase, allotment::{transform_spacebase2, core::allotment::Allotment}};
use std::hash::Hash;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape<A: Clone> {
    position: SpaceBase<f64,A>,
    pen: Pen,
    text: EachOrEvery<String>
}

impl<A: Clone> TextShape<A> {
    pub fn new_details(position: SpaceBase<f64,A>, pen: Pen, text: EachOrEvery<String>) -> Option<TextShape<A>> {
        if !text.compatible(position.len()) { return None; }
        Some(TextShape {
            position, pen, text
        })
    }

    pub fn new(position: SpaceBase<f64,AllotmentRequest>, depth: EachOrEvery<i8>, pen: Pen, text: EachOrEvery<String>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let mut out = vec![];
        let len = position.len();
        let demerge = position.demerge_by_allotment(|x| { x.coord_system() });
        let details = eoe_throw("new_text",TextShape::new_details(position,pen,text))?;
        for (coord_system,mut filter) in demerge {
            filter.set_size(len);
            out.push(Shape::new(
                eoe_throw("add_image",ShapeCommon::new(coord_system,depth.clone()))?,
                ShapeDetails::Text(details.clone().filter(&mut filter))
            ));
        }
        Ok(out)
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        self.position.make_base_filter(min,max)
    }

    pub(super) fn filter(&self, filter: &DataFilter) -> TextShape<A> {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            text: self.text.filter(filter)
        }
    }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common_in: &ShapeCommon, cat: &D) -> Vec<(T,ShapeCommon,TextShape<A>)> where D: ShapeDemerge<X=T> {
        let demerge = self.position.allotments().demerge(|a| cat.categorise(common_in.coord_system()));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            filter.set_size(self.position.len());
            let common = common_in.filter(&filter);
            out.push((draw_group,common,self.filter(&mut filter)));
        }
        out
    }
}

impl TextShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<TextShape<Allotment>,E> where F: Fn(&AllotmentRequest) -> Result<Allotment,E> {
        Ok(TextShape {
            position: self.position.map_allotments_results(cb)?,
            pen: self.pen.clone(),
            text: self.text.clone(),
        })
    }
}

impl TextShape<Allotment> {
    pub fn transform(&self, common: &ShapeCommon) -> TextShape<()> {
        TextShape {
            position: transform_spacebase2(&common.coord_system(),&self.position),
            pen: self.pen.clone(),
            text: self.text.clone()
        }
    }
}
