use std::rc::Rc;
use crate::webgl::canvas::canvas::Canvas;
use crate::webgl::{ ProcessStanzaBuilder, ProcessStanza };
use super::program::Program;
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues };
use crate::util::keyed::{ KeyedData };
use crate::webgl::util::handle_context_errors;
use crate::shape::core::stage::{ Stage, ProgramStage };

pub struct ProtoProcess {
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

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.uniforms.get_mut(handle).set_value(&self.program.context(),values)
    }

    pub(crate) fn get_stanza_builder(&mut self) -> &mut ProcessStanzaBuilder {
        &mut self.stanza_builder
    }

    pub fn add_texture(&mut self, index: u32, element: &Canvas) -> anyhow::Result<()> {
        let entry = TextureValues::new(&self.program.context(),index,element.clone())?;
        self.textures.push(entry);
        Ok(())
    }

    pub fn build(self) -> anyhow::Result<Process> {
        Process::new(self)
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
    fn new(builder: ProtoProcess) -> anyhow::Result<Process> {
        let stanzas = builder.program.make_stanzas(&builder.stanza_builder)?;
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

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.uniforms.get_mut(handle).set_value(&self.program.context(),values)
    }

    pub(super) fn draw(&mut self, stage: &Stage, opacity: f64) -> anyhow::Result<()> {
        let program_stage = self.program_stage.clone();
        program_stage.apply(stage,self.left,opacity,self)?;
        let context = self.program.context();
        self.program.select_program()?;
        for stanza in self.stanzas.iter() {
            for entry in self.uniforms.values() {
                entry.activate(context)?;
            }
            for entry in self.textures.iter() {
                entry.activate(context)?;
            }
            stanza.activate(context)?;
            stanza.draw(context,self.program.get_method())?;
            stanza.deactivate(context)?;
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self) -> anyhow::Result<()> {
        let context = self.program.context();
        for entry in self.uniforms.values_mut() {
            entry.discard(context)?;
        }
        for entry in self.textures.iter_mut() {
            entry.discard(context)?;
        }
        for stanza in self.stanzas.iter_mut() {
            stanza.discard(context)?;
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.discard();
    }
}
