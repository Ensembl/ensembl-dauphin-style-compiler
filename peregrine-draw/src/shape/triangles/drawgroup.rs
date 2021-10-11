use peregrine_data::{Allotment, CoordinateSystem, SpaceBase, SpaceBaseArea};
use crate::shape::{layers::geometry::{GeometryProcessName, GeometryProgramName}, util::arrayutil::rectangle64};
use super::trianglesyielder::TrackTrianglesYielder;
use web_sys::console;


#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct DrawGroup {
    coord_system: CoordinateSystem,
    depth: i8
}

impl DrawGroup {
    pub(crate) fn new(coord_system: &CoordinateSystem, depth: i8) -> DrawGroup {
        DrawGroup {
            coord_system: coord_system.clone(),
            depth
        }
    }

    pub(super) fn add_spacebase(&self, point: &SpaceBase<f64>, allotments: &[Allotment], left: f64, width: Option<f64>) -> (Vec<f32>,Vec<f32>) {
        let area = SpaceBaseArea::new(point.clone(),point.clone());
        self.add_spacebase_area(&area,allotments,left,width)
    }

    pub(super) fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }

    pub(super) fn add_spacebase_area(&self, area: &SpaceBaseArea<f64>, allotments: &[Allotment], left: f64, width: Option<f64>)-> (Vec<f32>,Vec<f32>) {
        let mut base = vec![];
        let mut delta = vec![];
        let base_width = if width.is_some() { Some(0.) } else { None };
        match self.coord_system() {
            CoordinateSystem::Tracking | CoordinateSystem::TrackingBottom => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let top_left = allotment.transform_spacebase(&top_left);
                    let bottom_right = allotment.transform_spacebase(&bottom_right);
                    rectangle64(&mut base, top_left.base-left, 0., bottom_right.base-left,0.,base_width);
                    rectangle64(&mut delta, top_left.tangent,top_left.normal,bottom_right.tangent,bottom_right.normal,width);
                }
            },
            CoordinateSystem::SidewaysLeft => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    rectangle64(&mut base, *top_left.base-left, 0., *bottom_right.base-left,0.,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }
            },
            CoordinateSystem::SidewaysRight => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let top_left = allotment.transform_spacebase(&top_left);
                    let bottom_right = allotment.transform_spacebase(&bottom_right);
                    let (mut x0,mut y0,mut x1,mut y1) = (top_left.tangent,top_left.normal,bottom_right.tangent,bottom_right.normal);
                    let (mut bx0,mut by0,mut bx1,mut by1) = (top_left.base,0.,bottom_right.base,0.);
                    if x0 < 0. { x0 = -x0-1.; bx0 = 1.; }
                    if y0 < 0. { y0 = -y0-1.; by0 = 1.; }
                    if x1 < 0. { x1 = -x1-1.; bx1 = 1.; }
                    if y1 < 0. { y1 = -y1-1.; by1 = 1.; }
                    rectangle64(&mut base, bx0,by0, bx1,by1,base_width);
                    rectangle64(&mut delta, x0,y0,x1,y1,width);
                }
            },
            CoordinateSystem::Window | CoordinateSystem::WindowBottom => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let top_left = allotment.transform_spacebase(&top_left);
                    let bottom_right = allotment.transform_spacebase(&bottom_right);
                    let (mut x0,mut y0,mut x1,mut y1) = (top_left.tangent,top_left.normal,bottom_right.tangent,bottom_right.normal);
                    let (mut bx0,mut by0,mut bx1,mut by1) = (top_left.base,0.,bottom_right.base,0.);
                    if x0 < 0. { x0 = -x0-1.; bx0 = 1.; }
                    if y0 < 0. { y0 = -y0-1.; by0 = 1.; }
                    if x1 < 0. { x1 = -x1-1.; bx1 = 1.; }
                    if y1 < 0. { y1 = -y1-1.; by1 = 1.; }
                    rectangle64(&mut base, bx0,by0, bx1,by1,base_width);
                    rectangle64(&mut delta, x0,y0,x1,y1,width);
                }
            }
        }
        (base,delta)
    }

    pub(crate) fn geometry_process_name(&self) -> GeometryProcessName {
        let program = GeometryProgramName::Triangles(self.coord_system());
        GeometryProcessName::new(program)
    }

    pub(crate) fn geometry_yielder(&self) -> TrackTrianglesYielder {
        TrackTrianglesYielder::new(&self.geometry_process_name(),self.depth())
    }

    pub fn depth(&self) -> i8 { self.depth }
}
