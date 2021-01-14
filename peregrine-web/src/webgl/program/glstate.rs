use anyhow::Context;
use std::collections::HashMap;
use web_sys::{ WebGlProgram, WebGlUniformLocation, WebGlRenderingContext };
use super::compiler::WebGlCompiler;
use super::super::gpuspec::gpuspec::GPUSpec;
use super::program::Program;
use super::source::SourceInstrs;

pub(crate) struct ProgramHandle(usize);

pub struct GLState<'c> {
    context: &'c WebGlRenderingContext,
    compiler: WebGlCompiler<'c>,
    programs: Vec<Program<'c>>,
    current_program: Option<usize>
}

impl<'c> GLState<'c> {
    pub fn new(context: &'c WebGlRenderingContext) -> GLState<'c> {
        let mut gpuspec = GPUSpec::new();
        gpuspec.populate(context);
        let compiler = WebGlCompiler::new(context,gpuspec);
        GLState {
            context,
            compiler,
            programs: vec![],
            current_program: None
        }
    }

    pub(crate) fn register_program(&mut self, source: SourceInstrs) -> anyhow::Result<ProgramHandle> {
        let program = self.compiler.make_program(source).context(format!("building program"))?;
        let handle = ProgramHandle(self.programs.len());
        self.programs.push(program);
        Ok(handle)
    }

    pub(crate) fn use_program(&mut self, handle: ProgramHandle) -> anyhow::Result<()> {
        self.current_program = Some(handle.0);
        Ok(())
    }
}
