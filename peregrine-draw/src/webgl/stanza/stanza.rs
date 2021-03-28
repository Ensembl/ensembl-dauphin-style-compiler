use std::rc::Rc;
use std::cell::RefCell;
use super::super::program::attribute::{ AttribHandle, AttributeValues };
use keyed::{ KeyedData };
use web_sys::{ WebGlBuffer, WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;
use crate::webgl::Attribute;
use crate::util::message::Message;

fn create_index_buffer(context: &WebGlRenderingContext, values: &[u16]) -> Result<WebGlBuffer,Message> {
    let buffer = context.create_buffer().ok_or(Message::WebGLFailure(format!("failed to create buffer")))?;
    context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&buffer));
    // After `Int16Array::view` be very careful not to do any memory allocations before it's dropped.
    unsafe {
        let value_array = js_sys::Uint16Array::view(values);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &value_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        drop(value_array);
    }
    handle_context_errors(context)?;
    Ok(buffer)
}

pub(crate) struct ProcessStanza {
    attribs: KeyedData<AttribHandle,AttributeValues>,
    index: Option<WebGlBuffer>,
    len: usize
}

impl ProcessStanza {
    pub(super) fn new_elements(context: &WebGlRenderingContext, index: &[u16], values: &KeyedData<AttribHandle,Attribute>, attribs: KeyedData<AttribHandle,Vec<f64>>) -> Result<Option<ProcessStanza>,Message> {
        if index.len() > 0 {
            Ok(Some(ProcessStanza {
                index: Some(create_index_buffer(context,index)?),
                len: index.len(),
                attribs: attribs.map_into(|k,v| AttributeValues::new(values.get(&k),v,context))?
            }))
        } else {
            Ok(None)
        }
    }

    pub(super) fn new_array(context: &WebGlRenderingContext, len: usize, values: &KeyedData<AttribHandle,Attribute>, attribs: &Rc<RefCell<KeyedData<AttribHandle,Vec<f64>>>>) -> Result<Option<ProcessStanza>,Message> {
        if len > 0 {
            Ok(Some(ProcessStanza {
                index: None,
                len,
                attribs: attribs.replace(KeyedData::new()).map_into(|k,v| AttributeValues::new(values.get(&k),v,context))?
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn activate(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        if let Some(index) = &self.index {
            context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(index));
            handle_context_errors(context)?;
        }
        for attrib in self.attribs.values() {
            attrib.activate(context)?;
        }
        Ok(())
    }

    pub(crate) fn deactivate(&self, context: &WebGlRenderingContext) -> Result<(),Message> {
        context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,None);
        handle_context_errors(context)?;
        Ok(())
    }

    pub fn draw(&self, context: &WebGlRenderingContext, method: u32) -> Result<(),Message> {
        if self.index.is_some() {
            context.draw_elements_with_i32(method,self.len as i32,WebGlRenderingContext::UNSIGNED_SHORT,0);
            handle_context_errors(context)?;
        } else {
            context.draw_arrays(method,0,self.len as i32);
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub fn discard(&mut self, context: &WebGlRenderingContext) -> Result<(),Message> {
        if let Some(index) = &self.index {
            context.delete_buffer(Some(index));
            handle_context_errors(context)?;
        }
       for attrib in self.attribs.values_mut() {
            attrib.discard(context)?;
        }
        Ok(())
    }
}
