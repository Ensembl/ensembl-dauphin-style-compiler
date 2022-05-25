use super::super::program::attribute::{ Attribute, AttributeProto, AttribHandle };
use keyed::{ KeyedValues, KeyedDataMaker };
use super::array::ProcessStanzaArray;
use super::elements::{ ProcessStanzaElements, ProcessStanzaElementsEntry };
use super::stanza::{AttribSource, ProcessStanza};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;

pub trait ProcessStanzaAddable {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Message>;
    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) ->Result<(),Message>;
}

pub struct ProcessStanzaBuilder {
    elements: Vec<Rc<RefCell<ProcessStanzaElementsEntry>>>,
    arrays: Vec<ProcessStanzaArray>,
    maker: KeyedDataMaker<'static,AttribHandle,AttribSource>,
    active: Rc<RefCell<bool>>

}

impl ProcessStanzaBuilder {
    pub(crate) fn new(attribs: &KeyedValues<AttribHandle,AttributeProto>) -> ProcessStanzaBuilder {
        let maker = attribs.keys().make_maker(|| AttribSource::new());
        ProcessStanzaBuilder {
            maker,
            elements: vec![],
            arrays: vec![],
            active: Rc::new(RefCell::new(false))
        }
    }

    pub(super) fn active(&self) -> &Rc<RefCell<bool>> { &self.active }

    pub(super) fn make_elements_entry(&mut self) {
        self.elements.push(Rc::new(RefCell::new(ProcessStanzaElementsEntry::new(&self.maker))));
    }

    pub(super) fn elements<'a>(&'a mut self) -> &Rc<RefCell<ProcessStanzaElementsEntry>> {
        self.elements.last_mut().unwrap()
    }

    pub(crate) fn make_elements(&mut self, count: usize, indexes: &[u16]) -> Result<ProcessStanzaElements,Message> {
        if self.elements.len() == 0 {
            self.make_elements_entry();
        }
        ProcessStanzaElements::new(self,count,indexes)
    }

    pub(crate) fn make_array(&mut self, len: usize) -> Result<ProcessStanzaArray,Message> {
        let out = ProcessStanzaArray::new(&self.active,&self.maker,len)?;
        self.arrays.push(out.clone());
        Ok(out)
    }

    pub(crate) async fn make_stanzas(&self, gl: &Arc<Mutex<WebGlGlobal>>, attribs: &KeyedValues<AttribHandle,Attribute>) -> Result<Vec<ProcessStanza>,Message> {
        if *self.active.borrow() {
            return Err(Message::CodeInvariantFailed(format!("attempt to make while campaign still open")));
        }
        let mut out = vec![];
        for element in &self.elements {
            out.push(element.borrow().make_stanza(attribs.data(),gl).await?);
        }
        for array in &self.arrays {
            out.push(array.make_stanza(attribs.data(),gl).await?);
        }
        Ok(out.drain(..).filter(|x| x.is_some()).map(|x| x.unwrap()).collect())
    }
}
