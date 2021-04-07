use std::rc::Rc;
use crate::webgl::{ ProcessStanzaBuilder, ProcessStanza };
use super::program::{ Program, ProtoProgram };
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
    uniforms: Vec<(UniformHandle,Vec<f64>)>,
    textures: Vec<(UniformHandle,FlatId)>,
    left: f64
}

impl ProtoProcess {
    pub(crate) fn new(program: Rc<Program>, left: f64) -> ProtoProcess {
        let stanza_builder = program.make_stanza_builder();
        ProtoProcess {
            program,
            stanza_builder,
            uniforms: vec![],
            textures: vec![],
            left
        }
    }

    pub(crate) fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> Result<(),Message> {
        self.uniforms.push((handle.clone(),values));
        Ok(())
    }

    pub(crate) fn set_texture(&mut self, uniform_name: &str, canvas_id: &FlatId) -> Result<(),Message> {
        let handle = self.program.get_texture_handle(uniform_name)?;
        self.textures.push((handle.clone(),canvas_id.clone()));
        Ok(())
    }

    pub(crate) fn get_stanza_builder(&mut self) -> &mut ProcessStanzaBuilder {
        &mut self.stanza_builder
    }

    pub(crate) fn build(self, gl: &mut WebGlGlobal) -> Result<Process,Message> {
        let mut uniforms = self.program.make_uniforms();
        for (name,value) in self.uniforms {
            uniforms.get_mut(&name).set_value(value)?;
        }
        let textures = self.program.make_textures();
        let (program,stanza_builder,left) = (
            self.program,
            self.stanza_builder,
            self.left
        );
        Process::new(gl,program,stanza_builder,uniforms,textures,left)
    }
}

pub struct Process {
    program: Rc<Program>,
    stanzas: Vec<ProcessStanza>,
    program_stage: ProgramStage,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: KeyedData<UniformHandle,TextureValues>,
    left: f64
}

impl Process {
    fn new(gl: &mut WebGlGlobal, program: Rc<Program>, stanza_builder: ProcessStanzaBuilder, uniforms: KeyedData<UniformHandle,UniformValues>, textures: KeyedData<UniformHandle,TextureValues>, left: f64) -> Result<Process,Message> {
        let stanzas = program.make_stanzas(gl.context(),&stanza_builder)?;
        let program_stage = ProgramStage::new(&program)?;
        Ok(Process {
            program,
            stanzas,
            program_stage,
            uniforms,
            textures,
            left
        })
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> Result<(),Message> {
        self.uniforms.get_mut(handle).set_value(values)
    }

    pub(super) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, opacity: f64) -> Result<(),Message> {
        gl.bindery().clear();
        let program_stage = self.program_stage.clone();
        program_stage.apply(stage,self.left,opacity,self)?;
        for entry in self.textures.values_mut() {
            entry.apply(gl)?;
        }
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
        for entry in self.textures.values_mut() {
            entry.discard(gl)?;
        }
        let context = gl.context();
        for stanza in self.stanzas.iter_mut() {
            stanza.discard(context)?;
        }
        Ok(())
    }
}
