use anyhow::{ anyhow as err };
use web_sys::{ WebGlRenderingContext, WebGlTexture };
use crate::webgl::canvas::weave::CanvasWeave;
use crate::webgl::canvas::flatstore::{ FlatStore, FlatId };
use crate::webgl::util::handle_context_errors;


fn create_texture(context: &WebGlRenderingContext,canvas_store: &FlatStore, our_data: (u32,&FlatId)) -> anyhow::Result<(u32,WebGlTexture)> {
    let canvas = canvas_store.get_main_canvas(our_data.1)?;
    let texture = context.create_texture().ok_or_else(|| err!("cannot create texture"))?;
    handle_context_errors(context)?;
    context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
    handle_context_errors(context)?;
    context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
        WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,canvas.element()?
    ).map_err(|e| err!("cannot bind texture: {:?}",&e.as_string()))?;
    handle_context_errors(context)?;
    apply_weave(context,canvas.weave())?;
    Ok((our_data.0,texture))
}


fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> anyhow::Result<()> {
    let (minf,magf,wraps,wrapt) = match weave {
        CanvasWeave::Crisp =>
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::CLAMP_TO_EDGE,WebGlRenderingContext::CLAMP_TO_EDGE),
        CanvasWeave::Fuzzy =>
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

pub(crate) struct TextureValues {
    gl_value: Option<(u32,WebGlTexture)>,
    object: Texture
}

impl TextureValues {
    pub(super) fn new(context: &WebGlRenderingContext, store: &FlatStore, index: u32, canvas: &FlatId) -> anyhow::Result<TextureValues> {
        let object = Texture::new();
        let gl_value = Some(create_texture(context,store,(index,canvas))?);
        Ok(TextureValues { gl_value, object })
    }

    pub(super) fn activate(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        if let Some(buffer) = &self.gl_value {
            context.active_texture(WebGlRenderingContext::TEXTURE0+buffer.0);
            handle_context_errors(context)?;
            context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&buffer.1));
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub fn discard(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        if let Some(gl_value) = &self.gl_value {
            context.delete_texture(Some(&gl_value.1));
            handle_context_errors(context)?;
        }
        Ok(())
    }
}
