use peregrine_toolkit::{eachorevery::{EachOrEveryFilter, EachOrEvery}, log};

use crate::{DataMessage, Pen, ShapeDemerge, Shape, SpaceBase, allotment::{transformers::{transformers::{Transformer, TransformerVariety}}, style::{style::LeafStyle}, util::rangeused::RangeUsed, core::allotmentname::AllotmentName}, CoordinateSystem, LeafRequest, SpaceBaseArea, PartialSpaceBase};
use std::{hash::Hash, sync::Arc};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape<A> {
    position: SpaceBase<f64,A>,
    run: Option<SpaceBase<f64,()>>,
    pen: Pen,
    text: EachOrEvery<String>
}

impl<A> TextShape<A> {
    pub(super) fn map_new_allotment<F,B>(&self, cb: F) -> TextShape<B> where F: FnMut(&A) -> B {
        TextShape {
            position: self.position.map_allotments(cb),
            run: self.run.clone(),
            pen: self.pen.clone(),
            text: self.text.clone()
        }
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn position(&self) -> &SpaceBase<f64,A> { &self.position }
    pub fn run(&self) -> Option<&SpaceBase<f64,()>> { self.run.as_ref() }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    fn new_details(position: SpaceBase<f64,A>, run: Option<SpaceBase<f64,()>>, pen: Pen, text: EachOrEvery<String>) -> Result<TextShape<A>,DataMessage> {
        if !text.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("text patina"))); }
        Ok(TextShape {
            position, pen, run, text
        })
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> TextShape<A> {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            run: self.run.as_ref().map(|x| x.filter(filter)),
            text: self.text.filter(filter)
        }
    }
}

impl TextShape<LeafRequest> {
    pub fn new(position: SpaceBase<f64,LeafRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = TextShape::new_details(position,None,pen,text.clone())?;
        Ok(Shape::Text(details))
    }

    pub fn new_running(position: SpaceBase<f64,LeafRequest>, run: SpaceBase<f64,()>, pen: Pen, text: EachOrEvery<String>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = TextShape::new_details(position,Some(run),pen,text.clone())?;
        Ok(Shape::Text(details))
    }

    pub(super) fn register_space(&self) {
        let size = self.pen().geometry().size_in_webgl();
        let major = self.position().iter();
        let minor = self.run().map(|x| x.iter());
        if let Some(minor) = minor {
            /* Running */
            for ((top_left,bottom_right),text) in major.zip(minor).zip(self.iter_texts()) {
                top_left.allotment.update_drawing_info(|allotment| {
                    allotment.merge_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base+1.));
                    allotment.merge_pixel_range(&RangeUsed::Part(*top_left.tangent,(top_left.tangent+size*text.len() as f64).max(*bottom_right.tangent))); // Not ideal: assume square
                    allotment.merge_max_y((*top_left.normal + size).ceil());
                });
            }    
        } else {
            /* Normal */
            for (position,text) in major.zip(self.iter_texts()) {
                position.allotment.update_drawing_info(|allotment| {
                    allotment.merge_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                    allotment.merge_pixel_range(&RangeUsed::Part(*position.tangent,position.tangent+size*text.len() as f64)); // Not ideal: assume square
                    allotment.merge_max_y((*position.normal + size).ceil());
                });
            }    
        }
    }
}

impl<A> Clone for TextShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self {
            position: self.position.clone(),
            pen: self.pen.clone(),
            run: self.run.clone(),
            text: self.text.clone()
        }
    }
}

impl TextShape<LeafRequest> {
    fn make_base_filter(&self, min: f64,max: f64) -> EachOrEveryFilter {
        if let Some(run) = &self.run {
            let anon = LeafRequest::new(&AllotmentName::new(""));
            let run = run.map_allotments(|_| anon.clone());
            let area = SpaceBaseArea::new(PartialSpaceBase::from_spacebase(self.position.clone()),PartialSpaceBase::from_spacebase(run)).unwrap();
            area.make_base_filter(min,max)
        } else {
            self.position.make_base_filter(min,max)
        }
    }

    pub fn base_filter(&self, min: f64, max: f64) -> TextShape<LeafRequest> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.make_base_filter(min,max);
        self.filter(&filter.or(&non_tracking))
    }
}

impl TextShape<LeafStyle> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self,  cat: &D) -> Vec<(T,TextShape<LeafStyle>)> where D: ShapeDemerge<X=T> {
        let demerge = self.position.allotments().demerge(self.position.len(),|a| cat.categorise(&a.coord_system));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            out.push((draw_group,self.filter(&mut filter)));
        }
        out
    }
}

impl TextShape<Arc<dyn Transformer>> {
    fn demerge_by_variety(&self) -> Vec<((TransformerVariety,CoordinateSystem),TextShape<Arc<dyn Transformer>>)> {
        let demerge = self.position.allotments().demerge(self.position.len(),|x| {
            x.choose_variety()
        });
        let mut out = vec![];
        for (variety,filter) in demerge {
            out.push((variety,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self) -> Vec<TextShape<LeafStyle>> {
        let mut out = vec![];
        for ((variety,coord_system),texts) in self.demerge_by_variety() {
            out.push(TextShape {
                position: variety.spacebase_transform(&coord_system,&texts.position),
                run: texts.run.clone(),
                text: texts.text.clone(),
                pen: texts.pen.clone()
            });
        }
        out
    }
}
