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
pub enum ShapeDetails<A> {
    Text(TextShape<A>),
    Image(ImageShape<A>),
    Wiggle(WiggleShape<A>),
    SpaceBaseRect(RectangleShape<A>)
}

impl<A> Clone for ShapeDetails<A> where A: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Text(arg0) => Self::Text(arg0.clone()),
            Self::Image(arg0) => Self::Image(arg0.clone()),
            Self::Wiggle(arg0) => Self::Wiggle(arg0.clone()),
            Self::SpaceBaseRect(arg0) => Self::SpaceBaseRect(arg0.clone()),
        }
    }
}

impl<A> ShapeDetails<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> ShapeDetails<B> where F: Fn(&A) -> B {
        match self {
            Self::Text(arg0) => ShapeDetails::<B>::Text(arg0.map_new_allotment(cb)),
            Self::Image(arg0) => ShapeDetails::<B>::Image(arg0.map_new_allotment(cb)),
            Self::Wiggle(arg0) => ShapeDetails::<B>::Wiggle(arg0.map_new_allotment(cb)),
            Self::SpaceBaseRect(arg0) => ShapeDetails::<B>::SpaceBaseRect(arg0.map_new_allotment(cb)),
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => shape.len(),
            ShapeDetails::Text(shape) => shape.len(),
            ShapeDetails::Image(shape) => shape.len(),
            ShapeDetails::Wiggle(shape) => shape.len()
        }
    }
}

impl<A: Clone> ShapeDetails<A> {
    pub fn make_base_filter(&self, min: f64, max: f64) -> EachOrEveryFilter {
        match self {
            ShapeDetails::SpaceBaseRect(shape) => shape.make_base_filter(min,max),
            ShapeDetails::Text(shape) => shape.make_base_filter(min,max),
            ShapeDetails::Image(shape) => shape.make_base_filter(min,max),
            ShapeDetails::Wiggle(shape) => shape.make_base_filter(min,max),
        }
    }

    pub fn filter(&self, filter: &EachOrEveryFilter) -> ShapeDetails<A> {
        match self {
            ShapeDetails::SpaceBaseRect(shape) => ShapeDetails::SpaceBaseRect(shape.filter(filter)),
            ShapeDetails::Text(shape) => ShapeDetails::Text(shape.filter(filter)),
            ShapeDetails::Image(shape) => ShapeDetails::Image(shape.filter(filter)),
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.filter(filter))
        }
    }

    pub fn reduce_by_minmax(&self, min_value: f64, max_value: f64) -> ShapeDetails<A> {
        match self {
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.reduce_by_minmax(min_value,max_value)),
            x => x.clone()
        }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common: &ShapeCommon, cat: &D) -> Vec<(T,Shape<A>)> where D: ShapeDemerge<X=T> {
        match self {
            ShapeDetails::Wiggle(shape) => {
                return shape.demerge(common,cat).drain(..).map(|(x,common,details)| 
                    (x, Shape { common, details: ShapeDetails::Wiggle(details) })
                ).collect()
            },
            ShapeDetails::Text(shape) => {
                return shape.demerge(common,cat).drain(..).map(|(x,common,details)|
                    (x, Shape { common, details: ShapeDetails::Text(details) })
                ).collect()
            },
            ShapeDetails::Image(shape) => {
                return shape.demerge(common,cat).drain(..).map(|(x,common,details)|
                    (x, Shape { common, details: ShapeDetails::Image(details) })
                ).collect()
            },
            ShapeDetails::SpaceBaseRect(shape) => {
                return shape.demerge(common,cat).drain(..).map(|(x,common,details)|
                    (x, Shape { common, details: ShapeDetails::SpaceBaseRect(details) })
                ).collect()
            }
        }
    }
}

impl ShapeDetails<Arc<dyn Transformer>> {
    pub fn make(&self, solution: &PuzzleSolution, common: &ShapeCommon) -> Vec<ShapeDetails<()>> {
        match self {
            ShapeDetails::SpaceBaseRect(shape) => shape.make(solution,common).drain(..).map(|x| ShapeDetails::SpaceBaseRect(x)).collect(),
            ShapeDetails::Text(shape) => shape.make(solution,common).drain(..).map(|x| ShapeDetails::Text(x)).collect(),
            ShapeDetails::Image(shape) => shape.make(solution,common).drain(..).map(|x| ShapeDetails::Image(x)).collect(),
            ShapeDetails::Wiggle(shape) => shape.make(solution,common).drain(..).map(|x| ShapeDetails::Wiggle(x)).collect(),
        }
    }
}

