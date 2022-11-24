use std::rc::Rc;
use std::sync::{Arc, Mutex};
use crate::shape::layers::layer::ProgramCharacter;
use crate::webgl::{ ProcessStanzaBuilder, ProcessStanza };
use super::program::{ Program, ProgramBuilder };
use super::session::SessionMetric;
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues, TextureHandle };
use commander::cdr_tick;
use keyed::KeyedData;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use crate::webgl::util::{handle_context_errors2};
use crate::stage::stage::{ ReadStage, ProgramStage };
use crate::webgl::{ CanvasInUse };
use crate::webgl::global::WebGlGlobal;

pub(crate) struct ProcessBuilder {
    builder: Rc<ProgramBuilder>,
    stanza_builder: ProcessStanzaBuilder,
    uniforms: Vec<(UniformHandle,Vec<f32>)>,
    textures: Vec<(String,CanvasInUse)>
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

    pub(crate) fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f32>) -> Result<(),Error> {
        self.uniforms.push((handle.clone(),values));
        Ok(())
    }

    pub(crate) fn set_texture(&mut self, uniform_name: &str, canvas_id: &CanvasInUse) -> Result<(),Error> {
        self.textures.push((uniform_name.to_string(),canvas_id.clone()));
        Ok(())
    }

    pub(crate) fn get_stanza_builder(&mut self) -> &mut ProcessStanzaBuilder {
        &mut self.stanza_builder
    }

    pub(crate) async fn build(self, gl: &Arc<Mutex<WebGlGlobal>>, left: f64, character: &ProgramCharacter) -> Result<Process,Error> {
        let mut lgl = lock!(gl);
        let gl_ref = lgl.refs();
        let program = self.builder.make(gl_ref.context,gl_ref.gpuspec)?;
        drop(lgl);
        cdr_tick(0).await;
        let mut uniforms = program.make_uniforms();
        for (name,value) in self.uniforms {
            uniforms.get_mut(&name).set_value(&value)?;
            cdr_tick(0).await;
        }
        let mut textures = program.make_textures();
        for (name,value) in self.textures {
            let handle = self.builder.get_texture_handle(&name)?;
            textures.get_mut(&handle).set_value(&value)?;
            cdr_tick(0).await;
        }
        Process::new(gl,program,&self.builder,self.stanza_builder,uniforms,textures,left,character).await
    }
}

pub struct Process {
    program: Rc<Program>,
    stanzas: Vec<ProcessStanza>,
    program_stage: ProgramStage,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: KeyedData<TextureHandle,TextureValues>,
    left: f64,
    character: ProgramCharacter
}

impl Process {
    async fn new(gl: &Arc<Mutex<WebGlGlobal>>, program: Rc<Program>, builder: &Rc<ProgramBuilder>, stanza_builder: ProcessStanzaBuilder, uniforms: KeyedData<UniformHandle,UniformValues>, textures: KeyedData<TextureHandle,TextureValues>, left: f64, character: &ProgramCharacter) -> Result<Process,Error> {
        let stanzas = program.make_stanzas(gl,&stanza_builder).await?;
        let program_stage = ProgramStage::new(&builder)?;
        Ok(Process {
            program,
            stanzas,
            program_stage,
            uniforms,
            textures,
            left,
            character: character.clone()
        })
    }

    pub fn number_of_buffers(&self) -> usize {
        self.stanzas.iter().map(|x| x.number_of_buffers()).sum()
    }

    pub fn update_attributes(&self) -> Result<(),Error> {
        for stanza in &self.stanzas {
            stanza.update_values()?;
        }
        Ok(())
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: &[f32]) -> Result<(),Error> {
        self.uniforms.get_mut(handle).set_value(values)
    }

    pub(super) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, opacity: f64, dpr: f64, stats: &mut SessionMetric) -> Result<(),Error> {
        let mut gl = gl.refs();
        gl.textures.clear(gl.context);
        let program_stage = self.program_stage.clone();
        program_stage.apply(stage,self.left,opacity,dpr,self).map_err(|e| Error::fatal(&format!("XXX transition {:?}",e)))?;
        self.program.select_program(gl.context)?;
        for stanza in self.stanzas.iter() {
            stanza.activate()?;
            for entry in self.textures.values_mut() {
                entry.apply(&mut gl)?;
            }
            for entry in self.uniforms.values() {
                entry.activate(gl.context)?;
            }
            stanza.draw(gl.context,self.program.get_method())?;
            stanza.deactivate()?;
            handle_context_errors2(gl.context)?;
        }
        stats.add_character(&self.character);
        Ok(())
    }

    pub(crate) fn discard(&mut self,  gl: &mut WebGlGlobal) -> Result<(),Error> {
        let mut gl = gl.refs();
        for entry in self.uniforms.values_mut() {
            entry.discard(gl.context)?;
        }
        for stanza in self.stanzas.iter_mut() {
            stanza.discard(gl.context)?;
        }
        Ok(())
    }
}
