use std::collections::{HashMap, HashSet};

use super::source::Source;
use super::program::{ Program, ProgramBuilder };
use super::super::{ GLArity, GPUSpec, Precision, Phase };
use web_sys::{ WebGlRenderingContext, WebGlBuffer, WebGlProgram };
use keyed::keyed_handle;
use crate::webgl::util::handle_context_errors;
use crate::util::message::Message;
use js_sys::Float32Array;

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

    fn register(&self, builder: &mut ProgramBuilder, _flags: &HashSet<String>) -> Result<(),Message> { 
        builder.add_attrib(&self)
    }
}

#[derive(Clone)]
pub(crate) struct Attribute {
    proto: AttributeProto,
    location: Option<u32>
}

impl Attribute {
    pub(super) fn new(proto: &AttributeProto, context: &WebGlRenderingContext, program: &WebGlProgram) -> Result<Attribute,Message> { 
        let location = context.get_attrib_location(program,&proto.name);
        handle_context_errors(context)?;
        if location == -1 {
            return Err(Message::WebGLFailure(format!("cannot get attrib '{}'",proto.name)));
        }
        Ok(Attribute {
            proto: proto.clone(),
            location: Some(location as u32)
        })
    }
}

// TODO less big-big to avoid f32 issue
fn create_buffer(context: &WebGlRenderingContext, aux_array: &Float32Array, values: &[f32]) -> Result<WebGlBuffer,Message> {
    let mut local_buffer = None;
    let values_js = if values.len() <= aux_array.length() as usize {
        aux_array
    } else {
        local_buffer = Some(Float32Array::new_with_length(values.len() as u32));
        local_buffer.as_ref().unwrap()
    };
    unsafe { values_js.set(&Float32Array::view(&values),0) }
    let buffer = context.create_buffer().ok_or(Message::WebGLFailure(format!("failed to create buffer")))?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&buffer));
    context.buffer_data_with_opt_array_buffer(
        WebGlRenderingContext::ARRAY_BUFFER,
        Some(&values_js.subarray(0,values.len() as u32).buffer()),
        WebGlRenderingContext::STATIC_DRAW
    );
    handle_context_errors(context)?;
    Ok(buffer)
}

fn replace_buffer(context: &WebGlRenderingContext, buffer: &WebGlBuffer, aux_array: &Float32Array, values: &[f32]) -> Result<(),Message> {
    let mut local_buffer = None;
    let values_js = if values.len() <= aux_array.length() as usize {
        aux_array
    } else {
        local_buffer = Some(Float32Array::new_with_length(values.len() as u32));
        local_buffer.as_ref().unwrap()
    };
    unsafe { values_js.set(&Float32Array::view(&values),0) }
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&buffer));
    context.buffer_data_with_opt_array_buffer(
        WebGlRenderingContext::ARRAY_BUFFER,
        Some(&values_js.subarray(0,values.len() as u32).buffer()),
        WebGlRenderingContext::STATIC_DRAW
    );
    handle_context_errors(context)?;
    Ok(())
}

pub(crate) struct AttributeValues {
    gl_value: WebGlBuffer,
    arity: i32,
    location: u32
}

impl AttributeValues {
    pub(crate) fn new(object: &Attribute, our_value: &[f32], context: &WebGlRenderingContext, aux_array: &Float32Array) -> Result<AttributeValues,Message> {
        Ok(AttributeValues {
            gl_value: create_buffer(context,aux_array,our_value)?,
            arity: object.proto.arity.to_num() as i32,
            location: object.location.unwrap()
        })
    }

    pub(crate) fn replace(&self, our_value: &[f32], context: &WebGlRenderingContext, aux_array: &Float32Array) -> Result<(),Message> {
        replace_buffer(context,&self.gl_value,aux_array,our_value)?;
        Ok(())
    }

    pub(crate) fn activate(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        let location = self.location;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&self.gl_value));
        handle_context_errors(context)?;
        context.enable_vertex_attrib_array(location);
        handle_context_errors(context)?;
        context.vertex_attrib_pointer_with_i32(location,self.arity,WebGlRenderingContext::FLOAT,false,0,0);
        handle_context_errors(context)?;    
        Ok(())
    }

    pub(crate) fn deactivate(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        context.disable_vertex_attrib_array(self.location);
        handle_context_errors(context)?;
        Ok(())
    }

    pub(crate) fn discard(&mut self, context: &WebGlRenderingContext) -> Result<(),Message> {
        context.delete_buffer(Some(&self.gl_value));
        handle_context_errors(context)?;
        Ok(())
    }
}
