use std::sync::{Arc, Mutex, MutexGuard};
use super::super::program::attribute::{ AttribHandle, AttributeValues };
use commander::cdr_tick;
use keyed::{ KeyedData };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use web_sys::{ WebGlRenderingContext };
use crate::webgl::glbufferstore::{GLIndexBuffer};
use crate::webgl::global::WebGlGlobal;
use crate::webgl::util::{handle_context_errors2};
use crate::webgl::Attribute;

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct AttribSource(Arc<Mutex<Vec<f32>>>);

impl AttribSource {
    pub fn new() -> AttribSource {
        AttribSource(Arc::new(Mutex::new(vec![])))
    }

    pub fn len(&self) -> usize { self.0.lock().unwrap().len() }

    pub fn get(&self) -> MutexGuard<Vec<f32>> {
        self.0.lock().unwrap()
    }
}

pub(crate) struct ProcessStanza {
    attribs: KeyedData<AttribHandle,(AttribSource,AttributeValues)>,
    buffer: Option<GLIndexBuffer>,
    len: usize
}

impl ProcessStanza {
    pub(crate) fn update_values(&self) -> Result<(),Error> {
        for (source,attrib) in self.attribs.values() {
            attrib.replace(&source.get())?;
        }
        Ok(())
    }

    pub(crate) fn number_of_buffers(&self) -> usize {
        self.attribs.len()
    }

    async fn make_attribs(gl: &Arc<Mutex<WebGlGlobal>>, values: &KeyedData<AttribHandle,Attribute>, attribs: &KeyedData<AttribHandle,AttribSource>) -> Result<KeyedData<AttribHandle,(AttribSource,AttributeValues)>,Error> {
        let mut a_values = KeyedData::new();
        for (k,v) in attribs.items() {
            let mut lgl = lock!(gl);
            let gl_refs = lgl.refs();
            let value = AttributeValues::new(values.get(&k),&v.get(),&gl_refs)?;
            drop(lgl);
            cdr_tick(0).await;
            a_values.insert(&k,value);
        }
        attribs.map(|k,v| 
            Ok((v.clone(),a_values.remove(&k).unwrap()))
        )
    }

    pub(super) async fn new_elements(gl: &Arc<Mutex<WebGlGlobal>>, index: &[u16], values: &KeyedData<AttribHandle,Attribute>, attribs: &KeyedData<AttribHandle,AttribSource>) -> Result<Option<ProcessStanza>,Error> {
        if index.len() > 0 {
            let mut lgl = lock!(gl);
            let gl_refs = lgl.refs();
            let index_buffer = gl_refs.buffer_store.allocate_index_buffer(index.len())?;
            index_buffer.set(index)?;
            drop(lgl);
            cdr_tick(0).await;
            let attribs = ProcessStanza::make_attribs(gl,values,attribs).await?;
            Ok(Some(ProcessStanza {
                buffer: Some(index_buffer),
                len: index.len(),
                attribs
            }))
        } else {
            Ok(None)
        }
    }

    pub(super) async fn new_array(gl: &Arc<Mutex<WebGlGlobal>>, len: usize, values: &KeyedData<AttribHandle,Attribute>, attribs: &KeyedData<AttribHandle,AttribSource>) -> Result<Option<ProcessStanza>,Error> {
        if len > 0 {
            Ok(Some(ProcessStanza {
                buffer: None,
                len,
                attribs: ProcessStanza::make_attribs(gl,values,attribs).await?
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn activate(&self) -> Result<(),Error> {
        if let Some(buffer) = &self.buffer {
            buffer.activate()?;
        }
        for (_,attrib) in self.attribs.values() {
            attrib.activate()?;
        }
        Ok(())
    }

    pub(crate) fn deactivate(&self) -> Result<(),Error> {
        if let Some(buffer) = &self.buffer {
            buffer.deactivate()?;
        }
        for (_,attrib) in self.attribs.values() {
            attrib.deactivate()?;
        }
        Ok(())
    }

    pub fn draw(&self, context: &WebGlRenderingContext, method: u32) -> Result<(),Error> {
        if self.buffer.is_some() {
            context.draw_elements_with_i32(method,self.len as i32,WebGlRenderingContext::UNSIGNED_SHORT,0);
            handle_context_errors2(context)?;
        } else {
            context.draw_arrays(method,0,self.len as i32);
            handle_context_errors2(context)?;
        }
        Ok(())
    }
}
