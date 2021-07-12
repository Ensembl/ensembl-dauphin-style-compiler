use std::collections::HashSet;

use crate::shape::layers::consts::PR_LOW;
use crate::webgl::{FlatId, FlatStore, GLArity};
use crate::webgl::global::{WebGlGlobal, WebGlGlobalRefs};
use crate::util::message::Message;
use keyed::keyed_handle;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlProgram };
use super::source::{ Source };
use super::super::{ GPUSpec, Phase };
use super::program::{ ProgramBuilder };
use crate::webgl::util::handle_context_errors;

// XXX some merging into uniform?

keyed_handle!(TextureHandle);

#[derive(Clone)]
pub(crate) struct TextureProto {
    name: String,
    size_name: String
}

impl TextureProto {
    pub fn new(name: &str, size_name: &str) -> Box<TextureProto> {
        Box::new(TextureProto {
            name: name.to_string(),
            size_name: size_name.to_string()
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for TextureProto {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase, _flags: &HashSet<String>) -> String {
        if phase != Phase::Fragment { return String::new(); }
        format!("uniform {} {};\nuniform sampler2D {};\n",spec.best_size(&PR_LOW,&&Phase::Fragment).as_string(GLArity::Vec2),self.size_name,self.name)
    }

    fn register(&self, builder: &mut ProgramBuilder, _flags: &HashSet<String>) -> Result<(),Message> {
        builder.add_texture(&self)
    }
}

#[derive(Clone)]
pub(crate) struct Texture {
    proto: TextureProto,
    location: Option<WebGlUniformLocation>,
    location_size: Option<WebGlUniformLocation>
}

impl Texture {
    pub(super) fn new(proto: &TextureProto, context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<Texture,Message> {
        let location = context.get_uniform_location(program,&proto.name);
        let location_size = context.get_uniform_location(program,&proto.size_name);
        handle_context_errors(context)?;
        Ok(Texture { proto: proto.clone(), location, location_size })
    }
}

pub(crate) struct TextureValues {
    texture: Texture,
    flat_id: Option<FlatId>,
    flat_size: Option<(u32,u32)>,
    bound: bool
}

impl TextureValues {
    pub(super) fn new(texture: Texture) -> TextureValues {
        TextureValues { texture: texture, flat_id: None, flat_size: None, bound: false }
    }

    pub fn set_value(&mut self, flat_store: &FlatStore, flat_id: &FlatId) -> Result<(),Message> {
        self.flat_id = Some(flat_id.clone());
        let flat = flat_store.get(flat_id)?;
        self.flat_size = Some(flat.size().clone());
        Ok(())
    }

    pub(super) fn apply(&mut self, gl: &mut WebGlGlobalRefs) -> Result<(),Message> {
        if let (Some(flat_id),Some(location)) = (&self.flat_id,&self.texture.location) {
            let index = gl.bindery.allocate(flat_id,gl.flat_store,gl.context)?;
            self.bound = true;
            gl.context.uniform1i(Some(location),index as i32);
            handle_context_errors(gl.context)?;
        }
        if let (Some(flat_size),Some(location_size)) = (&self.flat_size,&self.texture.location_size) {
            gl.context.uniform2f(Some(location_size),flat_size.0 as f32, flat_size.1 as f32);
            handle_context_errors(gl.context)?;
        }
        Ok(())
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobalRefs) -> Result<(),Message> {
        if self.bound {
            if let Some(flat) = &self.flat_id {
                gl.bindery.free(flat,gl.flat_store)?;
            }
        }
        Ok(())
    }
}
