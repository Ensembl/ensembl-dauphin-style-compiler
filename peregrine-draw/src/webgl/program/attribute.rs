use std::collections::{HashSet};

use super::source::Source;
use super::program::{ ProgramBuilder };
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use peregrine_toolkit::error::Error;
use web_sys::{ WebGlRenderingContext, WebGlProgram };
use keyed::keyed_handle;
use crate::webgl::glbufferstore::GLDataBuffer;
use crate::webgl::global::{WebGlGlobalRefs};
use crate::webgl::util::{handle_context_errors2};

keyed_handle!(AttribHandle);

#[derive(Clone)]
pub(crate) struct AttributeProto {
    precision: Precision,
    arity: GLArity,
    name: String,
}

impl AttributeProto {
    pub fn new(precision: Precision, arity: GLArity, name: &str) -> Box<AttributeProto> {
        Box::new(AttributeProto {
            precision, arity,
            name: name.to_string()
        })
    }

    pub fn name(&self) -> &str { &self.name }
}

impl Source for AttributeProto {
    fn cloned(&self) -> Box<dyn Source> { Box::new(self.clone()) }

    fn declare(&self, spec: &GPUSpec, phase: Phase, _flags: &HashSet<String>) -> String {
        if phase != Phase::Vertex { return String::new(); }
        format!("attribute {} {};\n",spec.best_size(&self.precision,&Phase::Vertex).as_string(self.arity),self.name)
    }

    fn register(&self, builder: &mut ProgramBuilder, _flags: &HashSet<String>) -> Result<(),Error> { 
        builder.add_attrib(&self)
    }
}

#[derive(Clone)]
pub(crate) struct Attribute {
    proto: AttributeProto,
    location: Option<u32>
}

impl Attribute {
    pub(super) fn new(proto: &AttributeProto, context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<Attribute,Error> { 
        let location = context.get_attrib_location(program,&proto.name);
        handle_context_errors2(context)?;
        if location == -1 {
            return Err(Error::fatal(&format!("cannot get attrib '{}'",proto.name)));
        }
        Ok(Attribute {
            proto: proto.clone(),
            location: Some(location as u32)
        })
    }
}

pub(crate) struct AttributeValues {
    buffer: GLDataBuffer,
    arity: i32,
    location: u32
}

impl AttributeValues {
    pub(crate) fn new(object: &Attribute, our_value: &[f32], gl_refs: &WebGlGlobalRefs) -> Result<AttributeValues,Error> {
        let buffer = gl_refs.buffer_store.allocate_data_buffer(our_value.len())?;
        buffer.set(our_value)?;
        Ok(AttributeValues {
            buffer,
            arity: object.proto.arity.to_num() as i32,
            location: object.location.unwrap()
        })
    }

    pub(crate) fn replace(&self, our_value: &[f32]) -> Result<(),Error> {
        self.buffer.set(our_value)?;
        Ok(())
    }

    pub(crate) fn activate(&self) -> Result<(),Error> {
        self.buffer.activate_data(self.location,self.arity)?;
        Ok(())
    }

    pub(crate) fn deactivate(&self) -> Result<(),Error> {
        self.buffer.deactivate()?;
        Ok(())
    }
}
