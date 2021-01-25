use super::consts::{ PR_LOW, PR_DEF };
use crate::webgl::{ SourceInstrs, Uniform, Attribute, GLArity, Statement };

pub(crate) enum PaintGeometry {
    Pin,     /* tied to document (x and y) at a single position */
    Stretch, /* tied to x document at two positions (so that it stretches) and y document at one */
    Fix,     /* tied to window in both axes */
    Tape,    /* tied to document in x axis but to screen in y (eg rulers) */
    Page     /* tied to windox in x axis but to document in y (eg track markers, etc) */
}

impl PaintGeometry {
    pub fn to_index(&self) -> usize {
        match self {
            PaintGeometry::Pin => 0,
            PaintGeometry::Stretch => 1,
            PaintGeometry::Fix => 2,
            PaintGeometry::Tape => 3,
            PaintGeometry::Page => 4
        }
    }

    pub fn num_values(&self) -> usize { 5 }

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
                Attribute::new(PR_LOW,GLArity::Scalar,"aVertexSign"),
                Attribute::new(PR_LOW,GLArity::Scalar,"aOrigin"),
                Statement::new_vertex("
                    gl_Position = vec4(
                        (aOrigin - uStageHpos) * uStageZoom + 
                                    aVertexPosition.x / uSize.x,
                        (1.0 - aVertexPosition.y / uSize.y) * aVertexSign,
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