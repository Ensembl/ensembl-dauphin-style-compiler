use std::hash::Hash;
use std::sync::Arc;

use super::emptyshape::EmptyShape;
use super::imageshape::ImageShape;
use super::rectangleshape::RectangleShape;
use super::textshape::TextShape;
use super::wiggleshape::WiggleShape;
use crate::Assets;
use crate::Colour;
use crate::CoordinateSystem;
use crate::DataMessage;
use crate::DrawnType;
use crate::LeafRequest;
use crate::SpaceBaseArea;
use crate::allotment::core::boxtraits::Transformable;
use crate::allotment::style::style::LeafStyle;
use crate::allotment::transformers::transformers::Transformer;
use crate::allotment::util::rangeused::RangeUsed;
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
    SpaceBaseRect(RectangleShape<A>),
    Empty(EmptyShape<A>)
}

/* A shape without reference to its surroundings re placement -> */
pub(crate) type UnplacedShape = Shape<LeafRequest>;

/* -> A shape with reference only to the surroundings of its own carriage -> */
pub(crate) type AbstractShape = Shape<Arc<dyn Transformable>>;

/* -> A completely placed shape, ready to draw */
pub type DrawingShape = Shape<LeafStyle>;

impl<A> Clone for Shape<A> where A: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Text(arg0) => Self::Text(arg0.clone()),
            Self::Image(arg0) => Self::Image(arg0.clone()),
            Self::Wiggle(arg0) => Self::Wiggle(arg0.clone()),
            Self::SpaceBaseRect(arg0) => Self::SpaceBaseRect(arg0.clone()),
            Self::Empty(arg0) => Self::Empty(arg0.clone())
        }
    }
}

impl<A> Shape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> Shape<B> where F: FnMut(&A) -> B {
        match self {
            Self::Text(arg0) => Shape::<B>::Text(arg0.map_new_allotment(cb)),
            Self::Image(arg0) => Shape::<B>::Image(arg0.map_new_allotment(cb)),
            Self::Wiggle(arg0) => Shape::<B>::Wiggle(arg0.map_new_allotment(cb)),
            Self::SpaceBaseRect(arg0) => Shape::<B>::SpaceBaseRect(arg0.map_new_allotment(cb)),
            Self::Empty(arg0) => Shape::<B>::Empty(arg0.map_new_allotment(cb)),
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Shape::SpaceBaseRect(shape) => shape.len(),
            Shape::Text(shape) => shape.len(),
            Shape::Image(shape) => shape.len(),
            Shape::Wiggle(shape) => shape.len(),
            Shape::Empty(shape) => shape.len()
        }
    }
}

impl<A: Clone> Shape<A> {
    pub fn filter(&self, filter: &EachOrEveryFilter) -> Shape<A> {
        match self {
            Shape::SpaceBaseRect(shape) => Shape::SpaceBaseRect(shape.filter(filter)),
            Shape::Text(shape) => Shape::Text(shape.filter(filter)),
            Shape::Image(shape) => Shape::Image(shape.filter(filter)),
            Shape::Wiggle(shape) => Shape::Wiggle(shape.filter(filter)),
            Shape::Empty(shape) => Shape::Empty(shape.filter(filter))
        }
    }
}

impl Shape<LeafRequest> {
    pub fn base_filter(&self, min: f64, max: f64) -> Shape<LeafRequest> {
        match self {
            Shape::SpaceBaseRect(shape) => Shape::SpaceBaseRect(shape.base_filter(min,max)),
            Shape::Text(shape) => Shape::Text(shape.base_filter(min,max)),
            Shape::Image(shape) => Shape::Image(shape.base_filter(min,max)),
            Shape::Wiggle(shape) => Shape::Wiggle(shape.base_filter(min,max)),
            Shape::Empty(shape) => Shape::Empty(shape.base_filter(min, max))
        }
    }
}

impl Shape<LeafStyle> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,Shape<LeafStyle>)> where D: ShapeDemerge<X=T> {
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
            },
            Shape::Empty(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)|
                    (x,Shape::Empty(details))
                ).collect()
            },
        }
    }
}

impl Shape<Arc<dyn Transformer>> {
    pub fn make(&self) -> Vec<Shape<LeafStyle>> {
        match self {
            Shape::SpaceBaseRect(shape) => shape.make().drain(..).map(|x| Shape::SpaceBaseRect(x)).collect(),
            Shape::Text(shape) => shape.make().drain(..).map(|x| Shape::Text(x)).collect(),
            Shape::Image(shape) => shape.make().drain(..).map(|x| Shape::Image(x)).collect(),
            Shape::Wiggle(shape) => shape.make().drain(..).map(|x| Shape::Wiggle(x)).collect(),
            Shape::Empty(shape) => shape.make().drain(..).map(|x| Shape::Empty(x)).collect(),
        }
    }
}

fn register_space_area(area: &SpaceBaseArea<f64,LeafRequest>) {
    for (top_left,bottom_right) in area.iter() {
        top_left.allotment.update_drawing_info(|allotment| {
            allotment.merge_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
            allotment.merge_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
            allotment.merge_max_y(top_left.normal.ceil());
            allotment.merge_max_y(bottom_right.normal.ceil());
        });
    }
}

impl Shape<LeafRequest> {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            Shape::SpaceBaseRect(shape) => {
                register_space_area(shape.area());
            },
            Shape::Empty(area) => {
                register_space_area(area.area());
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
