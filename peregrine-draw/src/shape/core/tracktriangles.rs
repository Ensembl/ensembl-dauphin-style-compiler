use crate::webgl::{AttribHandle, ProcessBuilder, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder};
use peregrine_data::{Allotment, AllotmentPosition, PositionVariant, SpaceBase, SpaceBaseArea};
use super::super::util::arrayutil::rectangle64;
use crate::shape::layers::geometry::{GeometryProcessName, GeometryProgram, GeometryProgramName, GeometryYielder};
use crate::util::message::Message;

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

pub(crate) struct TrackTrianglesYielder {
    geometry_process_name: GeometryProcessName,
    track_triangles: Option<TrackTrianglesProgram>
}

impl<'a> GeometryYielder for TrackTrianglesYielder {
    fn name(&self) -> &GeometryProcessName { &self.geometry_process_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<GeometryProgram,Message> {
        self.geometry_process_name.get_program_name().make_geometry_program(builder)
    }

    fn set(&mut self, program: &GeometryProgram) -> Result<(),Message> {
        self.track_triangles = Some(match program {
            GeometryProgram::BaseLabelTriangles(t) => t,
            GeometryProgram::SpaceLabelTriangles(t) => t,
            GeometryProgram::TrackTriangles(t) => t,
            GeometryProgram::WindowTriangles(t) => t,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: tracktriangles")))? }
        }.clone());
        Ok(())
    }
}

impl TrackTrianglesYielder {
    pub(crate) fn new(geometry_process_name: &GeometryProcessName) -> TrackTrianglesYielder {
        TrackTrianglesYielder {
            geometry_process_name: geometry_process_name.clone(),
            track_triangles: None
        }
    }

    pub(crate) fn track_triangles(&self) -> Result<&TrackTrianglesProgram,Message> {
        self.track_triangles.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}

#[derive(Debug)]
pub enum TrianglesKind {
    Track,
    Base,
    Space,
    Window(i64)
}

impl TrianglesKind {
    fn add_spacebase(&self, point: &SpaceBase<f64>, allotments: &[Allotment], left: f64, width: Option<f64>) -> (Vec<f32>,Vec<f32>) {
        let area = SpaceBaseArea::new(point.clone(),point.clone());
        self.add_spacebase_area(&area,allotments,left,width)
    }

    fn add_spacebase_area(&self, area: &SpaceBaseArea<f64>, allotments: &[Allotment], left: f64, width: Option<f64>)-> (Vec<f32>,Vec<f32>) {
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
        let (program,priority) = match self {
            TrianglesKind::Track => (GeometryProgramName::TrackTriangles,0),
            TrianglesKind::Base => (GeometryProgramName::BaseLabelTriangles,0),
            TrianglesKind::Space => (GeometryProgramName::SpaceLabelTriangles,0),
            TrianglesKind::Window(p) => (GeometryProgramName::WindowTriangles,*p)
        };
        GeometryProcessName::new(program,priority)
    }

    pub(crate) fn geometry_yielder(&self) -> TrackTrianglesYielder {
        TrackTrianglesYielder::new(&self.geometry_process_name())
    }
}

#[derive(Clone)]
pub struct TrackTrianglesProgram {
    base: AttribHandle,
    delta: AttribHandle,
    origin_base: Option<AttribHandle>,
    origin_delta: Option<AttribHandle>,
}

impl TrackTrianglesProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TrackTrianglesProgram,Message> {
        Ok(TrackTrianglesProgram {
            base: builder.get_attrib_handle("aBase")?,
            delta: builder.get_attrib_handle("aDelta")?,
            origin_base: builder.try_get_attrib_handle("aOriginBase"),
            origin_delta: builder.try_get_attrib_handle("aOriginDelta")
        })
    }

    fn add_rectangles_real(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea<f64>, allotments: &[Allotment], left: f64,width: Option<f64>, kind: &TrianglesKind)-> Result<ProcessStanzaElements,Message> {
        let indexes = if width.is_some() {
            vec![0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1]
        } else {
            vec![0,3,1,2,0,3]
        };
        let mut elements = builder.get_stanza_builder().make_elements(area.len(),&indexes)?;
        let (base,delta) = kind.add_spacebase_area(area,allotments,left,width);
        // XXX only if needed
        let (origin_base,origin_delta) = kind.add_spacebase(&area.middle_base(),allotments,left,width);
        elements.add(&self.delta,delta,2)?;
        elements.add(&self.base,base,2)?;
        if let Some(origin_base_handle) = &self.origin_base {
            elements.add(origin_base_handle,origin_base,2)?;
        }
        if let Some(origin_delta_handle) = &self.origin_delta {
            elements.add(origin_delta_handle,origin_delta,2)?;
        }
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea<f64>, allotments: &[Allotment], left: f64, hollow: bool, kind: &TrianglesKind)-> Result<ProcessStanzaElements,Message> {
        Ok(match hollow {
            true => self.add_rectangles_real(builder,area,allotments,left,Some(1.),kind)?,
            false => self.add_rectangles_real(builder,area,allotments,left,None,kind)?,
        })
    }
}
