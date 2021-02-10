use anyhow::{ bail };
use super::super::program::attribute::{ Attribute, AttribHandle };
use super::super::program::keyed::{ KeyedValues, KeyedDataMaker };
use super::array::ProcessStanzaArray;
use super::elements::{ ProcessStanzaElements, ProcessStanzaElementsEntry };
use super::stanza::ProcessStanza;
use web_sys::WebGlRenderingContext;
use std::rc::Rc;
use std::cell::RefCell;

pub trait ProcessStanzaAddable {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()>;
    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()>;
}

pub struct ProcessStanzaBuilder {
    elements: Vec<Rc<RefCell<ProcessStanzaElementsEntry>>>,
    arrays: Vec<ProcessStanzaArray>,
    maker: KeyedDataMaker<'static,AttribHandle,Vec<f64>>,
    active: Rc<RefCell<bool>>

}

impl ProcessStanzaBuilder {
    pub(crate) fn new(attribs: &KeyedValues<AttribHandle,Attribute>) -> ProcessStanzaBuilder {
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

    pub(crate) fn make_elements(&mut self, count: usize, indexes: &[u16]) -> anyhow::Result<ProcessStanzaElements> {
        if *self.active.borrow() {
            bail!("can only have one active campaign/array at once");
        }
        if self.elements.len() == 0 {
            self.make_elements_entry();
        }
        *self.active.borrow_mut() = true;
        Ok(ProcessStanzaElements::new(self,count,indexes))
    }

    pub(crate) fn make_array(&mut self, len: usize) -> anyhow::Result<ProcessStanzaArray> {
        if *self.active.borrow() {
            bail!("can only have one active campaign/array at once");
        }
        let out = ProcessStanzaArray::new(&self.active,&self.maker,len);
        self.arrays.push(out.clone());
        *self.active.borrow_mut() = true;
        Ok(out)
    }

    pub(crate) fn make_stanzas(&self, context: &WebGlRenderingContext, attribs: &KeyedValues<AttribHandle,Attribute>) -> anyhow::Result<Vec<ProcessStanza>> {
        if *self.active.borrow() {
            bail!("can only make when inactive");
        }
        let mut out = self.elements.iter().map(|x| x.replace(ProcessStanzaElementsEntry::new(&self.maker)).make_stanza(attribs.data(),context)).collect::<Result<Vec<_>,_>>()?;
        out.append(&mut self.arrays.iter().map(|x| x.make_stanza(attribs.data(),context)).collect::<Result<_,_>>()?);
        Ok(out.drain(..).filter(|x| x.is_some()).map(|x| x.unwrap()).collect())
    }
}
