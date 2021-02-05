use crate::webgl::{ SourceInstrs, Uniform, GLArity };
use super::consts::{ PR_DEF };

pub(crate) fn get_stage_source() -> SourceInstrs {
    SourceInstrs::new(vec![
        Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
        Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
        Uniform::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
        Uniform::new_vertex(PR_DEF,GLArity::Vec2,"uSize")
    ])
}
