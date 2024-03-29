use std::collections::HashSet;

use crate::shape::layers::consts::PR_LOW;
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasInUse;
use crate::webgl::{GLArity};
use crate::webgl::global::{WebGlGlobalRefs};
use keyed::keyed_handle;
use peregrine_toolkit::error::Error;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlProgram };
use super::source::{ Source };
use super::super::{ GPUSpec, Phase };
use super::program::{ ProgramBuilder };
use crate::webgl::util::{handle_context_errors2};

// XXX some merging into uniform?

keyed_handle!(TextureHandle);

#[derive(Clone)]
pub(crate) struct TextureProto {
    name: String,
    size_name: String,
    scale_name: String,
}

impl TextureProto {
    pub fn new(name: &str, size_name: &str, scale_name: &str) -> Box<TextureProto> {
        Box::new(TextureProto {
            name: name.to_string(),
            size_name: size_name.to_string(),
            scale_name: scale_name.to_string()
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for TextureProto {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase, _flags: &HashSet<String>) -> String {
        if phase != Phase::Fragment { return String::new(); }
        format!("uniform {} {};\nuniform {} {};\nuniform sampler2D {};\n",
            spec.best_size(&PR_LOW,&&Phase::Fragment).as_string(GLArity::Vec2), self.size_name,
            spec.best_size(&PR_LOW,&&Phase::Fragment).as_string(GLArity::Vec2), self.scale_name,
            self.name)
    }

    fn register(&self, builder: &mut ProgramBuilder, _flags: &HashSet<String>) -> Result<(),Error> {
        builder.add_texture(&self)
    }
}

#[derive(Clone)]
pub(crate) struct Texture {
    location: Option<WebGlUniformLocation>,
    location_size: Option<WebGlUniformLocation>,
    location_scale: Option<WebGlUniformLocation>
}

impl Texture {
    pub(super) fn new(proto: &TextureProto, context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<Texture,Error> {
        let location = context.get_uniform_location(program,&proto.name);
        let location_size = context.get_uniform_location(program,&proto.size_name);
        let location_scale = context.get_uniform_location(program,&proto.scale_name);
        handle_context_errors2(context)?;
        Ok(Texture { location, location_size, location_scale })
    }
}

pub(crate) struct TextureValues {
    texture: Texture,
    flat_id: Option<CanvasInUse>,
    flat_size: Option<(u32,u32)>,
    bound: bool
}

impl TextureValues {
    pub(super) fn new(texture: Texture) -> TextureValues {
        TextureValues { texture: texture, flat_id: None, flat_size: None, bound: false }
    }

    pub fn set_value(&mut self, flat_id: &CanvasInUse) -> Result<(),Error> {
        self.flat_id = Some(flat_id.clone());
        let size = flat_id.retrieve(|flat| { flat.size().clone() });
        self.flat_size = Some(size);
        Ok(())
    }

    pub(super) fn apply(&mut self, gl: &mut WebGlGlobalRefs) -> Result<(),Error> {
        if let (Some(flat_id),Some(location)) = (&self.flat_id,&self.texture.location) {
            let index = flat_id.modify(|c| c.activate(gl.textures,gl.context))?;
            self.bound = true;
            gl.context.uniform1i(Some(location),index as i32);
            handle_context_errors2(gl.context)?;
        }
        if let (Some(flat_size),Some(location_size)) = (&self.flat_size,&self.texture.location_size) {
            gl.context.uniform2f(Some(location_size),flat_size.0 as f32, flat_size.1 as f32);
            handle_context_errors2(gl.context)?;
        }
        if let Some(flat_scale) = &self.texture.location_scale {
            let bitmap_multiplier = gl.canvas_source.bitmap_multiplier();
            gl.context.uniform2f(Some(flat_scale),bitmap_multiplier, bitmap_multiplier);
        }
        Ok(())
    }
}
