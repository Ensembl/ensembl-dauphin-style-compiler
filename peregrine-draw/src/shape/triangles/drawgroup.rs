use peregrine_data::{CoordinateSystem};
use crate::shape::{layers::geometry::{GeometryProcessName, GeometryYielder, TrianglesGeometry, TrianglesTransform}};

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct DrawGroup {
    coord_system: CoordinateSystem,
    depth: i8
}

impl DrawGroup {
    pub(crate) fn new(coord_system: &CoordinateSystem, depth: i8) -> DrawGroup {
        DrawGroup {
            coord_system: coord_system.clone(),
            depth,
        }
    }

    pub(super) fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }

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
