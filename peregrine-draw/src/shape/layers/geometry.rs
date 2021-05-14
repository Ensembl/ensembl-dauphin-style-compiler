use super::super::core::wigglegeometry::{WiggleProgram };
use super::super::core::tracktriangles::{ TrackTrianglesProgram };
use crate::webgl::{ SourceInstrs, GLArity, Header, Statement, AttributeProto, ProgramBuilder };
use super::consts::{ PR_LOW };
use web_sys::{ WebGlRenderingContext };
use crate::util::message::Message;

#[derive(Clone)]
pub(crate) enum GeometryProgram {
    Wiggle(WiggleProgram),
    TrackTriangles(TrackTrianglesProgram),
    BaseLabelTriangles(TrackTrianglesProgram),
    SpaceLabelTriangles(TrackTrianglesProgram),
}

#[derive(Clone)]
pub(crate) enum GeometryProgramName { Wiggle, TrackTriangles, BaseLabelTriangles, SpaceLabelTriangles }

impl GeometryProgramName {
    pub const COUNT : usize = 4;

    pub fn get_index(&self) -> usize {
        match self {
            GeometryProgramName::Wiggle => 0,
            GeometryProgramName::TrackTriangles => 1,
            GeometryProgramName::BaseLabelTriangles => 2,
            GeometryProgramName::SpaceLabelTriangles => 3,
        }
    }

    pub(super) fn make_geometry_program(&self, builder: &ProgramBuilder) -> Result<GeometryProgram,Message> {
        Ok(match self {
            GeometryProgramName::Wiggle => GeometryProgram::Wiggle(WiggleProgram::new(builder)?),
            GeometryProgramName::TrackTriangles => GeometryProgram::TrackTriangles(TrackTrianglesProgram::new(builder)?),
            GeometryProgramName::BaseLabelTriangles => GeometryProgram::BaseLabelTriangles(TrackTrianglesProgram::new(builder)?),
            GeometryProgramName::SpaceLabelTriangles => GeometryProgram::SpaceLabelTriangles(TrackTrianglesProgram::new(builder)?),
        })
    }

    pub(crate) fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            GeometryProgramName::TrackTriangles => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aBase.x -uStageHpos) * uStageZoom + 
                                    aDelta.x / uSize.x,
                        1.0 - (aBase.y - uStageVpos + aDelta.y) / uSize.y, 
                        0.0, 1.0)")
            ],
            GeometryProgramName::BaseLabelTriangles => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aBase.x -uStageHpos) * uStageZoom + 
                                    aDelta.x / uSize.x,
                        (1.0 - aDelta.y / uSize.y) * aBase.y, 
                        0.0, 1.0)")
            ],
            GeometryProgramName::SpaceLabelTriangles => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aDelta.x / uSize.x - 1.0) * aBase.x, 
                        1.0 - (aBase.y - uStageVpos + aDelta.y) / uSize.y, 
                        0.0, 1.0)")
            ],
            GeometryProgramName::Wiggle => vec![
                Header::new(WebGlRenderingContext::TRIANGLE_STRIP),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aData"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aData.x -uStageHpos) * uStageZoom,
                        - (aData.y - uStageVpos) / uSize.y, 
                        0.0, 1.0)")
            ]
        })
    }
}
