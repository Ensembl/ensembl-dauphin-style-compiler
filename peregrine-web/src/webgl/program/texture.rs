use anyhow::{ anyhow as err };
use web_sys::{ WebGlRenderingContext, WebGlTexture, HtmlCanvasElement };
use super::values::ProcessValueType;
use crate::webgl::canvas::canvas::{ Canvas, CanvasWeave };
use crate::webgl::util::handle_context_errors;

fn create_texture(context: &WebGlRenderingContext, element: &HtmlCanvasElement) -> anyhow::Result<()> {
    context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
        WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,element
    );
    handle_context_errors(context)?;
    Ok(())
}

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> anyhow::Result<()> {
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
    handle_context_errors(context)?;
    Ok(())
}

pub struct Texture {
    texture_num: Option<u32>
}

impl Texture {
    pub(super) fn new() -> Texture {
        Texture {
            texture_num: None
        }
    }
}

impl ProcessValueType for Texture {
    type OurValue = (u32,Canvas);
    type GLValue = (u32,WebGlTexture);

    fn name(&self) -> &str { "" }

    fn activate(&self, context: &WebGlRenderingContext, gl_value: &(u32,WebGlTexture)) -> anyhow::Result<()> {
        context.active_texture(WebGlRenderingContext::TEXTURE0+gl_value.0);
        handle_context_errors(context)?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&gl_value.1));
        handle_context_errors(context)?;
        Ok(())
    }

    fn value_to_gl(&self, context: &WebGlRenderingContext, our_data: (u32,Canvas)) -> anyhow::Result<(u32,WebGlTexture)> {
        let texture = context.create_texture().ok_or_else(|| err!("cannot create texture"))?;
        handle_context_errors(context)?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
        handle_context_errors(context)?;
        create_texture(context,our_data.1.element())?;
        apply_weave(context,our_data.1.weave())?;
        Ok((our_data.0,texture))
    }

    fn delete(&self, context: &WebGlRenderingContext, gl_value: &(u32,WebGlTexture)) -> anyhow::Result<()> {
        context.delete_texture(Some(&gl_value.1));
        handle_context_errors(context)?;
        Ok(())
    }
}
