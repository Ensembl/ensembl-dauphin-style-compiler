use crate::webgl::FlatId;
use crate::webgl::global::WebGlGlobal;
use super::uniform::UniformHandle;
use crate::util::message::Message;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext };
use super::source::{ Source };
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use super::program::Program;
use crate::webgl::util::handle_context_errors;

#[derive(Clone)]
pub(crate) struct Texture {
    name: String,
    location: Option<WebGlUniformLocation>
}

impl Texture {
    pub(crate) fn new(name: &str) -> Box<Texture> {
        Box::new(Texture {
            name: name.to_string(),
            location: None
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for Texture {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, _spec: &GPUSpec, phase: Phase) -> String {
        if phase != Phase::Fragment { return String::new(); }
        format!("uniform sampler2D {};\n",self.name)
    }

    fn build(&mut self, context: &WebGlRenderingContext, program: &mut Program) -> Result<(),Message> {
        self.location = context.get_uniform_location(program.program(),&self.name);
        handle_context_errors(context)?;
        program.add_texture(&self)
    }
}

pub(crate) struct TextureValues {
    texture: Texture,
    flat_id: Option<FlatId>,
    bound: bool
}

impl TextureValues {
    pub(super) fn new(texture: Texture) -> TextureValues {
        TextureValues { texture: texture, flat_id: None, bound: false }
    }

    pub fn set_value(&mut self, flat: &FlatId) -> Result<(),Message> {
        self.flat_id = Some(flat.clone());
        self.bound = true;
        Ok(())
    }

    pub(super) fn apply(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        if let (Some(flat_id),Some(location)) = (&self.flat_id,&self.texture.location) {
            let index = gl.bindery_mut().allocate(flat_id)?.apply(gl)?;
            self.bound = true;
            gl.context().uniform1i(Some(location),index as i32);
            handle_context_errors(gl.context())?;
        }
        Ok(())
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        if self.bound {
            if let Some(flat) = &self.flat_id {
                gl.bindery_mut().free(flat)?.apply(gl)?;
            }
        }
        Ok(())
    }
}
