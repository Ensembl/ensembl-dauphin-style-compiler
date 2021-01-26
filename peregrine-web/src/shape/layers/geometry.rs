use super::pingeometry::PinGeometry;
use super::fixgeometry::FixGeometry;
use super::tapegeometry::TapeGeometry;
use super::pagegeometry::PageGeometry;
use super::patina::PatinaAccessorName;
use crate::webgl::{ ProcessBuilder, SourceInstrs, Uniform, Attribute, GLArity, Header, Statement };
use super::consts::{ PR_LOW, PR_DEF };
use web_sys::{ WebGlRenderingContext };

pub(super) enum GeometryAccessorVariety { Pin, Fix, Tape, Page }

impl GeometryAccessorVariety {
    pub fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            GeometryAccessorVariety::Pin => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aOrigin.x -uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        - (aOrigin.y - uStageVpos + aVertexPosition.y) / uSize.y, 
                        0.0, 1.0)")
            ],
            /*
            PaintGeometry::Stretch => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aVertexPosition.x - uStageHpos) * uStageZoom,
                        - (aVertexPosition.y - uStageVpos) / uSize.y,
                        0.0, 1.0)")
            ],
            */
            GeometryAccessorVariety::Fix => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign.y,
                                        0.0, 1.0)")
            ],
            GeometryAccessorVariety::Tape => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
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
            GeometryAccessorVariety::Page => vec![
                Header::new(WebGlRenderingContext::TRIANGLES),
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                       (- (aVertexPosition.y - uStageVpos) / uSize.y) * aVertexSign.y, 
                                       0.0, 1.0)")
            ]
            // wiggles are Header::new(WebGlRenderingContext::TRIANGLES_STRIP),
        })
    }
}

pub(super) enum GeometryAccessor {
    Pin(PinGeometry),
    Fix(FixGeometry),
    Tape(TapeGeometry),
    Page(PageGeometry)
}

pub enum GeometryAccessorName { Pin, Fix, Tape, Page }

impl GeometryAccessorName {
    pub(super) fn make_accessor(&self, process: &ProcessBuilder, skin: &PatinaAccessorName) -> anyhow::Result<GeometryAccessor> {
        Ok(match self {
            GeometryAccessorName::Pin => GeometryAccessor::Pin(PinGeometry::new(process,skin)?),
            GeometryAccessorName::Fix => GeometryAccessor::Fix(FixGeometry::new(process,skin)?),
            GeometryAccessorName::Tape => GeometryAccessor::Tape(TapeGeometry::new(process,skin)?),
            GeometryAccessorName::Page => GeometryAccessor::Page(PageGeometry::new(process,skin)?),
        })
    }

    pub(super) fn get_variety(&self) -> GeometryAccessorVariety {
        match self {
            GeometryAccessorName::Pin => GeometryAccessorVariety::Pin,
            GeometryAccessorName::Fix => GeometryAccessorVariety::Fix,
            GeometryAccessorName::Tape => GeometryAccessorVariety::Tape,
            GeometryAccessorName::Page => GeometryAccessorVariety::Page,
        }
    }
}
