use anyhow::{ anyhow as err, bail };
use web_sys::{ WebGlProgram, WebGlUniformLocation, WebGlRenderingContext };
use super::attribute::Attribute;
use super::uniform::Uniform;
use crate::webgl::util::handle_context_errors;
use super::source::SourceInstrs;

pub struct Program<'c> {
    context: &'c WebGlRenderingContext,
    program: WebGlProgram,
    uniforms: Vec<(Uniform,WebGlUniformLocation)>,
    attribs: Vec<(Attribute,u32)>,
    method: u32
}

impl<'c> Program<'c> {
    pub(crate) fn new(context: &'c WebGlRenderingContext, program: WebGlProgram, mut source: SourceInstrs) -> anyhow::Result<Program<'c>> {
        let mut out = Program {
            program,
            context,
            uniforms: vec![],
            attribs: vec![],
            method: WebGlRenderingContext::TRIANGLES
        };
        source.build(&mut out)?;
        Ok(out)
    }

    pub(crate) fn set_method(&mut self, method: u32) { self.method = method; }
    pub(crate) fn get_method(&self) -> u32 { self.method }

    pub(crate) fn add_uniform(&mut self, uniform: &Uniform) -> anyhow::Result<()> {
        let location = self.context.get_uniform_location(&self.program,uniform.name());
        handle_context_errors(self.context)?;
        let location = location.ok_or_else(|| err!("cannot get uniform '{}'",uniform.name()))?;
        self.uniforms.push((uniform.clone(),location));
        Ok(())
    }

    pub(crate) fn add_attrib(&mut self, attrib: &Attribute) -> anyhow::Result<()> {
        let location = self.context.get_attrib_location(&self.program,attrib.name());
        handle_context_errors(self.context)?;
        if location == -1 {
            bail!("cannot get attrib '{}'",attrib.name());
        }
        self.attribs.push((attrib.clone(),location as u32));
        Ok(())
    }

    pub(crate) fn get_uniforms(&self) -> Vec<Uniform> {
        self.uniforms.iter().map(|x| x.0.clone()).collect()
    }

    pub(crate) fn get_attribs(&self) -> Vec<Attribute> {
        self.attribs.iter().map(|x| x.0.clone()).collect()
    }

    pub(crate) fn select_program(&self) -> anyhow::Result<()> {
        self.context.use_program(Some(&self.program));
        handle_context_errors(self.context)?;
        Ok(())
    }

    pub(crate) fn context(&self) -> &'c WebGlRenderingContext {
        self.context
    }

    pub(crate) fn program(&self) -> &WebGlProgram { &self.program }
}

impl<'c> Drop for Program<'c> {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.program));
    }
}
