use peregrine_data::{CoordinateSystem, DirectColour, CoordinateSystemVariety};
use crate::shape::{heraldry::heraldry::{HeraldryCanvasesUsed, HeraldryScale}, layers::geometry::{GeometryProcessName, GeometryYielder, TrianglesGeometry}};

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
    geometry: TrianglesGeometry,
    shape_category: ShapeCategory
}

fn geometry(coord_system: &CoordinateSystem) -> TrianglesGeometry {
    match coord_system.0 {
        CoordinateSystemVariety::Tracking => TrianglesGeometry::Tracking,
        CoordinateSystemVariety::TrackingWindow => TrianglesGeometry::TrackingWindow,
        CoordinateSystemVariety::Window => TrianglesGeometry::Window,
        CoordinateSystemVariety::Sideways => TrianglesGeometry::Window,
    }
}

impl DrawGroup {
    pub(crate) fn new(coord_system: &CoordinateSystem, shape_category: &ShapeCategory) -> DrawGroup {
        let geometry = geometry(coord_system);
        DrawGroup {
            geometry,
            shape_category: shape_category.clone()
        }
    }

    pub(crate) fn packed_format(&self) -> bool {
        match self.geometry_process_name() {
            GeometryProcessName::Triangles(TrianglesGeometry::Tracking) => true,
            _ => false
        }
    }

    pub(crate) fn shape_category(&self) -> &ShapeCategory { &self.shape_category }
    pub(crate) fn is_tracking(&self) -> bool {
        match self.geometry {
            TrianglesGeometry::Tracking | TrianglesGeometry::TrackingWindow => true,
            _ => false
        }
    }

    pub(crate) fn geometry_process_name(&self) -> GeometryProcessName {
        GeometryProcessName::Triangles(self.geometry.clone())
    }

    pub(crate) fn geometry_yielder(&self) -> GeometryYielder {
        GeometryYielder::new(self.geometry_process_name())
    }
}
