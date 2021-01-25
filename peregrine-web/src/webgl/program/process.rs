use std::rc::Rc;
use crate::webgl::canvas::canvas::Canvas;
use super::accumulator::{ Accumulator, AccumulatedRun };
use super::program::Program;
use super::attribute::{ AttribHandle };
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues };
use super::keyed::{ KeyedValues };
use web_sys::{ WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;

pub struct ProcessBuilder<'c> {
    program: Rc<Program<'c>>,
    context: &'c WebGlRenderingContext,
    accumulator: Accumulator,
    uniforms: KeyedValues<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>
}

impl<'c> ProcessBuilder<'c> {
    pub(crate) fn new(program: Rc<Program<'c>>) -> ProcessBuilder<'c> {
        let mut uniforms = KeyedValues::new();
        let mut attribs = KeyedValues::new();
        for uniform in program.get_uniforms().iter() {
            uniforms.add(uniform.name(),UniformValues::new(uniform.clone()));
        }
        for attrib in program.get_attribs().iter() {
            attribs.add(attrib.name(),attrib.clone());
        }
        let context = program.context();
        ProcessBuilder {
            program,
            context,
            accumulator: Accumulator::new(attribs),
            uniforms,
            textures: vec![]
        }
    }

    pub fn get_uniform_handle(&mut self, name: &str) -> anyhow::Result<UniformHandle> {
        self.uniforms.get_handle(name)
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f32>) -> anyhow::Result<()> {
        self.uniforms.data_mut().get_mut(handle).set_value(&self.context,values)
    }

    pub fn get_attrib_handle(&self, name: &str) -> anyhow::Result<AttribHandle> {
        self.accumulator.get_attrib_handle(name)
    }

    pub(crate) fn get_accumulator(&mut self) -> &mut Accumulator {
        &mut self.accumulator
    }

    pub fn add_texture(&mut self, index: u32, element: &Canvas) -> anyhow::Result<()> {
        let mut entry = TextureValues::new(&self.context,index,element.clone())?;
        self.textures.push(entry);
        Ok(())
    }

    pub fn build(self) -> anyhow::Result<Process<'c>> {
        Process::new(self)
    }
}

pub struct Process<'c> {
    program: Rc<Program<'c>>,
    context: &'c WebGlRenderingContext,
    runs: Vec<AccumulatedRun>,
    uniforms: KeyedValues<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>
}

impl<'c> Process<'c> {
    fn new(builder: ProcessBuilder<'c>) -> anyhow::Result<Process<'c>> {
       Ok(Process {
            context: builder.context,
            program: builder.program,
            runs: builder.accumulator.make(builder.context)?,
            uniforms: builder.uniforms,
            textures: builder.textures
        })
    }

    pub fn draw(&self) -> anyhow::Result<()> {
        for run in self.runs.iter() {
            for entry in self.uniforms.data().values() {
                entry.activate(&self.context)?;
            }
            for entry in self.textures.iter() {
                entry.activate(&self.context)?;
            }
            let len = run.activate(&self.context)?;
            self.program.select_program()?;
            self.context.draw_elements_with_i32(self.program.get_method(),len as i32,WebGlRenderingContext::UNSIGNED_SHORT,0);
            handle_context_errors(self.context)?;
        }
        Ok(())
    }
}

impl<'c> Drop for Process<'c> {
    fn drop(&mut self) {
        for run in self.runs.iter_mut() {
            run.delete(&self.context);
        }
        for entry in self.textures.iter_mut() {
            entry.delete(&self.context);
        }
    }
}
