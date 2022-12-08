use peregrine_data::{CoordinateSystem};
use crate::shape::{layers::geometry::{TrianglesGeometry}, canvasitem::heraldry::{HeraldryCanvasesUsed, HeraldryScale}};

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum ShapeCategory {
    SolidColour,
    Heraldry(HeraldryCanvasesUsed,HeraldryScale),
    Other
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct DrawGroup {
    coordsystem: CoordinateSystem,
    depth: i8,
    shape_category: ShapeCategory
}

impl DrawGroup {
    pub(crate) fn new(coord_system: &CoordinateSystem,depth: i8, shape_category: &ShapeCategory) -> DrawGroup {
        DrawGroup {
            coordsystem: coord_system.clone(),
            depth,
            shape_category: shape_category.clone()
        }
    }

    pub(crate) fn depth(&self) -> i8 { self.depth }

    pub(crate) fn packed_format(&self) -> bool {
        match self.geometry() {
            TrianglesGeometry::Tracking => true,
            _ => false
        }
    }

    pub(crate) fn geometry(&self) -> TrianglesGeometry {
        match &self.coordsystem {
            CoordinateSystem::Tracking => TrianglesGeometry::Tracking,
            CoordinateSystem::TrackingSpecial => TrianglesGeometry::TrackingSpecial(true),
            CoordinateSystem::TrackingWindow => TrianglesGeometry::TrackingSpecial(false),
            CoordinateSystem::Content => TrianglesGeometry::Window(true),
            _ => TrianglesGeometry::Window(false)
        }    
    }

    pub(crate) fn coord_system(&self) -> &CoordinateSystem { &self.coordsystem }
    pub(crate) fn shape_category(&self) -> &ShapeCategory { &self.shape_category }
}
