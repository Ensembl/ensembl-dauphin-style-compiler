use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{AllotmentRequest, DataMessage, Pen, ShapeDemerge, Shape, util::{eachorevery::EachOrEveryFilter}, SpaceBase, allotment::{transform_spacebase2, tree::allotmentbox::AllotmentBox, transformers::transformers::{Transformer, TransformerVariety}, style::{pendingleaf::PendingLeaf, style::LeafCommonStyle}, core::coordsystem}, EachOrEvery, CoordinateSystem};
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

    pub fn len(&self) -> usize { self.position.len() }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }
}

impl TextShape<PendingLeaf> {
    pub fn new2(position: SpaceBase<f64,PendingLeaf>, pen: Pen, text: EachOrEvery<String>) -> Result<Shape<PendingLeaf>,DataMessage> {
        let details = TextShape::new_details(position,pen,text.clone())?;
        Ok(Shape::Text(details))
    }
}

impl<A> Clone for TextShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { position: self.position.clone(), pen: self.pen.clone(), text: self.text.clone() }
    }
}

impl<A: Clone> TextShape<A> {
    pub fn new_details(position: SpaceBase<f64,A>, pen: Pen, text: EachOrEvery<String>) -> Result<TextShape<A>,DataMessage> {
        if !text.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("text patina"))); }
        Ok(TextShape {
            position, pen, text
        })
    }

    pub fn new(position: SpaceBase<f64,AllotmentRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        if !text.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("text patina"))); }
        let mut out = vec![];
        let len = position.len();
        let demerge = position.demerge_by_allotment(|x| { x.coord_system() });
        if let Ok(details) = TextShape::new_details(position,pen,text) {
            for (coord_system,mut filter) in demerge {
                out.push(Shape::Text(details.clone().filter(&mut filter)));
            }
        }
        Ok(out)
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> TextShape<A> {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            text: self.text.filter(filter)
        }
    }
}

impl TextShape<PendingLeaf> {
    pub fn base_filter(&self, min: f64, max: f64) -> TextShape<PendingLeaf> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.position.make_base_filter(min,max);
        self.filter(&filter.or(&non_tracking))
    }
}

impl TextShape<LeafCommonStyle> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self,  cat: &D) -> Vec<(T,TextShape<LeafCommonStyle>)> where D: ShapeDemerge<X=T> {
        let demerge = self.position.allotments().demerge(self.position.len(),|a| cat.categorise(&a.coord_system));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            out.push((draw_group,self.filter(&mut filter)));
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

/*
impl TextShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> TextShape<()> {
        TextShape {
            position: transform_spacebase2(solution,&common.coord_system(),&self.position),
            pen: self.pen.clone(),
            text: self.text.clone()
        }
    }
}
*/

impl TextShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<((TransformerVariety,CoordinateSystem),TextShape<Arc<dyn Transformer>>)> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,mut filter) in demerge {
            out.push((variety,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self, solution: &PuzzleSolution) -> Vec<TextShape<LeafCommonStyle>> {
        let mut out = vec![];
        for ((variety,coord_system),texts) in self.demerge_by_variety() {
            out.push(TextShape {
                position: variety.spacebase_transform(&coord_system,&texts.position),
                text: self.text.clone(),
                pen: self.pen.clone()
            });
        }
        out
    }
}

