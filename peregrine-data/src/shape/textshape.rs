use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentRequest, DataFilter, DataMessage, EachOrEvery, Pen, Shape, ShapeDemerge, ShapeDetails, shape::shape::ShapeCommon, util::eachorevery::eoe_throw, SpaceBase, allotment::{transform_spacebase2, tree::allotmentbox::AllotmentBox, transformers::transformers::{Transformer, TransformerVariety}}};
use std::{hash::Hash, sync::Arc};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape<A> {
    position: SpaceBase<f64,A>,
    pen: Pen,
    text: EachOrEvery<String>
}

impl<A> TextShape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> TextShape<B> where F: Fn(&A) -> B {
        TextShape {
            position: self.position.map_allotments(cb),
            pen: self.pen.clone(),
            text: self.text.clone()
        }
    }
}

impl<A> Clone for TextShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { position: self.position.clone(), pen: self.pen.clone(), text: self.text.clone() }
    }
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
    pub fn allot<F,E>(self, cb: F) -> Result<TextShape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(TextShape {
            position: self.position.fullmap_allotments_results(cb)?,
            pen: self.pen.clone(),
            text: self.text.clone(),
        })
    }
}

impl TextShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> TextShape<()> {
        TextShape {
            position: transform_spacebase2(solution,&common.coord_system(),&self.position),
            pen: self.pen.clone(),
            text: self.text.clone()
        }
    }
}

impl TextShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<(TransformerVariety,TextShape<Arc<dyn Transformer>>)> {
        let demerge = self.position.allotments().demerge(|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,mut filter) in demerge {
            filter.set_size(self.position.len());
            out.push((variety,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self, solution: &PuzzleSolution, common: &ShapeCommon) -> Vec<TextShape<()>> {
        let mut out = vec![];
        for (variety,rectangles) in self.demerge_by_variety() {
            out.push(TextShape {
                position: variety.spacebase_transform(&common.coord_system(),&self.position),
                text: self.text.clone(),
                pen: self.pen.clone()
            });
        }
        out
    }
}

