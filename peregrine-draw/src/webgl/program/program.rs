use web_sys::{ WebGlProgram, WebGlRenderingContext };
use crate::webgl::{GPUSpec, ProcessStanza, ProcessStanzaBuilder, make_program};
use super::attribute::{ Attribute, AttribHandle };
use keyed::{ KeyedValues, KeyedData };
use super::uniform::{ Uniform, UniformHandle, UniformValues };
use crate::webgl::util::handle_context_errors;
use super::source::SourceInstrs;
use crate::util::message::Message;

pub(crate) struct ProtoProgram {
    source: SourceInstrs
}

impl ProtoProgram {
    pub(crate) fn new(source: SourceInstrs) -> Result<ProtoProgram,Message> {
        Ok(ProtoProgram {
            source
        })
    }

    pub(crate) fn make(&self, context: &WebGlRenderingContext, gpuspec: &GPUSpec) -> Result<Program,Message> {
        let gl_prog = make_program(context, gpuspec, self.source.clone())?;
        Program::new(context,gl_prog,self)
    }
}

pub struct Program {
    program: WebGlProgram,
    uniforms: KeyedValues<UniformHandle,Uniform>,
    attribs: KeyedValues<AttribHandle,Attribute>,
    method: u32
}

impl Program {
    fn new(context: &WebGlRenderingContext, program: WebGlProgram, proto: &ProtoProgram) -> Result<Program,Message> {
        let mut out = Program {
            program,
            attribs: KeyedValues::new(),
            uniforms: KeyedValues::new(),
            method: WebGlRenderingContext::TRIANGLES
        };
        let mut source = proto.source.clone();
        source.build(context,&mut out)?;
        Ok(out)
    }

    pub(crate) fn set_method(&mut self, method: u32) { self.method = method; }
    pub(crate) fn get_method(&self) -> u32 { self.method }

    pub(crate) fn add_uniform(&mut self, uniform: &Uniform) -> Result<(),Message> {
        self.uniforms.add(uniform.name(),uniform.clone());
        Ok(())
    }

    pub(crate) fn get_attrib_handle(&self, name: &str) -> Result<AttribHandle,Message> {
        self.attribs.get_handle(name).map_err(|e| Message::CodeInvariantFailed(format!("missing attrib key: {}",name)))
    }

    pub(crate) fn get_uniform_handle(&self, name: &str) -> Result<UniformHandle,Message> {
        self.uniforms.get_handle(name).map_err(|e| Message::CodeInvariantFailed(format!("missing uniform key: {}",name)))
    }

    pub(crate) fn add_attrib(&mut self, attrib: &Attribute) -> Result<(),Message> {
        self.attribs.add(attrib.name(),attrib.clone());
        Ok(())
    }

    pub(crate) fn make_uniforms(&self) -> KeyedData<UniformHandle,UniformValues> {
        self.uniforms.data().map::<_,_,()>(|_,u| Ok(UniformValues::new(u.clone()))).unwrap()
    }

    pub(crate) fn make_stanza_builder(&self) -> ProcessStanzaBuilder {
        ProcessStanzaBuilder::new(&self.attribs)
    }

    pub(crate) fn make_stanzas(&self, context: &WebGlRenderingContext, stanza_builder: &ProcessStanzaBuilder) -> Result<Vec<ProcessStanza>,Message> {
        stanza_builder.make_stanzas(context,&self.attribs)
    }

    pub(crate) fn select_program(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        context.use_program(Some(&self.program));
        handle_context_errors(&context)?;
        Ok(())
    }

    pub(crate) fn program(&self) -> &WebGlProgram { &self.program }

    // XXX ensure called!
    fn discard(&mut self, context: &WebGlRenderingContext) {
        context.delete_program(Some(&self.program));
    }
}
