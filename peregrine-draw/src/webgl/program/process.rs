use std::rc::Rc;
use crate::webgl::{ ProcessStanzaBuilder, ProcessStanza };
use super::program::Program;
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues };
use keyed::KeyedData;
use crate::webgl::util::handle_context_errors;
use crate::shape::core::stage::{ ReadStage, ProgramStage };
use crate::webgl::{ FlatId };
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub(crate) struct ProtoProcess {
    program: Rc<Program>,
    stanza_builder: ProcessStanzaBuilder,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>,
    left: f64
}

impl ProtoProcess {
    pub(crate) fn new(program: Rc<Program>, left: f64) -> ProtoProcess {
        let uniforms = program.make_uniforms();
        let stanza_builder = program.make_stanza_builder();
        ProtoProcess {
            program,
            stanza_builder,
            uniforms,
            textures: vec![],
            left
        }
    }

    pub(crate) fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> Result<(),Message> {
        self.uniforms.get_mut(handle).set_value(values)
    }

    pub(crate) fn add_texture(&mut self, uniform_name: &str, canvas_id: &FlatId) -> Result<(),Message> {
        let handle = self.program.get_uniform_handle(uniform_name)?;
        let entry = TextureValues::new(&handle,canvas_id)?;
        self.textures.push(entry);
        Ok(())
    }

    pub(crate) fn get_stanza_builder(&mut self) -> &mut ProcessStanzaBuilder {
        &mut self.stanza_builder
    }

    pub(crate) fn build(self, gl: &mut WebGlGlobal) -> Result<Process,Message> {
        Process::new(gl,self)
    }
}

pub struct Process {
    program: Rc<Program>,
    stanzas: Vec<ProcessStanza>,
    program_stage: ProgramStage,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>,
    left: f64
}

impl Process {
    fn new(gl: &mut WebGlGlobal, builder: ProtoProcess) -> Result<Process,Message> {
        let stanzas = builder.program.make_stanzas(gl.context(),&builder.stanza_builder)?;
        let program_stage = ProgramStage::new(&builder.program)?;
        Ok(Process {
            program: builder.program,
            stanzas,
            program_stage,
            uniforms: builder.uniforms,
            textures: builder.textures,
            left: builder.left
        })
    }

    fn apply_textures(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let (textures, uniforms) = (&mut self.textures,&mut self.uniforms);
        for entry in textures.iter_mut() {
            let (uniform_handle,value) = entry.apply(gl)?;
            uniforms.get_mut(uniform_handle).set_value(vec![value as f64])?;
        }
        Ok(())
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> Result<(),Message> {
        self.uniforms.get_mut(handle).set_value(values)
    }

    pub(super) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, opacity: f64) -> Result<(),Message> {
        gl.bindery().clear();
        let program_stage = self.program_stage.clone();
        program_stage.apply(stage,self.left,opacity,self)?;
        self.apply_textures(gl)?;
        let context = gl.context();
        self.program.select_program(context)?;
        for stanza in self.stanzas.iter() {
            stanza.activate(context)?;
            for entry in self.uniforms.values() {
                entry.activate(context)?;
            }
            stanza.draw(context,self.program.get_method())?;
            stanza.deactivate(context)?;
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let context = gl.context();
        for entry in self.uniforms.values_mut() {
            entry.discard(context)?;
        }
        drop(context);
        for entry in self.textures.iter_mut() {
            entry.discard(gl)?;
        }
        let context = gl.context();
        for stanza in self.stanzas.iter_mut() {
            stanza.discard(context)?;
        }
        Ok(())
    }
}
