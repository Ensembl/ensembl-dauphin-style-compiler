use super::super::layers::layer::{ Layer };
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{AttribHandle, GPUSpec, Process, ProcessBuilder, ProcessStanzaAddable, ProcessStanzaElements, Program, ProgramBuilder};
use peregrine_data::{ SpaceBaseArea, Allotment, Patina, AllotmentPosition, PositionVariant };
use super::super::util::arrayutil::{ plain_rectangle, hollow_rectangle, rectangle };
use crate::stage::stage::{ ReadStage };
use super::geometrydata::GeometryData;
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
    fn add(&self, base:&mut Vec<f64>, delta: &mut Vec<f64>, area: &SpaceBaseArea, allotments: &[Allotment], left: f64, width: Option<f64>) {
        match self {
            TrianglesKind::Track => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let base_y = allotment.position().offset() as f64;
                    rectangle(base, *top_left.base-left, base_y, *bottom_right.base-left,base_y,width);
                    rectangle(delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }
            },
            TrianglesKind::Base => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let flip_y = flip(allotment);
                    rectangle(base, *top_left.base-left, flip_y, *bottom_right.base-left,flip_y,width);
                    rectangle(delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }        
            },
            TrianglesKind::Space => {
                for ((top_left,bottom_right),allotment) in area.iter().zip(allotments.iter().cycle()) {
                    let flip_x = flip(allotment);
                    let base_y = allotment.position().offset() as f64;
                    rectangle(base, flip_x, base_y, flip_x,base_y,width);
                    rectangle(delta, *top_left.tangent,*top_left.normal,*bottom_right.tangent,*bottom_right.normal,width);
                }        
            }
        }
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
}

impl TrackTrianglesProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TrackTrianglesProgram,Message> {
        Ok(TrackTrianglesProgram {
            base: builder.get_attrib_handle("aBase")?,
            delta: builder.get_attrib_handle("aDelta")?
        })
    }

    fn add_rectangles_real(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea, allotments: &[Allotment], left: f64,width: Option<f64>, kind: TrianglesKind)-> Result<ProcessStanzaElements,Message> {
        let indexes = if width.is_some() {
            vec![0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1]
        } else {
            vec![0,3,1,2,0,3]
        };
        let mut elements = builder.get_stanza_builder().make_elements(area.len(),&indexes)?;
        let mut base = vec![];
        let mut delta= vec![];
        kind.add(&mut base, &mut delta,area,allotments,left,width);
        elements.add(&self.base,base,2)?;
        elements.add(&self.delta,delta,2)?;
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, builder: &mut ProcessBuilder, area: &SpaceBaseArea, allotments: &[Allotment], left: f64, hollow: bool, kind: TrianglesKind)-> Result<Option<ProcessStanzaElements>,Message> {
        Ok(match hollow {
            true => Some(self.add_rectangles_real(builder,area,allotments,left,Some(1.),kind)?),
            false => Some(self.add_rectangles_real(builder,area,allotments,left,None,kind)?),
        })
    }
}
