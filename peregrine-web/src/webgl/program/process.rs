use std::rc::Rc;
use crate::webgl::canvas::canvas::Canvas;
use super::accumulator::{ Accumulator, AccumulatedRun };
use super::program::Program;
use super::uniform::{ UniformHandle, UniformValues };
use super::texture::{ TextureValues };
use super::keyed::{ KeyedData };
use web_sys::{ WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;
use crate::shape::core::stage::{ Stage, ProgramStage };

pub struct ProtoProcess {
    program: Rc<Program>,
    accumulator: Accumulator,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>,
    left: f64
}

impl ProtoProcess {
    pub(crate) fn new(program: Rc<Program>, left: f64) -> ProtoProcess {
        let uniforms = program.make_uniforms();
        let accumulator = program.make_accumulator();
        ProtoProcess {
            program,
            accumulator,
            uniforms,
            textures: vec![],
            left
        }
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.uniforms.get_mut(handle).set_value(&self.program.context(),values)
    }

    pub(crate) fn get_accumulator(&mut self) -> &mut Accumulator {
        &mut self.accumulator
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
    runs: Vec<AccumulatedRun>,
    program_stage: ProgramStage,
    uniforms: KeyedData<UniformHandle,UniformValues>,
    textures: Vec<TextureValues>,
    left: f64
}

impl Process {
    fn new(builder: ProtoProcess) -> anyhow::Result<Process> {
        let runs = builder.program.make_runs(&builder.accumulator)?;
        let program_stage = ProgramStage::new(&builder.program)?;
        Ok(Process {
            program: builder.program,
            runs,
            program_stage,
            uniforms: builder.uniforms,
            textures: builder.textures,
            left: builder.left
        })
    }

    pub fn set_uniform(&mut self, handle: &UniformHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.uniforms.get_mut(handle).set_value(&self.program.context(),values)
    }

    pub fn draw(&mut self, stage: &Stage, opacity: f64) -> anyhow::Result<()> {
        let program_stage = self.program_stage.clone();
        program_stage.apply(stage,self.left,opacity,self)?;
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

    pub(crate) fn discard(&mut self) -> anyhow::Result<()> {
        let context = self.program.context();
        for entry in self.uniforms.values_mut() {
            entry.discard(context)?;
        }
        for entry in self.textures.iter_mut() {
            entry.discard(context)?;
        }
        for run in self.runs.iter_mut() {
            run.discard(context)?;
        }
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.discard();
    }
}
