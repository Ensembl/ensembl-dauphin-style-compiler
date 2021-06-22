use super::super::program::attribute::{ Attribute, AttribHandle };
use js_sys::Float32Array;
use keyed::{ KeyedData, KeyedDataMaker };
use super::stanza::ProcessStanza;
use super::builder::ProcessStanzaAddable;
use web_sys::WebGlRenderingContext;
use std::rc::Rc;
use std::cell::RefCell;
use crate::util::message::Message;

#[derive(Clone)]
pub(crate) struct ProcessStanzaArray {
    attribs: Rc<RefCell<KeyedData<AttribHandle,Vec<f32>>>>,
    len: usize,
    active: Rc<RefCell<bool>>,
    self_active: bool
}

impl ProcessStanzaArray {
    pub(super) fn new(active: &Rc<RefCell<bool>>, maker: &KeyedDataMaker<'static,AttribHandle,Vec<f32>>, len: usize) -> Result<ProcessStanzaArray,Message> {
        let mut out = ProcessStanzaArray {
            attribs: Rc::new(RefCell::new(maker.make())),
            active: active.clone(),
            self_active: false,
            len
        };
        out.open()?;
        Ok(out)
    }

    pub(super) fn make_stanza(&self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext, aux_array: &Float32Array) -> Result<Option<ProcessStanza>,Message> {
        ProcessStanza::new_array(context,aux_array,self.len,values,&self.attribs)
    }

    pub(crate) fn open(&mut self) -> Result<(),Message> {
        if self.self_active { return Ok(()); }
        if *self.active.borrow() {
            return Err(Message::CodeInvariantFailed(format!("can only have one active campaign/array at once")));
        }
        *self.active.borrow_mut() = true;
        self.self_active = true;
        Ok(())
    }

    pub(crate) fn close(&mut self) -> Result<(),Message> {
        if !self.self_active {
            return Err(Message::CodeInvariantFailed(format!("closing unopened campaign/array")));
        }
        self.self_active = false;
        *self.active.borrow_mut() = false;
        Ok(())
    }
}

impl ProcessStanzaAddable for ProcessStanzaArray {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f32>, _dims: usize) -> Result<(),Message> {
        // TODO check size
        self.attribs.borrow_mut().get_mut(handle).extend_from_slice(&values);
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Message> {
        let values_size = values.len();
        let mut offset = 0;
        let mut remaining = self.len * dims;
        while remaining > 0 {
            let mut real_count = remaining;
            if offset+real_count > values_size { real_count = values_size-offset; }
            self.attribs.borrow_mut().get_mut(handle).extend_from_slice(&values[offset..(offset+real_count)]);
            remaining -= real_count;
            offset += real_count;
            if offset == values_size { offset = 0; }
        }
        Ok(())
    }
}
