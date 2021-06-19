use std::rc::Rc;
use crate::webgl::{ ProcessStanzaBuilder, ProcessStanza };
use super::program::{ Program, ProgramBuilder };
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues, TextureHandle };
use keyed::KeyedData;
use crate::webgl::util::handle_context_errors;
use crate::stage::stage::{ ReadStage, ProgramStage };
use crate::webgl::{ FlatId };
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub(crate) struct ProcessBuilder {
    builder: Rc<ProgramBuilder>,
    stanza_builder: ProcessStanzaBuilder,
    uniforms: Vec<(UniformHandle,Vec<f32>)>,
    textures: Vec<(String,FlatId)>
}

impl ProcessBuilder {
    pub(crate) fn new(builder: Rc<ProgramBuilder>) -> ProcessBuilder {
        let stanza_builder = builder.make_stanza_builder();
        ProcessBuilder {
            builder,
            stanza_builder,
            uniforms: vec![],
            textures: vec![]
        }
    }

    pub(crate) fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f32>) -> Result<(),Message> {
        self.uniforms.push((handle.clone(),values));
        Ok(())
    }

    pub(crate) fn set_texture(&mut self, uniform_name: &str, canvas_id: &FlatId) -> Result<(),Message> {
        self.textures.push((uniform_name.to_string(),canvas_id.clone()));
        Ok(())
    }

    pub(crate) fn get_stanza_builder(&mut self) -> &mut ProcessStanzaBuilder {
        &mut self.stanza_builder
    }

    pub(crate) fn build(self, gl: &mut WebGlGlobal, left: f64) -> Result<Process,Message> {
        let program = self.builder.make(gl.context(),gl.gpuspec())?;
        let mut uniforms = program.make_uniforms();
        for (name,value) in self.uniforms {
            uniforms.get_mut(&name).set_value(&value)?;
        }
        let mut textures = program.make_textures();
        for (name,value) in self.textures {
            let handle = self.builder.get_texture_handle(&name)?;
            textures.get_mut(&handle).set_value(gl.flat_store(),&value)?;
        }
        Process::new(gl,program,&self.builder,self.stanza_builder,uniforms,textures,left)
    }
}

pub struct Process {
    program: Rc<Program>,
    stanzas: Vec<ProcessStanza>,
    program_stage: ProgramStage,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: KeyedData<TextureHandle,TextureValues>,
    left: f64
}

impl Process {
    fn new(gl: &mut WebGlGlobal, program: Rc<Program>, builder: &Rc<ProgramBuilder>, stanza_builder: ProcessStanzaBuilder, uniforms: KeyedData<UniformHandle,UniformValues>, textures: KeyedData<TextureHandle,TextureValues>, left: f64) -> Result<Process,Message> {
        let stanzas = program.make_stanzas(gl.context(),gl.aux_array(),&stanza_builder)?;
        let program_stage = ProgramStage::new(&builder)?;
        Ok(Process {
            program,
            stanzas,
            program_stage,
            uniforms,
            textures,
            left
        })
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: &[f32]) -> Result<(),Message> {
        self.uniforms.get_mut(handle).set_value(values)
    }

    pub(super) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, opacity: f64) -> Result<(),Message> {
        gl.bindery_mut().clear();
        let program_stage = self.program_stage.clone();
        program_stage.apply(stage,self.left,opacity,self)?;
        let context = gl.context();
        self.program.select_program(context)?;
        drop(context);
        for stanza in self.stanzas.iter() {
            let context = gl.context();
            stanza.activate(context)?;
            drop(context);
            for entry in self.textures.values_mut() {
                entry.apply(gl)?;
            }
            let context = gl.context();
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
