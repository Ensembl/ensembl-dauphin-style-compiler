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
pub enum ShapeDetails {
    Text(TextShape),
    Image(ImageShape),
    Wiggle(WiggleShape),
    SpaceBaseRect(RectangleShape)
}

impl ShapeDetails {
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

    pub fn filter(&self, filter: &DataFilter) -> ShapeDetails {
        match self {
            ShapeDetails::SpaceBaseRect(shape) => ShapeDetails::SpaceBaseRect(shape.filter(filter)),
            ShapeDetails::Text(shape) => ShapeDetails::Text(shape.filter(filter)),
            ShapeDetails::Image(shape) => ShapeDetails::Image(shape.filter(filter)),
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.filter(filter))
        }
    }

    pub fn reduce_by_minmax(&self, min_value: f64, max_value: f64) -> ShapeDetails {
        match self {
            ShapeDetails::Wiggle(shape) => ShapeDetails::Wiggle(shape.reduce_by_minmax(min_value,max_value)),
            x => x.clone()
        }
    }

    pub fn register_space(&self, common: &ShapeCommon<AllotmentRequest>, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => {
                for ((top_left,bottom_right),allotment) in shape.area().iter().zip(common.iter_allotments()) {
                    allotment.set_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
                    allotment.set_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
                    allotment.set_max_y(top_left.normal.ceil() as i64);
                    allotment.set_max_y(bottom_right.normal.ceil() as i64);
                }
            },
            ShapeDetails::Text(shape) => {
                let size = shape.pen().size() as f64;
                for ((position,allotment),text) in shape.position().iter().zip(common.iter_allotments()).zip(shape.iter_texts()) {
                    allotment.set_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                    allotment.set_pixel_range(&RangeUsed::Part(0.,size*text.len() as f64)); // Not ideal: assume square
                    allotment.set_max_y((*position.normal + size).ceil() as i64);
                }
            },
            ShapeDetails::Image(shape) => {
                for (position,(allotment,asset_name)) in shape.position().iter().zip(common.iter_allotments().zip(shape.iter_names())) {
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
                for allotment in common.iter_allotments() {
                    allotment.set_base_range(&RangeUsed::All);
                    allotment.set_max_y(shape.plotter().0 as i64);    
                }
            }
        }
        Ok(())
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common: &ShapeCommon<Allotment>, cat: &D) -> Vec<(T,Shape<Allotment>)> where D: ShapeDemerge<X=T> {
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

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeCommon<A: Clone> {
    len: usize,
    coord_system: CoordinateSystem,
    allotments: EachOrEvery<A>
}

impl<A: Clone> ShapeCommon<A> {
    pub fn new(len: usize, coord_system: CoordinateSystem, allotments: EachOrEvery<A>) -> Option<ShapeCommon<A>> {
        if !allotments.compatible(len) { return None; }
        Some(ShapeCommon { len, coord_system, allotments })
    }

    pub fn coord_system(&self) -> &CoordinateSystem { &self.coord_system }
    pub fn allotments(&self) -> &EachOrEvery<A> { &self.allotments }

    pub fn filter(&self, filter: &DataFilter) -> ShapeCommon<A> {
        let allotments = self.allotments.filter(filter);
        ShapeCommon {
            len: filter.count(),
            coord_system: self.coord_system.clone(),
            allotments
        }
    }

    pub fn iter_allotments(&self) -> impl Iterator<Item=&A> {
        self.allotments.iter(self.len).unwrap()
    }

    pub fn into_map_result_box<F,E,B: Clone>(self, cb: F) -> Result<ShapeCommon<B>,E> where F: FnMut(&A) -> Result<B,E> {
        Ok(ShapeCommon {
            len: self.len,
            coord_system: self.coord_system,
            allotments: self.allotments.map_results(cb)?
        })
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Shape<A: Clone> {
    details: ShapeDetails,
    common: ShapeCommon<A>
}

impl<A: Clone> Shape<A> {
    pub fn new(common: ShapeCommon<A>, details: ShapeDetails) -> Shape<A> {
        Shape { common, details }
    }

    pub fn details(&self) -> &ShapeDetails { &self.details }
    pub fn common(&self) -> &ShapeCommon<A> { &self.common }

    pub(super) fn filter_shape(&self, filter: &DataFilter) -> Shape<A> {
        let mut filter = filter.clone();
        filter.set_size(self.len());
        let common = self.common.filter(&filter);
        let details = self.details.filter(&filter);
        Shape::new(common,details)
    }

    pub fn filter_by_allotment<F>(&self,  cb: F)  -> Shape<A> where F: Fn(&A) -> bool {
        let filter = self.common.allotments().new_filter(self.len(),cb);
        self.filter_shape(&filter)
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

    pub fn into_map_result_box<F,E,B: Clone>(self, cb: F) -> Result<Shape<B>,E> where F: FnMut(&A) -> Result<B,E> {
        Ok(Shape {
            details: self.details,
            common: self.common.into_map_result_box(cb)?
        })
    }
}

impl Shape<Allotment> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape<Allotment>)> where D: ShapeDemerge<X=T> {
        self.details.demerge(&self.common,cat)
    }
}

impl Shape<AllotmentRequest> {
    pub fn remove_nulls(self) -> Shape<AllotmentRequest> { self.filter_by_allotment(|a| !a.is_dustbin()) }

    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        self.details.register_space(&self.common,assets)
    }
}