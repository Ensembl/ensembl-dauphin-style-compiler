use std::sync::{Arc, Mutex};
use js_sys::{Float32Array, Uint16Array};
use peregrine_toolkit::lock;
use web_sys::{WebGlRenderingContext, WebGlBuffer};
use crate::Message;
use super::util::handle_context_errors;

pub(crate) struct GLDataBuffer {
    context: WebGlRenderingContext,
    buffer: WebGlBuffer,
    size: usize,
    activation: Arc<Mutex<Option<u32>>>
}

impl GLDataBuffer {
    fn new(context: &WebGlRenderingContext, size: usize) -> Result<GLDataBuffer,Message> {
        let values_js = Float32Array::new_with_length(size as u32);
        values_js.fill(0.,0,size as u32);
        let buffer = context.create_buffer().ok_or(Message::WebGLFailure(format!("failed to create buffer")))?;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&buffer));
        context.buffer_data_with_opt_array_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&values_js.buffer()),
            WebGlRenderingContext::STATIC_DRAW
        );
        handle_context_errors(context)?;
        Ok(GLDataBuffer {
            context: context.clone(),
            buffer,
            size,
            activation: Arc::new(Mutex::new(None))
        })
    }

    pub(crate) fn set(&self, values: &[f32]) -> Result<(),Message> {
        let values_js = Float32Array::new_with_length(values.len() as u32);
        unsafe { values_js.set(&Float32Array::view(&values),0) }
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&self.buffer));
        self.context.buffer_sub_data_with_i32_and_array_buffer(WebGlRenderingContext::ARRAY_BUFFER, 0, &values_js.buffer());
        handle_context_errors(&self.context)?;
        Ok(())
    }

    pub(crate) fn activate_data(&self, location: u32, arity: i32) -> Result<(),Message> {
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER,Some(&self.buffer));
        handle_context_errors(&self.context)?;
        self.context.enable_vertex_attrib_array(location);
        handle_context_errors(&self.context)?;
        self.context.vertex_attrib_pointer_with_i32(location,arity,WebGlRenderingContext::FLOAT,false,0,0);
        handle_context_errors(&self.context)?;
        *lock!(self.activation) = Some(location);
        Ok(())
    }

    pub(crate) fn deactivate(&self) -> Result<(),Message> {
        if let Some(location) =  &*lock!(self.activation) {
            self.context.disable_vertex_attrib_array(*location);
            handle_context_errors(&self.context)?;        
        }
        Ok(())
    }
}

impl Drop for GLDataBuffer {
    fn drop(&mut self) {
        self.context.delete_buffer(Some(&self.buffer));
    }
}

pub(crate) struct GLIndexBuffer {
    context: WebGlRenderingContext,
    buffer: WebGlBuffer,
    size: usize,
    activation: Arc<Mutex<Option<u32>>>
}

impl GLIndexBuffer {
    fn new(context: &WebGlRenderingContext, size: usize) -> Result<GLIndexBuffer,Message> {
        let values_js = Uint16Array::new_with_length(size as u32);
        values_js.fill(0,0,size as u32);
        let buffer = context.create_buffer().ok_or(Message::WebGLFailure(format!("failed to create buffer")))?;
        context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&buffer));
        context.buffer_data_with_opt_array_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&values_js.buffer()),
            WebGlRenderingContext::STATIC_DRAW
        );
        handle_context_errors(context)?;
        Ok(GLIndexBuffer {
            context: context.clone(),
            buffer,
            size,
            activation: Arc::new(Mutex::new(None))
        })
    }

    pub(crate) fn set(&self, values: &[u16]) -> Result<(),Message> {
        let values_js = Uint16Array::new_with_length(values.len() as u32);
        unsafe { values_js.set(&Uint16Array::view(&values),0) }
        self.context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&self.buffer));
        self.context.buffer_sub_data_with_i32_and_array_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, 0, &values_js.buffer());
        handle_context_errors(&self.context)?;
        Ok(())
    }

    pub(crate) fn activate(&self) -> Result<(),Message> {
        self.context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&self.buffer));
        handle_context_errors(&self.context)?;
        Ok(())
    }

    pub(crate) fn deactivate(&self) -> Result<(),Message> {
        self.context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,None);
        handle_context_errors(&self.context)?;
        Ok(())
    }
}

impl Drop for GLIndexBuffer {
    fn drop(&mut self) {
        self.context.delete_buffer(Some(&self.buffer));
    }
}

pub(crate) struct GLBufferStore {
    context: WebGlRenderingContext
}

impl GLBufferStore {
    pub(crate) fn new(context: &WebGlRenderingContext) -> GLBufferStore {
        GLBufferStore {
            context: context.clone()
        }
    }

    pub(crate) fn allocate_data_buffer(&self, size: usize) -> Result<GLDataBuffer,Message> {
        GLDataBuffer::new(&self.context,size)
    }

    pub(crate) fn allocate_index_buffer(&self, size: usize) -> Result<GLIndexBuffer,Message> {
        GLIndexBuffer::new(&self.context,size)
    }
}
