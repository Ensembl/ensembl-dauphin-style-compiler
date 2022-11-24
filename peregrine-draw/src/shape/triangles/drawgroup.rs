use peregrine_data::{DirectColour, CoordinateSystem};
use crate::shape::{layers::geometry::{GeometryProcessName, GeometryYielder, TrianglesGeometry}, canvasitem::heraldry::{HeraldryCanvasesUsed, HeraldryScale}};

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
    coordsystem: CoordinateSystem,
    shape_category: ShapeCategory
}

fn geometry(coord_system: &CoordinateSystem) -> TrianglesGeometry {
    match coord_system {
        CoordinateSystem::Tracking => TrianglesGeometry::Tracking,
        CoordinateSystem::TrackingSpecial => TrianglesGeometry::TrackingSpecial(true),
        CoordinateSystem::TrackingWindow => TrianglesGeometry::TrackingSpecial(false),
        CoordinateSystem::Content => TrianglesGeometry::Window(true),
        _ => TrianglesGeometry::Window(false)
    }
}

impl DrawGroup {
    pub(crate) fn new(coord_system: &CoordinateSystem, shape_category: &ShapeCategory) -> DrawGroup {
        DrawGroup {
            coordsystem: coord_system.clone(),
            shape_category: shape_category.clone()
        }
    }

    pub(crate) fn packed_format(&self) -> bool {
        match self.geometry_process_name() {
            GeometryProcessName::Triangles(TrianglesGeometry::Tracking) => true,
            _ => false
        }
    }

    pub(crate) fn coord_system(&self) -> &CoordinateSystem { &self.coordsystem }
    pub(crate) fn shape_category(&self) -> &ShapeCategory { &self.shape_category }

    pub(crate) fn geometry_process_name(&self) -> GeometryProcessName {
        GeometryProcessName::Triangles(geometry(self.coord_system()).clone())
    }

    pub(crate) fn geometry_yielder(&self) -> GeometryYielder {
        GeometryYielder::new(self.geometry_process_name())
    }
}
