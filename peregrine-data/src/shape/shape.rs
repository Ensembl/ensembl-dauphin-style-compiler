use std::hash::Hash;
use super::imageshape::ImageShape;
use super::rectangleshape::RectangleShape;
use super::textshape::TextShape;
use super::wiggleshape::WiggleShape;
use crate::Allotment;
use crate::AllotmentRequest;
use crate::Assets;
use crate::Colour;
use crate::CoordinateSystem;
use crate::DataFilter;
use crate::DataMessage;
use crate::DrawnType;
use crate::EachOrEvery;
use crate::allotment::core::rangeused::RangeUsed;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, coord_system: &CoordinateSystem) -> Self::X;
    
    fn categorise_with_colour(&self, coord_system: &CoordinateSystem, _variety: &DrawnType, _colour: &Colour) -> Self::X {
        self.categorise(coord_system)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum ShapeDetails<A: Clone> {
    Text(TextShape<A>),
    Image(ImageShape<A>),
    Wiggle(WiggleShape<A>),
    SpaceBaseRect(RectangleShape<A>)
}

impl<A: Clone> ShapeDetails<A> {
    pub fn len(&self) -> usize {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => shape.len(),
            ShapeDetails::Text(shape) => shape.len(),
            ShapeDetails::Image(shape) => shape.len(),
            ShapeDetails::Wiggle(shape) => shape.len()
        }
    }

    pub fn make_base_filter(&self, min: f64, max: f64) -> DataFilter {
        match self {
            ShapeDetails::SpaceBaseRect(shape) => shape.make_base_filter(min,max),
            ShapeDetails::Text(shape) => shape.make_base_filter(min,max),
            ShapeDetails::Image(shape) => shape.make_base_filter(min,max),
            ShapeDetails::Wiggle(shape) => shape.make_base_filter(min,max),
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> ShapeDetails<A> {
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

impl ShapeDetails<AllotmentRequest> {

    pub fn register_space(&self, common: &ShapeCommon, assets: &Assets) -> Result<(),DataMessage> {
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
                let size = shape.pen().size() as f64;
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

    pub fn allot<F,E>(self, cb: F) -> Result<ShapeDetails<Allotment>,E> where F: Fn(&AllotmentRequest) -> Result<Allotment,E> {
        Ok(match self {
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.allot(cb)?),
            ShapeDetails::Text(shape) => ShapeDetails::Text(shape.allot(cb)?),
            ShapeDetails::Image(shape) => ShapeDetails::Image(shape.allot(cb)?),
            ShapeDetails::SpaceBaseRect(shape) =>ShapeDetails::SpaceBaseRect(shape.allot(cb)?),
        })
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeCommon {
    coord_system: CoordinateSystem,
    depth: EachOrEvery<i8>
}

impl ShapeCommon {
    pub fn new(coord_system: CoordinateSystem, depth: EachOrEvery<i8>) -> Option<ShapeCommon> {
        Some(ShapeCommon { coord_system, depth })
    }

    pub fn depth(&self) -> &EachOrEvery<i8> { &self.depth }
    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }

    pub fn filter(&self, filter: &DataFilter) -> ShapeCommon {
        ShapeCommon {
            coord_system: self.coord_system.clone(),
            depth: self.depth.filter(filter)
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Shape<A: Clone> {
    details: ShapeDetails<A>,
    common: ShapeCommon
}

impl<A: Clone> Shape<A> {
    pub fn new(common: ShapeCommon, details: ShapeDetails<A>) -> Shape<A> {
        Shape { common, details }
    }

    pub fn details(&self) -> &ShapeDetails<A> { &self.details }
    pub fn common(&self) -> &ShapeCommon { &self.common }

    pub(super) fn filter_shape(&self, filter: &DataFilter) -> Shape<A> {
        let mut filter = filter.clone();
        filter.set_size(self.len());
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
    
    pub fn len(&self) -> usize { self.details.len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

impl Shape<Allotment> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape<Allotment>)> where D: ShapeDemerge<X=T> {
        self.details.demerge(&self.common,cat)
    }
}

impl Shape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<Shape<Allotment>,E> where F: Fn(&AllotmentRequest) -> Result<Allotment,E> {
        Ok(Shape {
            details: self.details.allot(&cb)?,
            common: self.common.clone()
        })
    }

    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        self.details.register_space(&self.common,assets)
    }
}