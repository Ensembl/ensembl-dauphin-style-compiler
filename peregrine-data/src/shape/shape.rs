use std::hash::Hash;
use super::metadata::AllotmentMetadataEntry;
use super::polygonshape::PolygonShape;
use super::emptyshape::EmptyShape;
use super::imageshape::ImageShape;
use super::rectangleshape::RectangleShape;
use super::textshape::TextShape;
use super::wiggleshape::WiggleShape;
use crate::Assets;
use crate::AuxLeaf;
use crate::Colour;
use crate::CoordinateSystem;
use crate::DataMessage;
use crate::DrawnType;
use crate::LeafRequest;
use crate::Patina;
use crate::SpaceBaseArea;
use crate::allotment::core::rangeused::RangeUsed;
use crate::allotment::leafs::anchored::AnchoredLeaf;
use crate::allotment::leafs::floating::FloatingLeaf;

pub trait ShapeDemerge {
    type X: Hash + PartialEq + Eq;

    fn categorise(&self, coord_system: &CoordinateSystem, depth: i8) -> Self::X;
    
    fn categorise_with_colour(&self, coord_system: &CoordinateSystem, depth: i8, _variety: &DrawnType, _colour: &Colour) -> Self::X {
        self.categorise(coord_system,depth)
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Shape<A> {
    Text(TextShape<A>),
    Polygon(PolygonShape<A>),
    Image(ImageShape<A>),
    Wiggle(WiggleShape<A>),
    Rectangle(RectangleShape<A>),
    Empty(EmptyShape<A>)
}

/* -> A shape with reference only to the surroundings of its own carriage -> */
pub(crate) type FloatingShape = Shape<FloatingLeaf>;

/* -> A completely placed shape, ready to draw */
pub type DrawingShape = Shape<AuxLeaf>;

impl<A> Clone for Shape<A> where A: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Text(arg0) => Self::Text(arg0.clone()),
            Self::Image(arg0) => Self::Image(arg0.clone()),
            Self::Wiggle(arg0) => Self::Wiggle(arg0.clone()),
            Self::Rectangle(arg0) => Self::Rectangle(arg0.clone()),
            Self::Empty(arg0) => Self::Empty(arg0.clone()),
            Self::Polygon(arg0) => Self::Polygon(arg0.clone())
        }
    }
}

impl<A> Shape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> Shape<B> where F: FnMut(&A) -> B {
        match self {
            Self::Text(arg0) => Shape::<B>::Text(arg0.map_new_allotment(cb)),
            Self::Image(arg0) => Shape::<B>::Image(arg0.map_new_allotment(cb)),
            Self::Wiggle(arg0) => Shape::<B>::Wiggle(arg0.map_new_allotment(cb)),
            Self::Rectangle(arg0) => Shape::<B>::Rectangle(arg0.map_new_allotment(cb)),
            Self::Empty(arg0) => Shape::<B>::Empty(arg0.map_new_allotment(cb)),
            Self::Polygon(arg0) => Shape::<B>::Polygon(arg0.map_new_allotment(cb)),
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Shape::Rectangle(shape) => shape.len(),
            Shape::Text(shape) => shape.len(),
            Shape::Image(shape) => shape.len(),
            Shape::Wiggle(shape) => shape.len(),
            Shape::Empty(shape) => shape.len(),
            Shape::Polygon(shape) => shape.len()
        }
    }
}

impl Shape<LeafRequest> {
    pub fn base_filter(&self, min: f64, max: f64) -> Shape<LeafRequest> {
        match self {
            Shape::Rectangle(shape) => Shape::Rectangle(shape.base_filter(min,max)),
            Shape::Text(shape) => Shape::Text(shape.base_filter(min,max)),
            Shape::Image(shape) => Shape::Image(shape.base_filter(min,max)),
            Shape::Wiggle(shape) => Shape::Wiggle(shape.base_filter(min,max)),
            Shape::Empty(shape) => Shape::Empty(shape.base_filter(min, max)),
            Shape::Polygon(shape) => Shape::Polygon(shape.base_filter(min, max))
        }
    }
}

impl Shape<AuxLeaf> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,Shape<AuxLeaf>)> where D: ShapeDemerge<X=T> {
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
            Shape::Rectangle(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)|
                    (x,Shape::Rectangle(details))
                ).collect()
            },
            Shape::Polygon(shape) => {
                return shape.demerge(cat).drain(..).map(|(x,details)|
                    (x,Shape::Polygon(details))
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

impl Shape<AnchoredLeaf> {
    pub fn make(&self) -> Vec<Shape<AuxLeaf>> {
        match self {
            Shape::Polygon(shape) => shape.make().drain(..).map(|x| Shape::Polygon(x)).collect(),
            Shape::Rectangle(shape) => shape.make().drain(..).map(|x| Shape::Rectangle(x)).collect(),
            Shape::Text(shape) => shape.make().drain(..).map(|x| Shape::Text(x)).collect(),
            Shape::Image(shape) => shape.make().drain(..).map(|x| Shape::Image(x)).collect(),
            Shape::Wiggle(shape) => shape.make().drain(..).map(|x| Shape::Wiggle(x)).collect(),
            Shape::Empty(shape) => shape.make().drain(..).map(|x| Shape::Empty(x)).collect(),
        }
    }
}

fn register_space_area(shape: &SpaceBaseArea<f64,LeafRequest>) {
    for (top_left,bottom_right) in shape.iter() {
        top_left.allotment.shape_bounds(|allotment| {
            allotment.merge_base_range(&RangeUsed::Part(*top_left.base,*bottom_right.base));
            allotment.merge_pixel_range(&RangeUsed::Part(*top_left.tangent,*bottom_right.tangent));
            allotment.merge_height(top_left.normal.ceil());
            allotment.merge_height(bottom_right.normal.ceil());
        });
    }
}

fn register_patina(shape: &RectangleShape<LeafRequest>) {
    let allotments = shape.area().top_left().allotments();
    if let Patina::Metadata(key,values) = shape.patina() {
        for (allotment,entry) in allotments.zip(values,|leaf,(id,value)| {
            (leaf.clone(),AllotmentMetadataEntry::new(leaf.name(),key,id,value))
        }).iter(shape.len()).unwrap() {
            allotment.shape_bounds(|bounds| {
                bounds.merge_metadata(entry.clone());
            });
        }
    }
}

impl Shape<LeafRequest> {
    pub fn register_space(&self, assets: &Assets) -> Result<(),DataMessage> {
        match &self {
            Shape::Rectangle(shape) => {
                register_space_area(shape.area());
                register_patina(shape);
            },
            Shape::Empty(area) => {
                register_space_area(area.area());
            },
            Shape::Text(shape) => {
                shape.register_space();
            },
            Shape::Polygon(shape) => {
                shape.register_space();
            },
            Shape::Image(shape) => {
                for (position,asset_name) in shape.position().iter().zip(shape.iter_names()) {
                    position.allotment.shape_bounds(|allotment| {
                        allotment.merge_base_range(&RangeUsed::Part(*position.base,*position.base+1.));
                        if let Some(asset) = assets.get(Some(&shape.channel()),asset_name) {
                            if let Some(height) = asset.metadata_u32("height") {
                                allotment.merge_height((position.normal + (height as f64)).ceil());
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
                    allotment.shape_bounds(|allotment| {
                        allotment.merge_base_range(&RangeUsed::All);
                        allotment.merge_height(shape.plotter().0);
                    });
                }
            }
        }
        Ok(())
    }
}
