use peregrine_data::{CoordinateSystem, DirectColour};
use crate::shape::{heraldry::heraldry::{HeraldryCanvasesUsed, HeraldryScale}, layers::geometry::{GeometryProcessName, GeometryYielder, TrianglesGeometry, TrianglesTransform}};

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum ShapeCategory {
    SolidColour,
    SpotColour(DirectColour),
    Heraldry(HeraldryCanvasesUsed,HeraldryScale),
    Other
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct DrawGroup {
    coord_system: CoordinateSystem,
    depth: i8,
    shape_category: ShapeCategory
}

impl DrawGroup {
    pub(crate) fn new(coord_system: &CoordinateSystem, depth: i8, shape_category: &ShapeCategory) -> DrawGroup {
        DrawGroup {
            coord_system: coord_system.clone(),
            depth,
            shape_category: shape_category.clone()
        }
    }

    pub(super) fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }
    pub(crate) fn shape_category(&self) -> &ShapeCategory { &self.shape_category }

    pub(crate) fn geometry_process_name(&self) -> GeometryProcessName {
        let (system,transform) = match self.coord_system() {
            CoordinateSystem::Tracking => (TrianglesGeometry::Tracking,TrianglesTransform::NegativeY),
            CoordinateSystem::TrackingBottom => (TrianglesGeometry::Tracking,TrianglesTransform::Identity),
            CoordinateSystem::Window => (TrianglesGeometry::Window,TrianglesTransform::NegativeY),
            CoordinateSystem::WindowBottom => (TrianglesGeometry::Window,TrianglesTransform::Identity),
            CoordinateSystem::SidewaysLeft => (TrianglesGeometry::Sideways,TrianglesTransform::NegativeY),
            CoordinateSystem::SidewaysRight =>  (TrianglesGeometry::Sideways,TrianglesTransform::NegativeXY),
        };
        GeometryProcessName::Triangles(system,transform)
    }

    pub(crate) fn geometry_yielder(&self) -> GeometryYielder {
        GeometryYielder::new(self.geometry_process_name(),self.depth())
    }

    pub fn depth(&self) -> i8 { self.depth }
}
