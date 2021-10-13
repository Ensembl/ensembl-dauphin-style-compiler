use std::any::Any;

use super::super::core::wigglegeometry::WiggleAdder;
use crate::shape::layers::consts::{ PR_DEF, PR_LOW };
use crate::shape::triangles::triangleadder::TriangleAdder;
use crate::util::enummap::{Enumerable, EnumerableKey};
use crate::webgl::{AttributeProto, Conditional, Declaration, GLArity, Header, ProcessBuilder, ProgramBuilder, SourceInstrs, Statement, UniformProto, Varying};
use web_sys::{ WebGlRenderingContext };
use crate::util::message::Message;
use peregrine_data::CoordinateSystem;

#[derive(Clone)]
pub(crate) enum GeometryAdder {
    Wiggle(WiggleAdder),
    Triangles(TriangleAdder),
}

impl GeometryAdder {
    fn to_any(&self) -> Box<dyn Any> { Box::new(self.clone()) }
}

pub struct GeometryYielder {
    name: GeometryProcessName,
    priority: i8,
    link: Option<Box<dyn Any>>
}

impl GeometryYielder {
    pub(crate) fn new(name: GeometryProcessName, priority: i8) -> GeometryYielder {
        GeometryYielder {
            name, priority,
            link: None
        }
    }

    pub fn get_adder<T: 'static>(&self) -> Result<&T,Message> {
        let x  = self.link.as_ref().map(|x| x.downcast_ref()).flatten();
        x.ok_or_else(|| Message::CodeInvariantFailed(format!("incorrect adder type")))
    }

    pub(crate) fn name(&self) -> &GeometryProcessName { &self.name }
    pub(crate) fn priority(&self) -> i8 { self.priority }
    pub(crate) fn set(&mut self, program: &GeometryAdder) -> Result<(),Message> {
        self.link = Some(program.to_any());
        Ok(())
    }
}

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub enum TrianglesGeometry {
    Tracking,
    Window,
    Sideways
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub enum TrianglesTransform {
    Identity,
    NegativeX,
    NegativeY,
    NegativeXY
}

#[derive(Clone,Hash,PartialEq,Eq,Debug)]
pub(crate) enum GeometryProgramName {
    Wiggle,
    Triangles(TrianglesGeometry)
}

impl EnumerableKey for GeometryProgramName {
    fn enumerable(&self) -> Enumerable {
        Enumerable(match self {
            GeometryProgramName::Wiggle => 0,
            GeometryProgramName::Triangles(TrianglesGeometry::Tracking) => 1,
            GeometryProgramName::Triangles(TrianglesGeometry::Window) => 2,
            GeometryProgramName::Triangles(TrianglesGeometry::Sideways) => 3,
        },4)
    }
}

impl GeometryProgramName {
    pub(crate) fn make_geometry_program(&self, builder: &ProgramBuilder) -> Result<GeometryAdder,Message> {
        Ok(match self {
            GeometryProgramName::Wiggle => GeometryAdder::Wiggle(WiggleAdder::new(builder)?),
            GeometryProgramName::Triangles(_) => GeometryAdder::Triangles(TriangleAdder::new(builder)?),
        })
    }

    pub(crate) fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            GeometryProgramName::Triangles(TrianglesGeometry::Tracking) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                UniformProto::new_vertex(PR_LOW,GLArity::Matrix4,"uTransform"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * uTransform * vec4(
                            (base.x -uStageHpos) * uStageZoom + 
                                        delta.x / uSize.x,
                            (base.y - uStageVpos + delta.y) / uSize.y - 1.0, 
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
            GeometryProgramName::Triangles(TrianglesGeometry::Sideways) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                UniformProto::new_vertex(PR_LOW,GLArity::Matrix4,"uTransform"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * uTransform * vec4(    delta.y/uSize.x+base.y*2.0-1.0,
                                                              delta.x/uSize.y+base.x*2.0-1.0,    
                                                              -0.5,1.0);
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
            GeometryProgramName::Triangles(TrianglesGeometry::Window) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aBase"),
                AttributeProto::new(PR_LOW,GLArity::Vec2,"aDelta"),
                UniformProto::new_vertex(PR_LOW,GLArity::Matrix4,"uTransform"),
                Declaration::new_vertex("
                    vec4 transform(in vec2 base, in vec2 delta)
                    {
                        return uModel * uTransform * vec4(delta.x/uSize.x+base.x*2.0-1.0,
                                                          delta.y/uSize.y+base.y*2.0-1.0,    -0.5,1.0);
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
                        1.0 - (aData.y - uStageVpos) / uSize.y, 
                        0.0, 1.0)")
            ]
        })
    }
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum GeometryProcessName {
    Wiggle,
    Triangles(TrianglesGeometry,TrianglesTransform)
}

impl GeometryProcessName {
    pub(crate) fn get_program_name(&self) -> GeometryProgramName {
        match self {
            GeometryProcessName::Triangles(g,_) => GeometryProgramName::Triangles(g.clone()),
            GeometryProcessName::Wiggle => GeometryProgramName::Wiggle
        }
    }

    pub(crate) fn apply_to_process(&self, geometry: &GeometryAdder, process: &mut ProcessBuilder) -> Result<(),Message> {
        match geometry {
            GeometryAdder::Triangles(adder) => {
                if let Some(handle) = &adder.transform {
                    match self {
                        GeometryProcessName::Wiggle => {},
                        GeometryProcessName::Triangles(_,transform) => {
                            match transform {
                                TrianglesTransform::NegativeX => {
                                    process.set_uniform(handle,vec![-1.,0.,0.,0., 0.,1.,0.,0., 0.,0.,1.,0., 0.,0.,0.,1.])?;
                                },
                                TrianglesTransform::Identity => {
                                    process.set_uniform(handle,vec![1.,0.,0.,0., 0.,1.,0.,0., 0.,0.,1.,0., 0.,0.,0.,1.])?;
                                },
                                TrianglesTransform::NegativeY => {
                                    process.set_uniform(handle,vec![1.,0.,0.,0., 0.,-1.,0.,0., 0.,0.,1.,0., 0.,0.,0.,1.])?;
                                },
                                TrianglesTransform::NegativeXY => {
                                    process.set_uniform(handle,vec![-1.,0.,0.,0., 0.,-1.,0.,0., 0.,0.,1.,0., 0.,0.,0.,1.])?;
                                },
                            }
                        }
                    }
                }
            }
            GeometryAdder::Wiggle(_) => {}
        }
        Ok(())
    }
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
