use super::consts::{ PR_LOW, PR_DEF };
use crate::webgl::{ SourceInstrs, Header, Uniform, Attribute, GLArity, Varying, Statement };

pub(crate) enum PaintGeometry {
    Pin,
    Stretch,
    Fix,
    Tape,
    Page
}

impl PaintGeometry {
    pub fn to_source(&self) -> SourceInstrs {
        SourceInstrs::new(match self {
            PaintGeometry::Pin => vec![
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
            PaintGeometry::Stretch => vec![
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
            PaintGeometry::Fix => vec![
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign.y,
                                        0.0, 1.0)")
            ],
            PaintGeometry::Tape => vec![
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aOrigin.x -uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        (1.0 - ((aOrigin.y + aVertexPosition.y) / uSize.y)) * aVertexSign.y,
                        0.0, 1.0)")
            ],
            PaintGeometry::Page => vec![
                Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
                Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexPosition"),
                Attribute::new(PR_LOW,GLArity::Vec2,"aVertexSign"),
                Statement::new_vertex("
                    gl_Position = vec4((aVertexPosition.x / uSize.x - 1.0) * aVertexSign.x,
                                       (- (aVertexPosition.y - uStageVpos) / uSize.y) * aVertexSign.y, 
                                       0.0, 1.0)")
            ]
        })
    }
}