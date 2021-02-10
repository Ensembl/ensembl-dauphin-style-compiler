use anyhow::{ bail };
use super::super::program::attribute::{ Attribute, AttribHandle };
use super::super::program::keyed::{ KeyedData, KeyedDataMaker };
use super::stanza::ProcessStanza;
use super::builder::{ ProcessStanzaBuilder, ProcessStanzaAddable };
use web_sys::{ WebGlRenderingContext };
use std::rc::Rc;
use std::cell::RefCell;

const LIMIT : usize = 16384;

pub(super) struct ProcessStanzaElementsEntry {
    attribs: KeyedData<AttribHandle,Vec<f64>>,
    index: Vec<u16>
}

impl ProcessStanzaElementsEntry {
    pub(super) fn new(maker: &KeyedDataMaker<'static,AttribHandle,Vec<f64>>) -> ProcessStanzaElementsEntry {
        ProcessStanzaElementsEntry {
            attribs: maker.make(),
            index: vec![]
        }
    }

    fn base(&self) -> usize {
        self.index.len()
    }

    fn space(&self, size: usize) -> usize {
        (LIMIT - self.index.len()) / size
    }

    fn add_indexes(&mut self, indexes: &[u16], count: usize) {
        for _ in 0..count {
            self.index.extend_from_slice(indexes);
        }
    }

    fn add(&mut self, handle: &AttribHandle, values: &[f64]) {
        self.attribs.get_mut(handle).extend_from_slice(values);
    }

    pub(super) fn make_stanza(self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> anyhow::Result<Option<ProcessStanza>> {
        ProcessStanza::new_elements(context,&self.index,values,self.attribs)
    }
}

pub struct ProcessStanzaElements {
    elements: Vec<(Rc<RefCell<ProcessStanzaElementsEntry>>,usize)>,
    tuple_size: usize,
    count: usize,
    active: Rc<RefCell<bool>>
}

impl ProcessStanzaElements {
    pub(super) fn new(stanza_builder: &mut ProcessStanzaBuilder, count: usize, indexes: &[u16]) -> ProcessStanzaElements {
        let mut out = ProcessStanzaElements {
            tuple_size: indexes.iter().max().map(|x| x+1).unwrap_or(0) as usize,
            elements: vec![],
            count,
            active: stanza_builder.active().clone()
        };
        let bases = out.allocate_entries(stanza_builder);
        out.add_indexes(indexes,&bases);
        out
    }

    fn allocate_entries(&mut self, stanza_builder: &mut ProcessStanzaBuilder) -> Vec<usize> {
        let mut bases = vec![];
        let mut remaining = self.count;
        while remaining > 0 {
            let entry = stanza_builder.elements().clone();
            let mut space = entry.borrow().space(self.tuple_size);
            if space > remaining { space = remaining; }
            if space > 0 {
                bases.push(entry.borrow().base());
                self.elements.push((entry,space));
            }
            remaining -= space;
            if remaining > 0 {
                stanza_builder.make_elements_entry();
            }
        }
        bases
    }

    fn add_indexes(&mut self, indexes: &[u16], bases: &[usize]) {
        for (i,(entry,count)) in self.elements.iter().enumerate() {
            let these_indexes : Vec<u16> = indexes.iter().map(|x| *x+(bases[i] as u16)).collect();
            entry.borrow_mut().add_indexes(&these_indexes,*count);
        }
    }

    pub(crate) fn close(&mut self) {
        *self.active.borrow_mut() = false;
    }
}

impl ProcessStanzaAddable for ProcessStanzaElements {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        let array_size = self.tuple_size * self.count;
        if values.len() != array_size {
            bail!("incorrect array length: expected {} got {}",array_size,values.len());
        }
        let mut offset = 0;
        for (entry,count) in &mut self.elements {
            let slice_size = *count*self.tuple_size;
            entry.borrow_mut().add(handle,&values[offset..(offset+slice_size)]);
            offset += slice_size;
        }
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        let values_size = values.len();
        let mut offset = 0;
        for (entry,count) in &mut self.elements {
            let mut remaining = *count*self.tuple_size;
            while remaining > 0 {
                let mut real_count = remaining;
                if offset+real_count > values_size { real_count = values_size-offset; }
                entry.borrow_mut().add(handle,&values[offset..(offset+real_count)]);
                remaining -= real_count;
                offset += real_count;
                if offset == values_size { offset = 0; }
            }
        }
        Ok(())
    }
}
