use super::super::program::attribute::{ Attribute, AttributeProto, AttribHandle };
use js_sys::Float32Array;
use keyed::{ KeyedValues, KeyedDataMaker };
use super::array::ProcessStanzaArray;
use super::elements::{ ProcessStanzaElements, ProcessStanzaElementsEntry };
use super::stanza::ProcessStanza;
use web_sys::WebGlRenderingContext;
use std::rc::Rc;
use std::cell::RefCell;
use crate::util::message::Message;

pub trait ProcessStanzaAddable {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Message>;
    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) ->Result<(),Message>;
}

pub struct ProcessStanzaBuilder {
    elements: Vec<Rc<RefCell<ProcessStanzaElementsEntry>>>,
    arrays: Vec<ProcessStanzaArray>,
    maker: KeyedDataMaker<'static,AttribHandle,Vec<f32>>,
    active: Rc<RefCell<bool>>

}

impl ProcessStanzaBuilder {
    pub(crate) fn new(attribs: &KeyedValues<AttribHandle,AttributeProto>) -> ProcessStanzaBuilder {
        let maker = attribs.keys().make_maker(|| vec![]);
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

    pub(crate) fn make_stanzas(&self, context: &WebGlRenderingContext, aux_array: &Float32Array, attribs: &KeyedValues<AttribHandle,Attribute>) -> Result<Vec<ProcessStanza>,Message> {
        if *self.active.borrow() {
            return Err(Message::CodeInvariantFailed(format!("attempt to make while campaign still open")));
        }
        let mut out = self.elements.iter().map(|x| x.replace(ProcessStanzaElementsEntry::new(&self.maker)).make_stanza(attribs.data(),context,aux_array)).collect::<Result<Vec<_>,_>>()?;
        out.append(&mut self.arrays.iter().map(|x| x.make_stanza(attribs.data(),context,aux_array)).collect::<Result<_,_>>()?);
        Ok(out.drain(..).filter(|x| x.is_some()).map(|x| x.unwrap()).collect())
    }
}
