use std::sync::{ Arc, Mutex };
use anyhow::{ anyhow as err, bail };
use web_sys::{ WebGlProgram, WebGlUniformLocation, WebGlRenderingContext };
use super::attribute::Attribute;
use super::uniform::Uniform;
use super::process::Process;
use super::values::ProcessValueType;
use crate::webgl::util::handle_context_errors;

struct ProgramData<'c> {
    context: &'c WebGlRenderingContext,
    program: WebGlProgram,
    uniforms: Vec<(Uniform,WebGlUniformLocation)>,
    attribs: Vec<(Attribute,u32)>,
    method: u32
}

impl<'c> ProgramData<'c> {
    fn new(context: &'c WebGlRenderingContext, program: WebGlProgram) -> ProgramData<'c> {
        ProgramData {
            program,
            context,
            uniforms: vec![],
            attribs: vec![],
            method: WebGlRenderingContext::TRIANGLES
        }
    }

    fn set_method(&mut self, method: u32) { self.method = method; }
    fn get_method(&self) -> u32 { self.method }

    fn add_uniform(&mut self, uniform: &Uniform) -> anyhow::Result<()> {
        let location = self.context.get_uniform_location(&self.program,uniform.name());
        handle_context_errors(self.context)?;
        let location = location.ok_or_else(|| err!("cannot get uniform '{}'",uniform.name()))?;
        self.uniforms.push((uniform.clone(),location));
        Ok(())
    }

    fn add_attrib(&mut self, attrib: &Attribute) -> anyhow::Result<()> {
        let location = self.context.get_attrib_location(&self.program,attrib.name());
        handle_context_errors(self.context)?;
        if location == -1 {
            bail!("cannot get attrib '{}'",attrib.name());
        }
        self.attribs.push((attrib.clone(),location as u32));
        Ok(())
    }

    fn get_uniforms(&self) -> Vec<(Uniform,WebGlUniformLocation)> {
        self.uniforms.iter().map(|x| (x.0.clone(),x.1.clone())).collect()
    }

    fn get_attribs(&self) -> Vec<(Attribute,u32)> {
        self.attribs.iter().map(|x| (x.0.clone(),x.1.clone())).collect()
    }

    fn select_program(&self) -> anyhow::Result<()> {
        self.context.use_program(Some(&self.program));
        handle_context_errors(self.context)?;
        Ok(())
    }

    fn context(&self) -> &'c WebGlRenderingContext {
        self.context
    }
}

impl<'c> Drop for ProgramData<'c> {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.program));
    }
}

#[derive(Clone)]
pub struct Program<'c>(Arc<Mutex<ProgramData<'c>>>);

impl<'c> Program<'c> {
    pub fn new(context: &'c WebGlRenderingContext, program: WebGlProgram) -> Program<'c> {
        Program(Arc::new(Mutex::new(ProgramData::new(context,program))))
    }

    pub(crate) fn add_uniform(&mut self, uniform: &Uniform) -> anyhow::Result<()> {
        self.0.lock().unwrap().add_uniform(uniform)
    }

    pub(crate) fn add_attrib(&mut self, attrib: &Attribute) -> anyhow::Result<()> {
        self.0.lock().unwrap().add_attrib(attrib)
    }

    pub(crate) fn set_method(&mut self, method: u32) {
        self.0.lock().unwrap().set_method(method);
    }

    pub(crate) fn get_method(&self) -> u32 {
        self.0.lock().unwrap().get_method()
    }

    pub fn select_program(&self) -> anyhow::Result<()> {
        self.0.lock().unwrap().select_program()
    }

    fn make_process(&self) -> anyhow::Result<Process<'c>> {
        Ok(Process::new(self))
    }

    pub(super) fn get_uniforms(&self) -> Vec<(Uniform,WebGlUniformLocation)> {
        self.0.lock().unwrap().get_uniforms()
    }

    pub(super) fn get_attribs(&self) -> Vec<(Attribute,u32)> {
        self.0.lock().unwrap().get_attribs()
    }

    pub(super) fn context(&self) -> &'c WebGlRenderingContext {
        self.0.lock().unwrap().context()
    }
}