impl ShapeDetails<PendingLeaf> {
    pub fn register_space(&self, _common: &ShapeCommon, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => {
                for (top_left,bottom_right) in shape.area().iter() {
                    top_left.allotment.update_drawing_info(|allotment| {
                        allotment.merge_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
                        allotment.merge_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
                        allotment.merge_max_y(top_left.normal.ceil());
                        allotment.merge_max_y(bottom_right.normal.ceil());
                    });
                }
            },
            ShapeDetails::Text(shape) => {
                let size = shape.pen().size_in_webgl();
                for (position,text) in shape.position().iter().zip(shape.iter_texts()) {
                    position.allotment.update_drawing_info(|allotment| {
                        allotment.merge_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                        allotment.merge_pixel_range(&RangeUsed::Part(0.,size*text.len() as f64)); // Not ideal: assume square
                        allotment.merge_max_y((*position.normal + size).ceil());
                    });
                }
            },
            ShapeDetails::Image(shape) => {
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
            ShapeDetails::Wiggle(shape) => {
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

impl ShapeDetails<AllotmentRequest> {
    pub fn register_space(&self, _common: &ShapeCommon, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => {
                for (top_left,bottom_right) in shape.area().iter() {
                    let allotment = top_left.allotment;
                    allotment.set_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
                    allotment.set_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
                    allotment.set_max_y(top_left.normal.ceil() as i64);
                    allotment.set_max_y(bottom_right.normal.ceil() as i64);
                }
            },
            ShapeDetails::Text(shape) => {
                let size = shape.pen().size_in_webgl() as f64;
                for (position,text) in shape.position().iter().zip(shape.iter_texts()) {
                    let allotment = position.allotment;
                    allotment.set_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                    allotment.set_pixel_range(&RangeUsed::Part(0.,size*text.len() as f64)); // Not ideal: assume square
                    allotment.set_max_y((*position.normal + size).ceil() as i64);
                }
            },
            ShapeDetails::Image(shape) => {
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
            ShapeDetails::Wiggle(shape) => {
                for allotment in shape.iter_allotments(1) {
                    allotment.set_base_range(&RangeUsed::All);
                    allotment.set_max_y(shape.plotter().0 as i64);    
                }
            }
        }
        Ok(())
    }

    pub fn allot<F,E>(self, cb: F) -> Result<ShapeDetails<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(match self {
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.allot(cb)?),
            ShapeDetails::Text(shape) => ShapeDetails::Text(shape.allot(cb)?),
            ShapeDetails::Image(shape) => ShapeDetails::Image(shape.allot(cb)?),
            ShapeDetails::SpaceBaseRect(shape) =>ShapeDetails::SpaceBaseRect(shape.allot(cb)?),
        })
    }
}

impl ShapeDetails<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> ShapeDetails<()> {
        match self {
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.transform(common,solution)),
            ShapeDetails::Text(shape) => ShapeDetails::Text(shape.transform(common,solution)),
            ShapeDetails::Image(shape) => ShapeDetails::Image(shape.transform(common,solution)),
            ShapeDetails::SpaceBaseRect(shape) => ShapeDetails::SpaceBaseRect(shape.transform(common,solution)),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeCommon {
    coord_system: CoordinateSystem,
    depth: EachOrEvery<i8>
}

impl ShapeCommon {
    pub fn new(coord_system: CoordinateSystem, depth: EachOrEvery<i8>) -> ShapeCommon {
        ShapeCommon { coord_system, depth }
    }

    pub fn depth(&self) -> &EachOrEvery<i8> { &self.depth }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }

    pub fn filter(&self, filter: &EachOrEveryFilter) -> ShapeCommon {
        ShapeCommon {
            coord_system: self.coord_system.clone(),
            depth: self.depth.filter(filter)
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Shape<A> {
    details: ShapeDetails<A>,
    common: ShapeCommon
}

impl<A> Clone for Shape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { details: self.details.clone(), common: self.common.clone() }
    }
}

impl<A> Shape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> Shape<B> where F: Fn(&A) -> B {
        Shape {
            common: self.common.clone(),
            details: self.details.map_new_allotment(cb)
        }
    }

    pub fn len(&self) -> usize { self.details.len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn common(&self) -> &ShapeCommon { &self.common }
}

impl<A: Clone> Shape<A> {
    pub fn new(common: ShapeCommon, details: ShapeDetails<A>) -> Shape<A> {
        Shape { common, details }
    }

    pub fn details(&self) -> &ShapeDetails<A> { &self.details }

    pub(super) fn filter_shape(&self, filter: &EachOrEveryFilter) -> Shape<A> {
        let common = self.common.filter(&filter);
        let details = self.details.filter(&filter);
        Shape::new(common,details)
    }

    pub fn filter_by_minmax(&self, min_value: f64, max_value: f64) -> Shape<A> {
        if !self.common.coord_system.is_tracking() {
            return self.clone();
        }
        let filter = self.details.make_base_filter(min_value,max_value);
        let common = self.common.filter(&filter);
        let details = self.details.filter(&filter).reduce_by_minmax(min_value,max_value);
        Shape::new(common,details)
    }    
}

impl<Z: Clone> Shape<Z> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape<Z>)> where D: ShapeDemerge<X=T> {
        self.details.demerge(&self.common,cat)
    }
}

impl Shape<AllotmentBox> {
    pub fn transform(&self, solution: &PuzzleSolution) -> Shape<()> { 
        Shape {
            common: self.common.clone(),
            details: self.details.transform(&self.common,solution)
        }
    }
}

impl Shape<Arc<dyn Transformer>> {
    pub fn make(&self, solution: &PuzzleSolution) -> Vec<Shape<()>> {
        self.details.make(solution,&self.common).drain(..).map(|details| 
            Shape {
                common: self.common.clone(),
                details: details
            }
        ).collect()
    }
}

impl Shape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<Shape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        Ok(Shape {
            details: self.details.allot(&cb)?,
            common: self.common.clone()
        })
    }

    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        self.details.register_space(&self.common,assets)
    }
}

impl Shape<PendingLeaf> {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        self.details.register_space(&self.common,assets)
    }
}
