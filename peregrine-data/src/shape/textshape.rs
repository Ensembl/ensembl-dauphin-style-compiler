use peregrine_toolkit::eachorevery::{EachOrEveryFilter, EachOrEvery};

use crate::{DataMessage, Pen, ShapeDemerge, Shape, SpaceBase, allotment::{transformers::{transformers::{Transformer, TransformerVariety}}, style::{style::LeafStyle}, util::rangeused::RangeUsed}, CoordinateSystem, LeafRequest, SpaceBaseArea, SpaceBasePointRef};
use std::{hash::Hash, sync::Arc};

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum TextShapePosition<A> {
    Normal(SpaceBase<f64,A>),
    Running(SpaceBaseArea<f64,A>)
}

impl<A> Clone for TextShapePosition<A> where A: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Normal(arg0) => Self::Normal(arg0.clone()),
            Self::Running(arg0) => Self::Running(arg0.clone()),
        }
    }
}

impl<A> TextShapePosition<A> {
    fn len(&self) -> usize {
        match self {
            TextShapePosition::Normal(x) => x.len(),
            TextShapePosition::Running(x) => x.len()
        }
    }

    pub fn major(&self) -> &SpaceBase<f64,A> {
        match self {
            TextShapePosition::Normal(x) => x,
            TextShapePosition::Running(x) => x.top_left()
        }        
    }

    pub fn map_allotments<F,B>(&self, cb: F) -> TextShapePosition<B> where F: FnMut(&A) -> B {
        match self {
            Self::Normal(x) =>  TextShapePosition::Normal(x.map_allotments(cb)),
            Self::Running(x) => TextShapePosition::Running(x.map_allotments(cb))
        }
    }

    pub fn allotments(&self) -> &EachOrEvery<A> { self.major().allotments() }

    fn filter(&self, filter: &EachOrEveryFilter) -> TextShapePosition<A> {
        match self {
            Self::Normal(x) =>  Self::Normal(x.filter(filter)),
            Self::Running(x) => Self::Running(x.filter(filter))
        }
    }

    fn make_base_filter(&self, min: f64, max: f64) -> EachOrEveryFilter {
        match self {
            Self::Normal(x) =>  x.make_base_filter(min,max),
            Self::Running(x) => x.make_base_filter(min,max)

        }
    }

    pub fn iter_major<'a>(&'a self) -> impl Iterator<Item=SpaceBasePointRef<'a,f64,A>> {
        match self {
            Self::Normal(x) => x.iter(),
            Self::Running(x) => x.top_left().iter()
        }
    }

    fn iter_minor<'a>(&'a self) -> Option<impl Iterator<Item=SpaceBasePointRef<'a,f64,A>>> {
        match self {
            Self::Normal(_) => None,
            Self::Running(x) => Some(x.bottom_right().iter())
        }
    }
}

impl<A: Clone> TextShapePosition<A> {
    pub fn fold_tangent<F,Z>(&mut self, values: &[Z], cb: F) -> bool where F: Fn(&f64,&Z) -> f64 {
        match self {
            Self::Normal(x) => x.fold_tangent(values,cb),
            Self::Running(x) => {
                x.top_left_mut().fold_tangent(values,&cb) &&
                x.bottom_right_mut().fold_tangent(values,&cb)
            }
        }
    }
}

impl TextShapePosition<Arc<dyn Transformer>> {
    fn transform(&self, variety: &TransformerVariety, coord_system: &CoordinateSystem) -> TextShapePosition<LeafStyle> {
        match self {
            Self::Normal(position) => TextShapePosition::Normal(variety.spacebase_transform(&coord_system,&position)),
            Self::Running(position) => TextShapePosition::Running(variety.spacebasearea_transform(&coord_system,&position)),
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TextShape<A> {
    position: TextShapePosition<A>,
    pen: Pen,
    text: EachOrEvery<String>
}

impl<A> TextShape<A> {
    pub(super) fn map_new_allotment<F,B>(&self, cb: F) -> TextShape<B> where F: FnMut(&A) -> B {
        TextShape {
            position: self.position.map_allotments(cb),
            pen: self.pen.clone(),
            text: self.text.clone()
        }
    }

    pub fn len(&self) -> usize { self.position.len() }
    pub fn pen(&self) -> &Pen { &self.pen }
    pub fn position(&self) -> &TextShapePosition<A> { &self.position }

    pub fn iter_texts(&self) -> impl Iterator<Item=&String> {
        self.text.iter(self.position.len()).unwrap()
    }

    fn new_details(position: TextShapePosition<A>, pen: Pen, text: EachOrEvery<String>) -> Result<TextShape<A>,DataMessage> {
        if !text.compatible(position.len()) { return Err(DataMessage::LengthMismatch(format!("text patina"))); }
        Ok(TextShape {
            position, pen, text
        })
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> TextShape<A> {
        TextShape {
            position: self.position.filter(filter),
            pen: self.pen.filter(filter),
            text: self.text.filter(filter)
        }
    }
}

impl TextShape<LeafRequest> {
    pub fn new(position: SpaceBase<f64,LeafRequest>, pen: Pen, text: EachOrEvery<String>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = TextShape::new_details(TextShapePosition::Normal(position),pen,text.clone())?;
        Ok(Shape::Text(details))
    }

    pub(super) fn register_space(&self) {
        let size = self.pen().geometry().size_in_webgl();
        let major = self.position().iter_major();
        let minor = self.position().iter_minor();
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
        Self { position: self.position.clone(), pen: self.pen.clone(), text: self.text.clone() }
    }
}

impl TextShape<LeafRequest> {
    pub fn base_filter(&self, min: f64, max: f64) -> TextShape<LeafRequest> {
        let non_tracking = self.position.allotments().make_filter(self.position.len(),|a| !a.leaf_style().coord_system.is_tracking());
        let filter = self.position.make_base_filter(min,max);
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
                position: texts.position.transform(&variety,&coord_system),
                text: texts.text.clone(),
                pen: texts.pen.clone()
            });
        }
        out
    }
}
