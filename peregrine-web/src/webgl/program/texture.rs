use anyhow::{ anyhow as err };
use web_sys::{ WebGlRenderingContext, WebGlTexture, HtmlCanvasElement };
use super::values::ProcessValueType;
use crate::webgl::canvas::canvas::{ Canvas, CanvasWeave };
use crate::process_value_handle;

// Unused, but necessary for ProcessValues
process_value_handle!(TextureHandle);


fn create_texture(context: &WebGlRenderingContext, element: &HtmlCanvasElement) {
    context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
        WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,element
    );
}

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) {
    let (minf,magf,wraps,wrapt) = match weave {
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

pub struct Texture {
}

impl Texture {
    pub(super) fn new() -> Texture {
        Texture {}
    }
}

impl ProcessValueType for Texture {
    type OurValue = Canvas;
    type GLKey = u32;
    type GLValue = WebGlTexture;

    fn name(&self) -> &str { "" }

    fn activate(&self, context: &WebGlRenderingContext, gl_key: &u32, gl_value: &WebGlTexture) -> anyhow::Result<()> {
        context.active_texture(WebGlRenderingContext::TEXTURE0+*gl_key);
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(gl_value));
        Ok(())
    }

    fn value_to_gl(&self, context: &WebGlRenderingContext, our_data: Canvas) -> anyhow::Result<WebGlTexture> {
        let texture = context.create_texture().ok_or_else(|| err!("cannot create texture"))?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
        create_texture(context,our_data.element());
        apply_weave(context,our_data.weave());
        Ok(texture)
    }

    fn delete(&self, context: &WebGlRenderingContext, gl_value: &WebGlTexture) -> anyhow::Result<()> {
        context.delete_texture(Some(gl_value));
        Ok(())
    }
}
