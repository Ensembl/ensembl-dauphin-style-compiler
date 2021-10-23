use std::hash::Hash;
use super::imageshape::ImageShape;
use super::rectangleshape::RectangleShape;
use super::textshape::TextShape;
use super::wiggleshape::WiggleShape;
use crate::Assets;
use crate::Colour;
use crate::DataFilter;
use crate::DataMessage;
use crate::DrawnType;
use crate::EachOrEvery;
use crate::allotment::allotment::CoordinateSystem;
use crate::allotment::allotmentrequest::AllotmentRequest;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, allotment: &AllotmentRequest) -> Self::X;
    
    fn categorise_with_colour(&self, allotment: &AllotmentRequest, _variety: &DrawnType, _colour: &Colour) -> Self::X {
        self.categorise(allotment)
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

    pub fn register_space(&self, common: &ShapeCommon, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => {
                for ((top_left,bottom_right),allotment) in shape.area().iter().zip(common.iter_allotments()) {
                    allotment.register_usage(top_left.normal.ceil() as i64);
                    allotment.register_usage(bottom_right.normal.ceil() as i64);
                }
            },
            ShapeDetails::Text(shape) => {
                for (position,allotment) in shape.position().iter().zip(common.iter_allotments()) {
                    allotment.register_usage((*position.normal + shape.pen().size() as f64).ceil() as i64);
                }
            },
            ShapeDetails::Image(shape) => {
                for (position,(allotment,asset_name)) in shape.position().iter().zip(common.iter_allotments().zip(shape.iter_names())) {
                    if let Some(asset) = assets.get(asset_name) {
                        if let Some(height) = asset.metadata_u32("height") {
                            allotment.register_usage((position.normal + (height as f64)).ceil() as i64);
                        }
                    }
                }
            },
            ShapeDetails::Wiggle(shape) => {
                shape.allotment().register_usage(shape.plotter().0 as i64);
            }
        }
        Ok(())
    }

    pub fn filter_by_minmax(&self, common: &mut ShapeCommon, min_value: f64, max_value: f64) -> ShapeDetails {
        match &self {
            ShapeDetails::SpaceBaseRect(shape) => {
                ShapeDetails::SpaceBaseRect(shape.filter_by_minmax(common,min_value,max_value))
            },
            ShapeDetails::Text(shape) => {
                ShapeDetails::Text(shape.filter_by_minmax(common,min_value,max_value))
            },
            ShapeDetails::Image(shape) => {
                ShapeDetails::Image(shape.filter_by_minmax(common,min_value,max_value))
            },
            ShapeDetails::Wiggle(shape) => {
                ShapeDetails::Wiggle(shape.filter_by_minmax(common,min_value,max_value))
            }
        }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, common: &ShapeCommon, cat: &D) -> Vec<(T,Shape)> where D: ShapeDemerge<X=T> {
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

    pub fn remove_nulls(self, common: &ShapeCommon) -> Shape {
        let mut common = common.clone();
        let details = match self {
            ShapeDetails::SpaceBaseRect(shape) => {
                ShapeDetails::SpaceBaseRect(shape.filter_by_allotment(&mut common,|a| !a.is_dustbin()))
            },
            ShapeDetails::Text(shape) => {
                ShapeDetails::Text(shape.filter_by_allotment(&mut common, |a| !a.is_dustbin()))
            },
            ShapeDetails::Image(shape) => {
                ShapeDetails::Image(shape.filter_by_allotment(&mut common, |a| !a.is_dustbin()))
            },
            ShapeDetails::Wiggle(shape) => {
                ShapeDetails::Wiggle(shape.filter_by_allotment(&mut common, |a| !a.is_dustbin()))
            }
        };
        Shape { details, common: common.clone() }
    }   
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeCommon {
    len: usize,
    coord_system: CoordinateSystem,
    allotments: EachOrEvery<AllotmentRequest>
}

impl ShapeCommon {
    pub fn new(len: usize, coord_system: CoordinateSystem, allotments: EachOrEvery<AllotmentRequest>) -> Option<ShapeCommon> {
        if !allotments.compatible(len) { return None; }
        Some(ShapeCommon { len, coord_system, allotments })
    }

    pub fn allotments(&self) -> &EachOrEvery<AllotmentRequest> { &self.allotments }

    pub fn filter(&self, filter: &DataFilter) -> ShapeCommon {
        let allotments = self.allotments.filter(filter);
        ShapeCommon {
            len: filter.count(),
            coord_system: self.coord_system.clone(),
            allotments
        }
    }

    pub fn iter_allotments(&self) -> impl Iterator<Item=&AllotmentRequest> {
        self.allotments.iter(self.len).unwrap()
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Shape {
    details: ShapeDetails,
    common: ShapeCommon
}

impl Shape {
    pub fn new(common: ShapeCommon, details: ShapeDetails) -> Shape {
        Shape { common, details }
    }

    pub fn details(&self) -> &ShapeDetails { &self.details }
    pub fn common(&self) -> &ShapeCommon { &self.common }
}

impl Shape {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        self.details.register_space(&self.common,assets)
    }

    pub fn filter_by_minmax(&self, min_value: f64, max_value: f64) -> Shape {
        if !self.common.coord_system.is_tracking() {
            return self.clone();
        }
        let mut common = self.common.clone();
        let details = self.details.filter_by_minmax(&mut common,min_value,max_value);
        Shape { details, common }
    }

    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,Shape)> where D: ShapeDemerge<X=T> {
        self.details.demerge(&self.common,cat)
    }
    
    pub fn len(&self) -> usize { self.details.len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn remove_nulls(self) -> Shape { self.details.remove_nulls(&self.common) }
}
