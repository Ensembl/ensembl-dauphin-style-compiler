use super::super::layers::layer::{ Layer };
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{AttribHandle, ProcessBuilder, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder};
use peregrine_data::{Allotment, AllotmentPosition, PositionVariant, SpaceBase, SpaceBaseArea};
use super::super::util::arrayutil::rectangle64;
use crate::shape::layers::geometry::GeometryProgramName;
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

pub enum TrianglesKind {
    Track,
    Base,
    Space
}

impl TrianglesKind {
    fn add_spacebase(&self, point: &SpaceBase, allotments: &[Allotment], left: f64, width: Option<f64>) -> (Vec<f32>,Vec<f32>) {
        let area = SpaceBaseArea::new(point.clone(),point.clone());
        self.add_spacebase_area(&area,allotments,left,width)
    }

    fn add_spacebase_area(&self, area: &SpaceBaseArea, allotments: &[Allotment], left: f64, width: Option<f64>)-> (Vec<f32>,Vec<f32>) {
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
            }
        }
        (base,delta)
    }

    pub(crate) fn get_process(&self, layer: &mut Layer, patina: &PatinaProcessName) -> Result<TrackTrianglesProgram,Message> {
        Ok(match self {
            TrianglesKind::Track => layer.get_track_triangles(patina)?,
            TrianglesKind::Base => layer.get_base_label_triangles(patina)?,
            TrianglesKind::Space => layer.get_space_label_triangles(patina)?,
        })
    }

    pub(crate) fn geometry_program_name(&self) -> GeometryProgramName {
        match self {
            TrianglesKind::Track => GeometryProgramName::TrackTriangles,
            TrianglesKind::Base => GeometryProgramName::BaseLabelTriangles,
            TrianglesKind::Space => GeometryProgramName::SpaceLabelTriangles
        }
    }
}

#[derive(Clone)]
pub struct TrackTrianglesProgram {
    base: AttribHandle,
    delta: AttribHandle,
    origin_base: AttribHandle,
    origin_delta: AttribHandle,
}

impl TrackTrianglesProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TrackTrianglesProgram,Message> {
        Ok(TrackTrianglesProgram {
            base: builder.get_attrib_handle("aBase")?,
            delta: builder.get_attrib_handle("aDelta")?,
            origin_base: builder.get_attrib_handle("aOriginBase")?,
            origin_delta: builder.get_attrib_handle("aOriginDelta")?
        })
    }

    fn add_rectangles_real(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea, allotments: &[Allotment], left: f64,width: Option<f64>, kind: &TrianglesKind)-> Result<ProcessStanzaElements,Message> {
        let indexes = if width.is_some() {
            vec![0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1]
        } else {
            vec![0,3,1,2,0,3]
        };
        let mut elements = builder.get_stanza_builder().make_elements(area.len(),&indexes)?;
        let (base,delta) = kind.add_spacebase_area(area,allotments,left,width);
        let (origin_base,origin_delta) = kind.add_spacebase(&area.top_left(),allotments,left,width);
        elements.add(&self.base,base,2)?;
        elements.add(&self.delta,delta,2)?;
        elements.add(&self.origin_base,origin_base,2)?;
        elements.add(&self.origin_delta,origin_delta,2)?;
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea, allotments: &[Allotment], left: f64, hollow: bool, kind: &TrianglesKind)-> Result<ProcessStanzaElements,Message> {
        Ok(match hollow {
            true => self.add_rectangles_real(builder,area,allotments,left,Some(1.),kind)?,
            false => self.add_rectangles_real(builder,area,allotments,left,None,kind)?,
        })
    }
}
