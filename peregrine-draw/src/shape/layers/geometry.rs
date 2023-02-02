use crate::shape::layers::consts::{ PR_DEF, PR_LOW };
use crate::webgl::{AttributeProto, Conditional, Declaration, GLArity, Header, ProgramBuilder, SourceInstrs, Statement, Varying, UniformProto, ProcessBuilder};
use web_sys::{ WebGlRenderingContext };
use enum_iterator::Sequence;

#[derive(Clone,Hash,PartialEq,Eq,Debug,Sequence)]
pub enum TrianglesGeometry {
    Tracking,
    TrackingSpecial(bool),
    Window(bool)
}

pub(crate) trait GeometryFactory {
    fn geometry_name(&self) -> GeometryProcessName;
}

#[derive(Clone,Hash,PartialEq,Eq,Debug,Sequence)]
pub(crate) enum GeometryProgramName {
    Wiggle,
    Triangles(TrianglesGeometry)
}

impl GeometryProgramName {
    pub(crate) fn get_source(&self) -> SourceInstrs {
        /* Most actual data, tracks etc. Follows movements around the region. Optimised for minimal GPU work.
         * Cannot do anything relative to the screen bottom to minimise the number of buffers required:
         * rulers etc need to use TrackingSpecial.
         * 
         * x = x, px
         * y = y, px
         * z = x, bp
         * a = depth
         */
        SourceInstrs::new(match self {
            GeometryProgramName::Triangles(TrianglesGeometry::Tracking) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_DEF,GLArity::Vec4,"aCoords"),
                Declaration::new_vertex("
                    vec4 transform(in vec4 p)
                    {
                        return uModel * vec4(
                            (p.z -uStageHpos) * uStageZoom + 
                                        p.x / uSize.x,
                            (uStageVpos - p.y) / uSize.y + 1.0, 
                            p.a, 1.0);
                    }
                "),
                Statement::new_vertex("
                    gl_Position = transform(aCoords);
                "),
                Conditional::new("need-origin",vec![
                    AttributeProto::new(PR_DEF,GLArity::Vec4,"aOriginCoords"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),    
                    Statement::new_vertex("
                        vec4 x = transform(aOriginCoords);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
            ],
            /* Data which follows movements around the region but which isn't well-enough behaved 
             * for Tracking, which mainly means things relative to the bottom of the screen or
             * running labels. Can  also be used for anything which Tracking is used for (strictly
             * more expressive) but is not optimised.
             */
            GeometryProgramName::Triangles(TrianglesGeometry::TrackingSpecial(_)) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_DEF,GLArity::Vec4,"aCoords"),
                AttributeProto::new(PR_DEF,GLArity::Vec4,"aRunCoords"),
                AttributeProto::new(PR_LOW,GLArity::Scalar,"aDepth"),
                AttributeProto::new(PR_DEF,GLArity::Vec4,"aOriginCoords"),
                UniformProto::new_vertex(PR_LOW,GLArity::Scalar,"uUseVertical"),
                Declaration::new_vertex("
                    vec4 transform(in vec4 p)
                    {
                        return uModel * vec4(
                        (p.z -uStageHpos) * uStageZoom + 
                        p.x / uSize.x,
                        (uUseVertical*uStageVpos-p.y)/uSize.y-p.a*2.0+1.0,
                        aDepth,1.0);
                    }
                "),
                Statement::new_vertex("
                    vec4 origin = transform(aOriginCoords);
                    vec4 base_origin = transform(vec4(
                        0,
                        aOriginCoords.y,
                        aOriginCoords.z,
                        aOriginCoords.a
                    ));
                    vec4 pos = transform(aCoords);
                    if (base_origin.x < uLeftRail) {
                        vec4 rightmost = transform(aRunCoords);
                        pos.x = min(uLeftRail,rightmost.x) - base_origin.x + pos.x;
                    }
                    gl_Position = pos;
                "),
                Conditional::new("need-origin",vec![
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),
                    Statement::new_vertex("
                        vec4 x = transform(aOriginCoords);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
            ],
            /* Data relative to the window, which doesn't follow the movements of the region.
             */
            GeometryProgramName::Triangles(TrianglesGeometry::Window(_)) => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                AttributeProto::new(PR_DEF,GLArity::Vec4,"aCoords"),
                AttributeProto::new(PR_LOW,GLArity::Scalar,"aDepth"),
                UniformProto::new_vertex(PR_LOW,GLArity::Scalar,"uUseVertical"),
                Declaration::new_vertex("
                    vec4 transform(in vec4 p)
                    {
                        return uModel * vec4(
                            p.x/uSize.x+p.z*2.0-1.0,
                            (uUseVertical*uStageVpos-p.y)/uSize.y-p.a*2.0+1.0,
                            aDepth,1.0);
                    }
                "),
                Statement::new_vertex("
                    gl_Position = transform(aCoords)
                "),
                Conditional::new("need-origin",vec![
                    AttributeProto::new(PR_DEF,GLArity::Vec4,"aOriginCoords"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vOrigin"),    
                    Statement::new_vertex("
                        vec4 x = transform(aOriginCoords);
                        vOrigin = vec2((x.x+1.0)*uFullSize.x,(x.y+1.0)*uFullSize.y);
                    ")
                ]),
            ],
            /* Wiggle tracks.
             */
            GeometryProgramName::Wiggle => vec![
                Header::new(WebGlRenderingContext::TRIANGLE_STRIP),
                AttributeProto::new(PR_DEF,GLArity::Vec2,"aData"),
                AttributeProto::new(PR_LOW,GLArity::Scalar,"aDepth"),
                Statement::new_vertex("
                    gl_Position = uModel * vec4(
                        (aData.x -uStageHpos) * uStageZoom,
                        1.0 - (aData.y - uStageVpos) / uSize.y, 
                        aDepth, 1.0)")
            ]
        })
    }
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum GeometryProcessName {
    Wiggle,
    Triangles(TrianglesGeometry)
}

impl GeometryProcessName {
    pub(crate) fn key(&self) -> String {
        match self {
            GeometryProcessName::Triangles(g) => format!("{:?}",g),
            GeometryProcessName::Wiggle => "wiggle".to_string()
        }
    }

    pub(crate) fn get_program_name(&self) -> GeometryProgramName {
        match self {
            GeometryProcessName::Triangles(g) => GeometryProgramName::Triangles(g.clone()),
            GeometryProcessName::Wiggle => GeometryProgramName::Wiggle
        }
    }

    pub(crate) fn is_vertical(&self) -> bool {
        match self {
            GeometryProcessName::Triangles(TrianglesGeometry::TrackingSpecial(use_vertical)) |
            GeometryProcessName::Triangles(TrianglesGeometry::Window(use_vertical)) => *use_vertical,
            _ => false
        }
    }
}