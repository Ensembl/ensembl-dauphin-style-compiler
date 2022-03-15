use std::hash::Hash;
use std::sync::Arc;
use peregrine_toolkit::puzzle::PuzzleSolution;

use super::imageshape::ImageShape;
use super::rectangleshape::RectangleShape;
use super::textshape::TextShape;
use super::wiggleshape::WiggleShape;
use crate::AllotmentRequest;
use crate::Assets;
use crate::Colour;
use crate::CoordinateSystem;
use crate::DataMessage;
use crate::DrawnType;
use crate::EachOrEvery;
use crate::allotment::core::rangeused::RangeUsed;
use crate::allotment::style::pendingleaf::PendingLeaf;
use crate::allotment::style::style::LeafCommonStyle;
use crate::allotment::transformers::transformers::Transformer;
use crate::allotment::tree::allotmentbox::AllotmentBox;
use crate::util::eachorevery::EachOrEveryFilter;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, coord_system: &CoordinateSystem) -> Self::X;
    
    fn categorise_with_colour(&self, coord_system: &CoordinateSystem, _variety: &DrawnType, _colour: &Colour) -> Self::X {
        self.categorise(coord_system)
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Shape<A> {
    Text(TextShape<A>),
    Image(ImageShape<A>),
    Wiggle(WiggleShape<A>),
    SpaceBaseRect(RectangleShape<A>)
}

impl<A> Clone for Shape<A> where A: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Text(arg0) => Self::Text(arg0.clone()),
            Self::Image(arg0) => Self::Image(arg0.clone()),
            Self::Wiggle(arg0) => Self::Wiggle(arg0.clone()),
            Self::SpaceBaseRect(arg0) => Self::SpaceBaseRect(arg0.clone()),
        }
    }
}

impl<A> Shape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> Shape<B> where F: Fn(&A) -> B {
        match self {
            Self::Text(arg0) => Shape::<B>::Text(arg0.map_new_allotment(cb)),
            Self::Image(arg0) => Shape::<B>::Image(arg0.map_new_allotment(cb)),
            Self::Wiggle(arg0) => Shape::<B>::Wiggle(arg0.map_new_allotment(cb)),
            Self::SpaceBaseRect(arg0) => Shape::<B>::SpaceBaseRect(arg0.map_new_allotment(cb)),
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Shape::SpaceBaseRect(shape) => shape.len(),
            Shape::Text(shape) => shape.len(),
            Shape::Image(shape) => shape.len(),
            Shape::Wiggle(shape) => shape.len()
        }
    }
}

impl<A: Clone> Shape<A> {
    pub fn filter(&self, filter: &EachOrEveryFilter) -> Shape<A> {
        match self {
            Shape::SpaceBaseRect(shape) => Shape::SpaceBaseRect(shape.filter(filter)),
            Shape::Text(shape) => Shape::Text(shape.filter(filter)),
            Shape::Image(shape) => Shape::Image(shape.filter(filter)),
            Shape::Wiggle(shape) => Shape::Wiggle(shape.filter(filter))
        }
    }
}

impl Shape<PendingLeaf> {
    pub fn base_filter(&self, min: f64, max: f64) -> Shape<PendingLeaf> {
        match self {
            Shape::SpaceBaseRect(shape) => Shape::SpaceBaseRect(shape.base_filter(min,max)),
            Shape::Text(shape) => Shape::Text(shape.base_filter(min,max)),
            Shape::Image(shape) => Shape::Image(shape.base_filter(min,max)),
            Shape::Wiggle(shape) => Shape::Wiggle(shape.base_filter(min,max)),
        }
    }
}

impl Shape<LeafCommonStyle> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape<LeafCommonStyle>)> where D: ShapeDemerge<X=T> {
        match self {
            Shape::Wiggle(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)| 
                    (x,Shape::Wiggle(details))
                ).collect()
            },
            Shape::Text(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)|
                    (x,Shape::Text(details))
                ).collect()
            },
            Shape::Image(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)|
                    (x,Shape::Image(details))
                ).collect()
            },
            Shape::SpaceBaseRect(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)|
                    (x,Shape::SpaceBaseRect(details))
                ).collect()
            }
        }
    }
}

