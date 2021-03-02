use super::super::core::pingeometry::{ PinGeometry, PinProgram };
use super::super::core::fixgeometry::{ FixGeometry, FixProgram };
use super::super::core::tapegeometry::{ TapeGeometry, TapeProgram };
use super::super::core::pagegeometry::{ PageGeometry, PageProgram };
use super::super::core::wigglegeometry::{ WiggleGeometry, WiggleProgram };
use super::patina::PatinaProcessName;
use crate::webgl::{ ProtoProcess, SourceInstrs, Attribute, GLArity, Header, Statement, Program };
use super::consts::{ PR_LOW };
use web_sys::{ WebGlRenderingContext };

pub(crate) enum GeometryProgram {
    Pin(PinProgram),
    Fix(FixProgram),
    Tape(TapeProgram),
    Page(PageProgram),
    Wiggle(WiggleProgram)
}

impl GeometryProgram {
    pub(super) fn make_geometry_process(&self, process: &ProtoProcess, skin: &PatinaProcessName) -> anyhow::Result<GeometryProcess> {
        Ok(match self {
            GeometryProgram::Pin(v) => GeometryProcess::Pin(PinGeometry::new(process,skin,v)?),
            GeometryProgram::Fix(v)=> GeometryProcess::Fix(FixGeometry::new(process,skin,v)?),
            GeometryProgram::Tape(v) => GeometryProcess::Tape(TapeGeometry::new(process,skin,v)?),
            GeometryProgram::Page(v) => GeometryProcess::Page(PageGeometry::new(process,skin,v)?),
            GeometryProgram::Wiggle(v) => GeometryProcess::Wiggle(WiggleGeometry::new(process,skin,v)?),            
        })
    }
}

pub(crate) enum GeometryProgramName { Pin, Fix, Tape, Page, Wiggle }

impl GeometryProgramName {
    pub const COUNT : usize = 5;

    pub fn get_index(&self) -> usize {
        match self {
            GeometryProgramName::Pin => 0,
            GeometryProgramName::Fix => 1,
            GeometryProgramName::Tape => 2,
            GeometryProgramName::Page => 3,
            GeometryProgramName::Wiggle => 4,
        }
    }

    pub(super) fn make_geometry_program(&self, program: &Program) -> anyhow::Result<GeometryProgram> {
        Ok(match self {
            GeometryProgramName::Page => GeometryProgram::Page(PageProgram::new(program)?),
            GeometryProgramName::Pin => GeometryProgram::Pin(PinProgram::new(program)?),
            GeometryProgramName::Tape => GeometryProgram::Tape(TapeProgram::new(program)?),
            GeometryProgramName::Fix => GeometryProgram::Fix(FixProgram::new(program)?),
            GeometryProgramName::Wiggle => GeometryProgram::Wiggle(WiggleProgram::new(program)?)
        })
    }

    pub fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            GeometryProgramName::Pin => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aOrigin.x -uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        - (aOrigin.y - uStageVpos + aVertexPosition.y) / uSize.y, 
                        0.0, 1.0)")
            ],
            GeometryProgramName::Fix => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign.y,
                                        0.0, 1.0)")
            ],
            GeometryProgramName::Tape => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Scalar,"aVertexSign"),
                Attribute::new(PR_LOW,GLArity::Scalar,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aOrigin - uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign,
                        0.0, 1.0)")
            ],
            GeometryProgramName::Page => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                       (- (aVertexPosition.y - uStageVpos) / uSize.y) * aVertexSign.y, 
                                       0.0, 1.0)")
            ],
            GeometryProgramName::Wiggle => vec![
                Header::new(WebGlRenderingContext::TRIANGLE_STRIP),
                Attribute::new(PR_LOW,GLArity::Vec2,"aData"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aData.x -uStageHpos) * uStageZoom,
                        - (aData.y - uStageVpos) / uSize.y, 
                        0.0, 1.0)")
            ]
        })
    }
}

pub(super) enum GeometryProcess {
    Pin(PinGeometry),
    Fix(FixGeometry),
    Tape(TapeGeometry),
    Page(PageGeometry),
    Wiggle(WiggleGeometry)
}

pub enum GeometryProcessName { Pin, Fix, Tape, Page, Wiggle }

impl GeometryProcessName {
    pub(super) fn get_program_name(&self) -> GeometryProgramName {
        match self {
            GeometryProcessName::Pin => GeometryProgramName::Pin,
            GeometryProcessName::Fix => GeometryProgramName::Fix,
            GeometryProcessName::Tape => GeometryProgramName::Tape,
            GeometryProcessName::Page => GeometryProgramName::Page,
            GeometryProcessName::Wiggle => GeometryProgramName::Wiggle
        }
    }
}