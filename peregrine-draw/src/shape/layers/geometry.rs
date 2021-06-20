use super::super::core::wigglegeometry::{WiggleProgram };
use super::super::core::tracktriangles::{ TrackTrianglesProgram };
use crate::shape::layers::consts::PR_DEF;
use crate::webgl::{AttributeProto, Conditional, Declaration, GLArity, Header, ProgramBuilder, SourceInstrs, Statement, Varying};
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

#[derive(Clone,Hash,PartialEq,Eq)]
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
            GeometryProgramName::BaseLabelTriangles => vec![
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
            GeometryProgramName::SpaceLabelTriangles => vec![
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