impl Shape<Arc<dyn Transformer>> {
    pub fn make(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> {
        match self {
            Shape::SpaceBaseRect(shape) => shape.make(solution).drain(..).map(|x| Shape::SpaceBaseRect(x)).collect(),
            Shape::Text(shape) => shape.make(solution).drain(..).map(|x| Shape::Text(x)).collect(),
            Shape::Image(shape) => shape.make(solution).drain(..).map(|x| Shape::Image(x)).collect(),
            Shape::Wiggle(shape) => shape.make(solution).drain(..).map(|x| Shape::Wiggle(x)).collect(),
        }
    }
}

impl Shape<PendingLeaf> {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            Shape::SpaceBaseRect(shape) => {
                for (top_left,bottom_right) in shape.area().iter() {
                    top_left.allotment.update_drawing_info(|allotment| {
                        allotment.merge_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
                        allotment.merge_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
                        allotment.merge_max_y(top_left.normal.ceil());
                        allotment.merge_max_y(bottom_right.normal.ceil());
                    });
                }
            },
            Shape::Text(shape) => {
                let size = shape.pen().size_in_webgl();
                for (position,text) in shape.position().iter().zip(shape.iter_texts()) {
                    position.allotment.update_drawing_info(|allotment| {
                        allotment.merge_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                        allotment.merge_pixel_range(&RangeUsed::Part(0.,size*text.len() as f64)); // Not ideal: assume square
                        allotment.merge_max_y((*position.normal + size).ceil());
                    });
                }
            },
            Shape::Image(shape) => {
                for (position,asset_name) in shape.position().iter().zip(shape.iter_names()) {
                    position.allotment.update_drawing_info(|allotment| {
                        allotment.merge_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                        if let Some(asset) = assets.get(asset_name) {
                            if let Some(height) = asset.metadata_u32("height") {
                                allotment.merge_max_y((position.normal + (height as f64)).ceil());
                            }
                            if let Some(width) = asset.metadata_u32("width") {
                                allotment.merge_pixel_range(&RangeUsed::Part(0.,(position.normal + (width as f64)).ceil()));
                            }
                        }
                    });
                }
            },
            Shape::Wiggle(shape) => {
                for allotment in shape.iter_allotments(1) {
                    allotment.update_drawing_info(|allotment| {
                        allotment.merge_base_range(&RangeUsed::All);
                        allotment.merge_max_y(shape.plotter().0);
                    });
                }
            }
        }
        Ok(())
    }
}

impl Shape<AllotmentRequest> {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            Shape::SpaceBaseRect(shape) => {
                for (top_left,bottom_right) in shape.area().iter() {
                    let allotment = top_left.allotment;
                    allotment.set_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
                    allotment.set_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
                    allotment.set_max_y(top_left.normal.ceil() as i64);
                    allotment.set_max_y(bottom_right.normal.ceil() as i64);
                }
            },
            Shape::Text(shape) => {
                let size = shape.pen().size_in_webgl() as f64;
                for (position,text) in shape.position().iter().zip(shape.iter_texts()) {
                    let allotment = position.allotment;
                    allotment.set_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                    allotment.set_pixel_range(&RangeUsed::Part(0.,size*text.len() as f64)); // Not ideal: assume square
                    allotment.set_max_y((*position.normal + size).ceil() as i64);
                }
            },
            Shape::Image(shape) => {
                for (position,asset_name) in shape.position().iter().zip(shape.iter_names()) {
                    let allotment = position.allotment;
                    allotment.set_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                    if let Some(asset) = assets.get(asset_name) {
                        if let Some(height) = asset.metadata_u32("height") {
                            allotment.set_max_y((position.normal + (height as f64)).ceil() as i64);
                        }
                        if let Some(width) = asset.metadata_u32("width") {
                            allotment.set_pixel_range(&RangeUsed::Part(0.,(position.normal + (width as f64)).ceil()));
                        }
                    }
                }
            },
            Shape::Wiggle(shape) => {
                for allotment in shape.iter_allotments(1) {
                    allotment.set_base_range(&RangeUsed::All);
                    allotment.set_max_y(shape.plotter().0 as i64);    
                }
            }
        }
        Ok(())
    }

    pub fn allot<F,E>(self, cb: F) -> Result<Shape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(match self {
            Shape::Wiggle(shape) => Shape::Wiggle(shape.allot(cb)?),
            Shape::Text(shape) => Shape::Text(shape.allot(cb)?),
            Shape::Image(shape) => Shape::Image(shape.allot(cb)?),
            Shape::SpaceBaseRect(shape) =>Shape::SpaceBaseRect(shape.allot(cb)?),
        })
    }
}
