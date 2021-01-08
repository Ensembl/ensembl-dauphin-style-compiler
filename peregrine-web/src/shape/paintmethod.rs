use crate::webgl::{ Program, Header };
use web_sys::WebGlRenderingContext;

pub(crate) enum PaintMethod {
    Triangle,
    Strip
}

impl PaintMethod {
    pub fn to_source(&self) -> Program {
        Program::new(vec![
            match self {
                PaintMethod::Triangle => Header::new(WebGlRenderingContext::TRIANGLES),
                PaintMethod::Strip => Header::new(WebGlRenderingContext::TRIANGLE_STRIP)
            }
        ])
    }
}