use super::super::program::attribute::{ Attribute, AttribHandle };
use keyed::{ KeyedData, KeyedDataMaker };
use super::stanza::ProcessStanza;
use super::builder::ProcessStanzaAddable;
use web_sys::WebGlRenderingContext;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub(crate) struct ProcessStanzaArray {
    attribs: Rc<RefCell<KeyedData<AttribHandle,Vec<f64>>>>,
    len: usize,
    active: Rc<RefCell<bool>>
}

impl ProcessStanzaArray {
    pub(super) fn new(active: &Rc<RefCell<bool>>, maker: &KeyedDataMaker<'static,AttribHandle,Vec<f64>>, len: usize) -> ProcessStanzaArray {
        ProcessStanzaArray {
            attribs: Rc::new(RefCell::new(maker.make())),
            active: active.clone(),
            len
        }
    }

    pub(super) fn make_stanza(&self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> anyhow::Result<Option<ProcessStanza>> {
        ProcessStanza::new_array(context,self.len,values,&self.attribs)
    }

    pub(crate) fn close(&mut self) {
        *self.active.borrow_mut() = false;
    }
}

impl ProcessStanzaAddable for ProcessStanzaArray {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.attribs.borrow_mut().get_mut(handle).extend_from_slice(&values);
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        let values_size = values.len();
        let mut offset = 0;
        let mut remaining = self.len;
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
