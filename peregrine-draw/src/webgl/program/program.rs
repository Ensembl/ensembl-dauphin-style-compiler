use web_sys::{ WebGlProgram, WebGlRenderingContext };
use crate::webgl::{GPUSpec, ProcessStanza, ProcessStanzaBuilder, make_program};
use super::attribute::{ Attribute, AttribHandle, AttributeProto };
use keyed::{ KeyedValues, KeyedData };
use super::uniform::{ Uniform, UniformHandle, UniformValues, UniformProto };
use super::texture::{ Texture, TextureValues, TextureProto };
use crate::webgl::util::handle_context_errors;
use super::source::SourceInstrs;
use crate::util::message::Message;
use std::rc::Rc;
use std::cell::RefCell;

pub struct ProgramBuilder {
    program: RefCell<Option<Rc<Program>>>,
    source: SourceInstrs,
    uniforms: KeyedValues<UniformHandle,UniformProto>,
    textures: KeyedValues<UniformHandle,TextureProto>,
    attribs: KeyedValues<AttribHandle,AttributeProto>,
    method: u32
}

impl ProgramBuilder {
    pub(crate) fn new(source: &SourceInstrs) -> Result<ProgramBuilder,Message> {
        let mut out = ProgramBuilder {
            program: RefCell::new(None),
            source: source.clone(),
            uniforms: KeyedValues::new(),
            textures: KeyedValues::new(),
            attribs: KeyedValues::new(),
            method: WebGlRenderingContext::TRIANGLES
        };
        source.register(&mut out)?;
        Ok(out)
    }

    pub(crate) fn add_uniform(&mut self, uniform: &UniformProto) -> Result<(),Message> {
        self.uniforms.add(uniform.name(),uniform.clone());
        Ok(())
    }

    pub(crate) fn add_texture(&mut self, texture: &TextureProto) -> Result<(),Message> {
        self.textures.add(texture.name(),texture.clone());
        Ok(())
    }

    pub(crate) fn add_attrib(&mut self, attrib: &AttributeProto) -> Result<(),Message> {
        self.attribs.add(attrib.name(),attrib.clone());
        Ok(())
    }

    pub(crate) fn set_method(&mut self, method: u32) { self.method = method; }

    pub(crate) fn make_stanza_builder(&self) -> ProcessStanzaBuilder {
        ProcessStanzaBuilder::new(&self.attribs)
    }

    pub(crate) fn get_uniform_handle(&self, name: &str) -> Result<UniformHandle,Message> {
        self.uniforms.get_handle(name).map_err(|e| Message::CodeInvariantFailed(format!("missing uniform key: {}",name)))
    }

    pub(crate) fn get_texture_handle(&self, name: &str) -> Result<UniformHandle,Message> {
        self.textures.get_handle(name).map_err(|e| Message::CodeInvariantFailed(format!("missing texture key: {}",name)))
    }

    pub(crate) fn get_attrib_handle(&self, name: &str) -> Result<AttribHandle,Message> {
        self.attribs.get_handle(name).map_err(|e| Message::CodeInvariantFailed(format!("missing attrib key: {}",name)))
    }

    pub(crate) fn make(&self, context: &WebGlRenderingContext, gpuspec: &GPUSpec) -> Result<Rc<Program>,Message> {
        let mut prog = self.program.borrow_mut();
        if prog.is_none() {
            let gl_prog = make_program(context, gpuspec, self.source.clone())?;
            *prog = Some(Rc::new(Program::new(context,gl_prog,&self)?));
        }
        Ok(prog.as_ref().unwrap().clone())
    }
}

pub struct Program {
    program: WebGlProgram,
    uniforms: KeyedValues<UniformHandle,Uniform>,
    attribs: KeyedValues<AttribHandle,Attribute>,
    textures: KeyedValues<UniformHandle,Texture>,
    method: u32
}

impl Program {
    fn new(context: &WebGlRenderingContext, program: WebGlProgram, builder: &ProgramBuilder) -> Result<Program,Message> {
        let mut out = Program {
            program: program.clone(),
            attribs: KeyedValues::new(),
            uniforms: KeyedValues::new(),
            textures: KeyedValues::new(),
            method: 0 // XXX dummy, suggests bad ordering
        };
        out.init(context,program,builder)?;
        Ok(out)
    }

    fn init(&mut self, context: &WebGlRenderingContext, program: WebGlProgram, builder: &ProgramBuilder) -> Result<(),Message> {
        self.attribs = builder.attribs.map(|_,a| { Attribute::new(a,context,&program) })?;
        self.uniforms = builder.uniforms.map(|_,u| { Uniform::new(u,context,&program) })?;
        self.textures = builder.textures.map(|_,t| { Texture::new(t,context,&program) })?;
        self.method = builder.method;
        Ok(())
    }

    pub(crate) fn get_method(&self) -> u32 { self.method }

    pub(super) fn make_uniforms(&self) -> KeyedData<UniformHandle,UniformValues> {
        self.uniforms.data().map::<_,_,()>(|_,u| Ok(UniformValues::new(u.clone()))).unwrap()
    }

    pub(super) fn make_textures(&self) -> KeyedData<UniformHandle,TextureValues> {
        self.textures.data().map::<_,_,()>(|_,t| Ok(TextureValues::new(t.clone()))).unwrap()
    }

    pub(super) fn make_stanzas(&self, context: &WebGlRenderingContext, stanza_builder: &ProcessStanzaBuilder) -> Result<Vec<ProcessStanza>,Message> {
        stanza_builder.make_stanzas(context,&self.attribs)
    }

    pub(crate) fn select_program(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        context.use_program(Some(&self.program));
        handle_context_errors(&context)?;
        Ok(())
    }

    // XXX ensure called!
    fn discard(&self, context: &WebGlRenderingContext) {
        context.delete_program(Some(&self.program));
    }
}
