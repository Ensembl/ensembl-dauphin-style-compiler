use peregrine_toolkit::error::Error;
use web_sys::{ WebGlProgram, WebGlRenderingContext };
use crate::shape::layers::programstore::WebGLProgramName;
use crate::webgl::global::WebGlGlobal;
use crate::webgl::{GPUSpec, ProcessStanza, ProcessStanzaBuilder, make_program};
use super::attribute::{ Attribute, AttribHandle, AttributeProto };
use keyed::{ KeyedValues, KeyedData };
use super::uniform::{ Uniform, UniformHandle, UniformValues, UniformProto };
use super::texture::{ Texture, TextureValues, TextureProto, TextureHandle };
use crate::webgl::util::{handle_context_errors2};
use super::source::SourceInstrs;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct ProgramBuilder {
    program: RefCell<Option<Rc<Program>>>,
    source: SourceInstrs,
    uniforms: KeyedValues<UniformHandle,UniformProto>,
    textures: KeyedValues<TextureHandle,TextureProto>,
    attribs: KeyedValues<AttribHandle,AttributeProto>,
    method: u32,
    name: WebGLProgramName
}

impl ProgramBuilder {
    pub(crate) fn new(source: &SourceInstrs, name:&WebGLProgramName) -> Result<ProgramBuilder,Error> {
        let mut out = ProgramBuilder {
            program: RefCell::new(None),
            source: source.clone(),
            uniforms: KeyedValues::new(),
            textures: KeyedValues::new(),
            attribs: KeyedValues::new(),
            method: WebGlRenderingContext::TRIANGLES,
            name: name.clone()
        };
        let flags = source.get_flags();
        source.register(&mut out, &flags)?;
        Ok(out)
    }

    pub(crate) fn add_uniform(&mut self, uniform: &UniformProto) -> Result<(),Error> {
        self.uniforms.add(uniform.name(),uniform.clone());
        Ok(())
    }

    pub(crate) fn add_texture(&mut self, texture: &TextureProto) -> Result<(),Error> {
        self.textures.add(texture.name(),texture.clone());
        Ok(())
    }

    pub(crate) fn add_attrib(&mut self, attrib: &AttributeProto) -> Result<(),Error> {
        self.attribs.add(attrib.name(),attrib.clone());
        Ok(())
    }

    pub(crate) fn set_method(&mut self, method: u32) { self.method = method; }

    pub(crate) fn make_stanza_builder(&self) -> ProcessStanzaBuilder {
        ProcessStanzaBuilder::new(&self.attribs)
    }

    pub(crate) fn get_uniform_handle(&self, name: &str) -> Result<UniformHandle,Error> {
        self.uniforms.get_handle(name).map_err(|e| Error::fatal(&format!("missing uniform key: {}",name)))
    }

    pub(crate) fn get_texture_handle(&self, name: &str) -> Result<TextureHandle,Error> {
        self.textures.get_handle(name).map_err(|e| Error::fatal(&format!("missing texture key: {}",name)))
    }

    pub(crate) fn get_attrib_handle(&self, name: &str) -> Result<AttribHandle,Error> {
        self.attribs.get_handle(name).map_err(|e| Error::fatal(&format!("missing attrib key: {}",name)))
    }

    pub(crate) fn try_get_attrib_handle(&self, name: &str) -> Option<AttribHandle> {
        self.attribs.try_get_handle(name)
    }

    pub(crate) fn try_get_uniform_handle(&self, name: &str) -> Option<UniformHandle> {
        self.uniforms.try_get_handle(name)
    }

    pub(crate) fn make(&self, context: &WebGlRenderingContext, gpuspec: &GPUSpec) -> Result<Rc<Program>,Error> {
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
    textures: KeyedValues<TextureHandle,Texture>,
    method: u32
}

impl Program {
    fn new(context: &WebGlRenderingContext, program: WebGlProgram, builder: &ProgramBuilder) -> Result<Program,Error> {
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

    fn init(&mut self, context: &WebGlRenderingContext, program: WebGlProgram, builder: &ProgramBuilder) -> Result<(),Error> {
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

    pub(super) fn make_textures(&self) -> KeyedData<TextureHandle,TextureValues> {
        self.textures.data().map::<_,_,()>(|_,t| Ok(TextureValues::new(t.clone()))).unwrap()
    }

    pub(super) async fn make_stanzas(&self, gl: &Arc<Mutex<WebGlGlobal>>, stanza_builder: &ProcessStanzaBuilder) -> Result<Vec<ProcessStanza>,Error> {
        stanza_builder.make_stanzas(gl,&self.attribs).await
    }

    pub(crate) fn select_program(&self, context: &WebGlRenderingContext) -> Result<(),Error> {
        context.use_program(Some(&self.program));
        handle_context_errors2(&context)?;
        Ok(())
    }

    // XXX ensure called!
    fn discard(&self, context: &WebGlRenderingContext) {
        context.delete_program(Some(&self.program));
    }
}
