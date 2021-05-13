use super::super::core::pingeometry::{ PinProgram };
use super::super::core::fixgeometry::{ FixProgram };
use super::super::core::tapegeometry::{TapeProgram };
use super::super::core::pagegeometry::{ PageProgram };
use super::super::core::wigglegeometry::{WiggleProgram };
use super::super::core::tracktriangles::{ TrackTrianglesProgram };
use crate::webgl::{ SourceInstrs, GLArity, Header, Statement, AttributeProto, ProgramBuilder };
use super::consts::{ PR_LOW };
use web_sys::{ WebGlRenderingContext };
use crate::util::message::Message;

#[derive(Clone)]
pub(crate) enum GeometryProgram {
    Pin(PinProgram),
    Fix(FixProgram),
    Tape(TapeProgram),
    Page(PageProgram),
    Wiggle(WiggleProgram),
    TrackTriangles(TrackTrianglesProgram),
    BaseLabelTriangles(TrackTrianglesProgram),
    SpaceLabelTriangles(TrackTrianglesProgram),
}

#[derive(Clone)]
pub(crate) enum GeometryProgramName { Pin, Fix, Tape, Page, Wiggle, TrackTriangles, BaseLabelTriangles, SpaceLabelTriangles }

impl GeometryProgramName {
    pub const COUNT : usize = 8;

    pub fn get_index(&self) -> usize {
        match self {
            GeometryProgramName::Pin => 0,
            GeometryProgramName::Fix => 1,
            GeometryProgramName::Tape => 2,
            GeometryProgramName::Page => 3,
            GeometryProgramName::Wiggle => 4,
            GeometryProgramName::TrackTriangles => 5,
            GeometryProgramName::BaseLabelTriangles => 6,
            GeometryProgramName::SpaceLabelTriangles => 7,
        }
    }

    pub(super) fn make_geometry_program(&self, builder: &ProgramBuilder) -> Result<GeometryProgram,Message> {
        Ok(match self {
            GeometryProgramName::Page => GeometryProgram::Page(PageProgram::new(builder)?),
            GeometryProgramName::Pin => GeometryProgram::Pin(PinProgram::new(builder)?),
            GeometryProgramName::Tape => GeometryProgram::Tape(TapeProgram::new(builder)?),
            GeometryProgramName::Fix => GeometryProgram::Fix(FixProgram::new(builder)?),
            GeometryProgramName::Wiggle => GeometryProgram::Wiggle(WiggleProgram::new(builder)?),
            GeometryProgramName::TrackTriangles => GeometryProgram::TrackTriangles(TrackTrianglesProgram::new(builder)?),
            GeometryProgramName::BaseLabelTriangles => GeometryProgram::BaseLabelTriangles(TrackTrianglesProgram::new(builder)?),
            GeometryProgramName::SpaceLabelTriangles => GeometryProgram::SpaceLabelTriangles(TrackTrianglesProgram::new(builder)?),
        })
    }

    pub(crate) fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            GeometryProgramName::Pin => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aOrigin.x -uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        1.0 - (aOrigin.y - uStageVpos + aVertexPosition.y) / uSize.y, 
                        0.0, 1.0)")
            ],
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
            GeometryProgramName::Fix => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign.y,
                                        0.0, 1.0)")
            ],
            GeometryProgramName::Tape => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                AttributeProto::new(PR_LOW,GLArity::Scalar,"aVertexSign"),
                AttributeProto::new(PR_LOW,GLArity::Scalar,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aOrigin - uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign,
                        0.0, 1.0)")
            ],
            GeometryProgramName::Page => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                       (- (aVertexPosition.y - uStageVpos) / uSize.y) * aVertexSign.y, 
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
