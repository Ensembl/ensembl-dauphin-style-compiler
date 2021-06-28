use super::super::core::wigglegeometry::{WiggleProgram };
use crate::shape::layers::consts::PR_DEF;
use crate::shape::triangles::triangleskind::TrianglesProgramKind;
use crate::shape::triangles::trianglesprogram::TrackTrianglesProgram;
use crate::util::enummap::{Enumerable, EnumerableKey};
use crate::webgl::{AttributeProto, Conditional, Declaration, GLArity, Header, ProgramBuilder, SourceInstrs, Statement, Varying};
use super::consts::{ PR_LOW };
use web_sys::{ WebGlRenderingContext };
use crate::util::message::Message;

#[derive(Clone)]
pub(crate) enum GeometryProgram {
    Wiggle(WiggleProgram),
    Triangles(TrianglesProgramKind,TrackTrianglesProgram),
}

pub(crate) trait GeometryYielder {
    fn name(&self) -> &GeometryProcessName;
    fn set(&mut self, program: &GeometryProgram) -> Result<(),Message>;
}

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub(crate) enum GeometryProgramName { Wiggle, Triangles(TrianglesProgramKind) }

impl EnumerableKey for GeometryProgramName {
    fn enumerable(&self) -> Enumerable {
        Enumerable(match self {
            GeometryProgramName::Wiggle => 0,
            GeometryProgramName::Triangles(TrianglesProgramKind::Track) => 1,
            GeometryProgramName::Triangles(TrianglesProgramKind::Base) => 2,
            GeometryProgramName::Triangles(TrianglesProgramKind::Space) => 3,
            GeometryProgramName::Triangles(TrianglesProgramKind::Window) => 4,
        },5)
    }
}

impl GeometryProgramName {
    pub(crate) fn make_geometry_program(&self, builder: &ProgramBuilder) -> Result<GeometryProgram,Message> {
        Ok(match self {
            GeometryProgramName::Wiggle => GeometryProgram::Wiggle(WiggleProgram::new(builder)?),
            GeometryProgramName::Triangles(kind) => GeometryProgram::Triangles(kind.clone(),TrackTrianglesProgram::new(builder)?),
        })
    }

    pub(crate) fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            GeometryProgramName::Triangles(TrianglesProgramKind::Track) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * vec4(
                            (base.x -uStageHpos) * uStageZoom + 
                                        delta.x / uSize.x,
                            1.0 - (base.y - uStageVpos + delta.y) / uSize.y, 
                            0.0, 1.0);                      
                    }
                "),
                Statement::new_vertex("
                    gl_Position = transform(aBase,aDelta);
                "),
                Conditional::new("need-origin",vec![
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginBase"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginDelta"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),    
                    Statement::new_vertex("
                        vec4 x = transform(aOriginBase,aOriginDelta);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
            ],
            GeometryProgramName::Triangles(TrianglesProgramKind::Base) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * vec4(
                            (base.x -uStageHpos) * uStageZoom + 
                                        delta.x / uSize.x,
                            (1.0 - delta.y / uSize.y) * base.y, 
                            0.0, 1.0)
                    }
                "),
                Statement::new_vertex("
                    gl_Position = transform(aBase,aDelta)
                "),
                Conditional::new("need-origin",vec![
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginBase"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginDelta"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),    
                    Statement::new_vertex("
                        vec4 x = transform(aOriginBase,aOriginDelta);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
            ],
            GeometryProgramName::Triangles(TrianglesProgramKind::Space) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * vec4(
                            (aData.x -uStageHpos) * uStageZoom,
                            - (aData.y - uStageVpos) / uSize.y, 
                            0.0, 1.0)
                    }
                "),
                Statement::new_vertex("
                    gl_Position = transform(aBase,aDelta)
                "),
                Conditional::new("need-origin",vec![
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginBase"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginDelta"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),    
                    Statement::new_vertex("
                        vec4 x = transform(aOriginBase,aOriginDelta);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
            ],
            GeometryProgramName::Triangles(TrianglesProgramKind::Window) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * vec4(    delta.x/uSize.x+base.x*2.0-1.0,
                                             1.0-delta.y/uSize.y-base.y*2.0,    -0.5,1.0);
                    }
                "),
                Statement::new_vertex("
                    gl_Position = transform(aBase,aDelta)
                "),
                Conditional::new("need-origin",vec![
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginBase"),
                    AttributeProto::new(PR_LOW,GLArity::Vec2,"aOriginDelta"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),    
                    Statement::new_vertex("
                        vec4 x = transform(aOriginBase,aOriginDelta);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
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

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) struct GeometryProcessName(GeometryProgramName);

impl GeometryProcessName {
    pub(crate) fn new(program: GeometryProgramName) -> GeometryProcessName {
        GeometryProcessName(program)
    }

    pub(crate) fn get_program_name(&self) -> GeometryProgramName { self.0.clone() }
}

impl PartialOrd for GeometryProcessName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_program_name().enumerable().partial_cmp(&other.get_program_name().enumerable())
    }
}

impl Ord for GeometryProcessName  {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
