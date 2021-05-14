use crate::webgl::FlatId;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;
use web_sys::{ WebGlUniformLocation, WebGlRenderingContext, WebGlProgram };
use super::source::{ Source };
use super::super::{ GPUSpec, Phase };
use super::program::{ Program, ProgramBuilder };
use crate::webgl::util::handle_context_errors;

// XXX some merging into uniform?

#[derive(Clone)]
pub(crate) struct TextureProto {
    name: String
}

impl TextureProto {
    pub fn new(name: &str) -> Box<TextureProto> {
        Box::new(TextureProto {
            name: name.to_string()
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for TextureProto {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, _spec: &GPUSpec, phase: Phase) -> String {
        if phase != Phase::Fragment { return String::new(); }
        format!("uniform sampler2D {};\n",self.name)
    }

    fn register(&self, builder: &mut ProgramBuilder) -> Result<(),Message> {
        builder.add_texture(&self)
    }
}

#[derive(Clone)]
pub(crate) struct Texture {
    proto: TextureProto,
    location: Option<WebGlUniformLocation>
}

impl Texture {
    pub(super) fn new(proto: &TextureProto, context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<Texture,Message> {
        let location = context.get_uniform_location(program,&proto.name);
        handle_context_errors(context)?;
        Ok(Texture { proto: proto.clone(), location })
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
