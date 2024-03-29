use super::super::program::attribute::{ Attribute, AttribHandle };
use keyed::{ KeyedData, KeyedDataMaker };
use peregrine_toolkit::error::Error;
use super::stanza::{AttribSource, ProcessStanza};
use super::builder::ProcessStanzaAddable;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use crate::webgl::global::WebGlGlobal;

#[derive(Clone)]
pub(crate) struct ProcessStanzaArray {
    attribs: Rc<RefCell<KeyedData<AttribHandle,AttribSource>>>,
    len: usize,
    active: Rc<RefCell<bool>>,
    self_active: bool
}

impl ProcessStanzaArray {
    pub(super) fn new(active: &Rc<RefCell<bool>>, maker: &KeyedDataMaker<'static,AttribHandle,AttribSource>, len: usize) -> Result<ProcessStanzaArray,Error> {
        let mut out = ProcessStanzaArray {
            attribs: Rc::new(RefCell::new(maker.make())),
            active: active.clone(),
            self_active: false,
            len
        };
        out.open()?;
        Ok(out)
    }

    pub(super) async fn make_stanza(&self, values: &KeyedData<AttribHandle,Attribute>, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<Option<ProcessStanza>,Error> {
        ProcessStanza::new_array(gl,self.len,values,&self.attribs.borrow()).await
    }

    pub(crate) fn open(&mut self) -> Result<(),Error> {
        if self.self_active { return Ok(()); }
        if *self.active.borrow() {
            return Err(Error::fatal("can only have one active campaign/array at once"));
        }
        *self.active.borrow_mut() = true;
        self.self_active = true;
        Ok(())
    }

    pub(crate) fn close(&mut self) -> Result<(),Error> {
        if !self.self_active {
            return Err(Error::fatal("closing unopened campaign/array"));
        }
        self.self_active = false;
        *self.active.borrow_mut() = false;
        Ok(())
    }
}

impl ProcessStanzaAddable for ProcessStanzaArray {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f32>, _dims: usize) -> Result<(),Error> {
        // TODO check size
        self.attribs.borrow_mut().get_mut(handle).get().extend_from_slice(&values);
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Error> {
        let values_size = values.len();
        if values_size == 0 { return Ok(()); }
        let mut offset = 0;
        let mut remaining = self.len * dims;
        while remaining > 0 {
            let mut real_count = remaining;
            if offset+real_count > values_size { real_count = values_size-offset; }
            self.attribs.borrow_mut().get_mut(handle).get().extend_from_slice(&values[offset..(offset+real_count)]);
            remaining -= real_count;
            offset += real_count;
            if offset == values_size { offset = 0; }
        }
        Ok(())
    }
}
