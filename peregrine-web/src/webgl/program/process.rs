use std::rc::Rc;
use crate::webgl::canvas::canvas::Canvas;
use super::accumulator::{ Accumulator, AccumulatedRun };
use super::program::Program;
use super::attribute::{ AttribHandle };
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues };
use super::keyed::{ KeyedValues, KeyedData };
use web_sys::{ WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;

pub struct ProtoProcess {
    program: Rc<Program>,
    accumulator: Accumulator,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>
}

impl ProtoProcess {
    pub(crate) fn new(program: Rc<Program>) -> ProtoProcess {
        let uniforms = program.make_uniforms();
        let context = program.context();
        let accumulator = program.make_accumulator();
        ProtoProcess {
            program,
            accumulator,
            uniforms,
            textures: vec![]
        }
    }

    pub fn program(&self) -> &Rc<Program> { &self.program }

    pub fn get_uniform_handle(&self, name: &str) -> anyhow::Result<UniformHandle> {
        self.program.get_uniform_handle(name)
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.uniforms.get_mut(handle).set_value(&self.program.context(),values)
    }

    pub fn get_attrib_handle(&self, name: &str) -> anyhow::Result<AttribHandle> {
        self.program.get_attrib_handle(name)
    }

    pub(crate) fn get_accumulator(&mut self) -> &mut Accumulator {
        &mut self.accumulator
    }

    pub fn add_texture(&mut self, index: u32, element: &Canvas) -> anyhow::Result<()> {
        let mut entry = TextureValues::new(&self.program.context(),index,element.clone())?;
        self.textures.push(entry);
        Ok(())
    }

    pub fn build(self) -> anyhow::Result<Process> {
        Process::new(self)
    }
}

pub struct Process {
    program: Rc<Program>,
    runs: Vec<AccumulatedRun>,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>
}

impl Process {
    fn new(builder: ProtoProcess) -> anyhow::Result<Process> {
        let runs = builder.program.make_runs(&builder.accumulator)?;
        Ok(Process {
            program: builder.program,
            runs,
            uniforms: builder.uniforms,
            textures: builder.textures
        })
    }

    pub fn draw(&self) -> anyhow::Result<()> {
        let context = self.program.context();
        for run in self.runs.iter() {
            for entry in self.uniforms.values() {
                entry.activate(context)?;
            }
            for entry in self.textures.iter() {
                entry.activate(context)?;
            }
            let len = run.activate(context)?;
            self.program.select_program()?;
            context.draw_elements_with_i32(self.program.get_method(),len as i32,WebGlRenderingContext::UNSIGNED_SHORT,0);
            handle_context_errors(context)?;
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let context = self.program.context();
        for run in self.runs.iter_mut() {
            run.delete(context);
        }
        for entry in self.textures.iter_mut() {
            entry.delete(context);
        }
    }
}
