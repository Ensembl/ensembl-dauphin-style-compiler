/* This file is mainly full of tedious distractions about the process of binding textures in
 * order that such neither end up polluting the horrible binding algorithm itself nor the rest
 * of the code which uses the bindery. See binding.rs for purpose and details.
 */

use peregrine_toolkit::{error::Error};
use web_sys::{WebGlRenderingContext, WebGlTexture, HtmlCanvasElement};
use crate::webgl::{CanvasWeave, util::handle_context_errors2, GPUSpec};
use super::binding::{Binding, TextureProfile, SlotToken, Stats};

fn apply_weave(context: &WebGlRenderingContext,weave: &CanvasWeave) -> Result<(),Error> {
    let (minf,magf,wraps,wrapt) = match weave {
        CanvasWeave::Crisp =>
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::Fuzzy =>
            (WebGlRenderingContext::LINEAR,WebGlRenderingContext::LINEAR,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::Heraldry => 
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::HorizStack => 
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),
        CanvasWeave::VertStack => 
            (WebGlRenderingContext::NEAREST,WebGlRenderingContext::NEAREST,
                WebGlRenderingContext::REPEAT,WebGlRenderingContext::REPEAT),        
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
    handle_context_errors2(context)?;
    Ok(())
}

#[cfg(debug_canvasstore)]
fn show_stats(activations: u64, creates: u64) {
    use peregrine_toolkit::log;

    if activations > 0 && (activations % 5000) == 0 {
        let perc = 100 - (creates*100/activations);
        log!("{} activations needed {} creates, {}% hit rate",activations,creates,perc);
    }
}

#[cfg(not(debug_canvasstore))]
fn show_stats(_activations: u64, _creates: u64) {}

struct Profile;

impl TextureProfile<WebGlRenderingContext,(HtmlCanvasElement,CanvasWeave),WebGlTexture,Error> for Profile {
    fn create(&mut self, context: &WebGlRenderingContext, weave:(HtmlCanvasElement,CanvasWeave), _slot: usize) -> Result<WebGlTexture,Error> {
        let (element,weave) = weave;
        let texture = context.create_texture().ok_or_else(|| Error::fatal("cannot create texture"))?;
        handle_context_errors2(context)?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
        handle_context_errors2(context)?;
        context.tex_image_2d_with_u32_and_u32_and_canvas( // wow
            WebGlRenderingContext::TEXTURE_2D,0,WebGlRenderingContext::RGBA as i32,WebGlRenderingContext::RGBA,
            WebGlRenderingContext::UNSIGNED_BYTE,&element
        ).map_err(|e| Error::fatal(&format!("cannot bind texture: {:?}",&e.as_string())))?;
        handle_context_errors2(context)?;
        apply_weave(context,&weave)?;
        Ok(texture)
    }

    fn destroy(&mut self, context: &WebGlRenderingContext, texture: &WebGlTexture, slot: usize) -> Result<(),Error> {
        context.active_texture(WebGlRenderingContext::TEXTURE0 + (slot as u32));
        handle_context_errors2(context)?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,None);
        handle_context_errors2(context)?;
        context.delete_texture(Some(&texture));
        Ok(())
    }

    fn no_slots(&self) -> Error { Error::fatal("no free slots") }

    fn stats(&mut self, stats: &Stats) {
        show_stats(stats.activations,stats.creates)
    }
}

#[derive(Clone)]
pub(crate) struct TextureBindingSlot(SlotToken<WebGlRenderingContext,(HtmlCanvasElement,CanvasWeave),WebGlTexture,Error>);

impl TextureBindingSlot {
    pub(crate) fn activate(&self, element: &HtmlCanvasElement, weave: &CanvasWeave, context: &WebGlRenderingContext) -> Result<(WebGlTexture,u32),Error> {
        self.0.activate((element.clone(),weave.clone()),context)
    }
}

#[derive(Clone)]
pub(crate) struct TextureBinding {
    binding: Binding<WebGlRenderingContext,(HtmlCanvasElement,CanvasWeave),WebGlTexture,Error>
}

impl TextureBinding {
    pub(crate) fn new(gpu_spec: &GPUSpec) -> TextureBinding {
        TextureBinding {
            binding: Binding::new(Profile,gpu_spec.max_textures() as usize)
        }
    }

    pub(crate) fn new_token(&self, context: &WebGlRenderingContext) -> Result<TextureBindingSlot,Error> {
        self.binding.new_token(context).map(|t| TextureBindingSlot(t))
    }

    pub(crate) fn clear(&self, context: &WebGlRenderingContext) -> Result<(),Error> {
        self.binding.clear(context)
    }
}
 