use web_sys::{ WebGlProgram, WebGlRenderingContext };
use crate::webgl::{ ProcessStanzaBuilder, ProcessStanza };
use super::attribute::{ Attribute, AttribHandle };
use keyed::{ KeyedValues, KeyedData };
use super::uniform::{ Uniform, UniformHandle, UniformValues };
use crate::webgl::util::handle_context_errors;
use super::source::SourceInstrs;

pub struct Program {
    context: WebGlRenderingContext,
    program: WebGlProgram,
    uniforms: KeyedValues<UniformHandle,Uniform>,
    attribs: KeyedValues<AttribHandle,Attribute>,
    method: u32
}

impl Program {
    pub(crate) fn new(context: &WebGlRenderingContext, program: WebGlProgram, mut source: SourceInstrs) -> anyhow::Result<Program> {
        let mut out = Program {
            program,
            context: context.clone(),
            attribs: KeyedValues::new(),
            uniforms: KeyedValues::new(),
            method: WebGlRenderingContext::TRIANGLES
        };
        source.build(&mut out)?;
        Ok(out)
    }

    pub(crate) fn set_method(&mut self, method: u32) { self.method = method; }
    pub(crate) fn get_method(&self) -> u32 { self.method }

    pub(crate) fn add_uniform(&mut self, uniform: &Uniform) -> anyhow::Result<()> {
        self.uniforms.add(uniform.name(),uniform.clone());
        Ok(())
    }

    pub fn get_attrib_handle(&self, name: &str) -> anyhow::Result<AttribHandle> {
        self.attribs.get_handle(name)
    }

    pub fn get_uniform_handle(&self, name: &str) -> anyhow::Result<UniformHandle> {
        self.uniforms.get_handle(name)
    }

    pub(crate) fn add_attrib(&mut self, attrib: &Attribute) -> anyhow::Result<()> {
        self.attribs.add(attrib.name(),attrib.clone());
        Ok(())
    }

    pub(crate) fn make_uniforms(&self) -> KeyedData<UniformHandle,UniformValues> {
        self.uniforms.data().map::<_,_,()>(|_,u| Ok(UniformValues::new(u.clone()))).unwrap()
    }

    pub(crate) fn make_stanza_builder(&self) -> ProcessStanzaBuilder {
        ProcessStanzaBuilder::new(&self.attribs)
    }

    pub(crate) fn make_stanzas(&self, stanza_builder: &ProcessStanzaBuilder) -> anyhow::Result<Vec<ProcessStanza>> {
        stanza_builder.make_stanzas(&self.context,&self.attribs)
    }

    pub(crate) fn select_program(&self) -> anyhow::Result<()> {
        self.context.use_program(Some(&self.program));
        handle_context_errors(&self.context)?;
        Ok(())
    }

    pub(crate) fn context(&self) -> &WebGlRenderingContext {
        &self.context
    }

    pub(crate) fn program(&self) -> &WebGlProgram { &self.program }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.context.delete_program(Some(&self.program));
    }
}
