use web_sys::WebGlRenderingContext;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub enum CanvasWeave {
    Pixelate,
    Blur
}

impl CanvasWeave {
    fn apply(&self, context: &WebGlRenderingContext) {
        let (minf,magf,wraps,wrapt) = match self {
            CanvasWeave::Pixelate =>
                (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                 WebGlRenderingContext::CLAMP_TO_EDGE,WebGlRenderingContext::CLAMP_TO_EDGE),
            CanvasWeave::Blur =>
                (WebGlRenderingContext::LINEAR,WebGlRenderingContext::LINEAR,
                 WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT)
        };
        context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                           WebGlRenderingContext::TEXTURE_MIN_FILTER,
                           minf as i32);
        context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                           WebGlRenderingContext::TEXTURE_MAG_FILTER,
                           magf as i32);
        context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                           WebGlRenderingContext::TEXTURE_WRAP_S,
                           wraps as i32);
        context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
                           WebGlRenderingContext::TEXTURE_WRAP_T,
                           wrapt as i32);
    }
}