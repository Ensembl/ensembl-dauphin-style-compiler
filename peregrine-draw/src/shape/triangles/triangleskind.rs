use peregrine_data::{Allotment, AllotmentPosition, PositionVariant, SpaceBase, SpaceBaseArea};
use crate::shape::{layers::geometry::{GeometryProcessName, GeometryProgramName}, util::arrayutil::rectangle64};
use super::trianglesyielder::TrackTrianglesYielder;

fn flip(allotment: &Allotment) -> f64 {
    match  match allotment.position() {
        AllotmentPosition::BaseLabel(p,_) => p,
        AllotmentPosition::SpaceLabel(p,_) => p,
        _ => &PositionVariant::HighPriority
    } {
        PositionVariant::HighPriority => 1.,
        PositionVariant::LowPriority => -1.
    }
}

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub enum TrianglesProgramKind {
    Track,
    Base,
    Space,
    Window
}

#[derive(Debug,Clone)]
pub enum TrianglesKind {
    Track,
    Base,
    Space,
    Window(i64)
}

impl TrianglesKind {
    pub(super) fn add_spacebase(&self, point: &SpaceBase<f64>, allotments: &[Allotment], left: f64, width: Option<f64>) -> (Vec<f32>,Vec<f32>) {
        let area = SpaceBaseArea::new(point.clone(),point.clone());
        self.add_spacebase_area(&area,allotments,left,width)
    }

    pub(super) fn to_program_kind(&self) -> TrianglesProgramKind {
        match self {
            TrianglesKind::Track => TrianglesProgramKind::Track,
            TrianglesKind::Base => TrianglesProgramKind::Base,
            TrianglesKind::Space => TrianglesProgramKind::Space,
            TrianglesKind::Window(_) => TrianglesProgramKind::Window
        }
    }

    pub(super) fn add_spacebase_area(&self, area: &SpaceBaseArea<f64>, allotments: &[Allotment], left: f64, width: Option<f64>)-> (Vec<f32>,Vec<f32>) {
        let mut base = vec![];
        let mut delta = vec![];
        let base_width = if width.is_some() { Some(0.) } else { None };
        match self {
            TrianglesKind::Track => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let base_y = allotment.position().offset() as f64;
                    rectangle64(&mut base, *top_left.base-left, base_y, *bottom_right.base-left,base_y,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }
            },
            TrianglesKind::Base => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let flip_y = flip(allotment);
                    rectangle64(&mut base, *top_left.base-left, flip_y, *bottom_right.base-left,flip_y,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }        
            },
            TrianglesKind::Space => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let flip_x = flip(allotment);
                    let base_y = allotment.position().offset() as f64;
                    rectangle64(&mut base, flip_x, base_y, flip_x,base_y,base_width);
                    rectangle64(&mut delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }
            },
            TrianglesKind::Window(_) => {
                for ((top_left,bottom_right),_) in area.iter().zip(allotments.iter().cycle()) {
                    let (mut x0,mut y0,mut x1,mut y1) = (*top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal);
                    let (mut bx0,mut by0,mut bx1,mut by1) = (0.,0.,0.,0.);
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
        let program = GeometryProgramName::Triangles(self.to_program_kind());
        GeometryProcessName::new(program)
    }

    pub(crate) fn geometry_yielder(&self, priority: i8) -> TrackTrianglesYielder {
        TrackTrianglesYielder::new(&self.geometry_process_name(),priority)
    }
}
