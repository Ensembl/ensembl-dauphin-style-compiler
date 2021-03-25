use anyhow::{ bail };
use super::super::program::attribute::{ Attribute, AttribHandle };
use keyed::{ KeyedData, KeyedDataMaker };
use super::stanza::ProcessStanza;
use super::builder::{ ProcessStanzaBuilder, ProcessStanzaAddable };
use web_sys::{ WebGlRenderingContext };
use std::rc::Rc;
use std::cell::RefCell;
use crate::util::message::Message;

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

    fn space(&self, points_per_shape: usize) -> usize {
        (LIMIT - self.index.len()) / points_per_shape
    }

    fn add_indexes(&mut self, indexes: &[u16], count: usize) {
        for _ in 0..count {
            self.index.extend_from_slice(indexes);
        }
    }

    fn add(&mut self, handle: &AttribHandle, values: &[f64]) {
        self.attribs.get_mut(handle).extend_from_slice(values);
    }

    pub(super) fn make_stanza(self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> Result<Option<ProcessStanza>,Message> {
        ProcessStanza::new_elements(context,&self.index,values,self.attribs)
    }
}

pub struct ProcessStanzaElements {
    elements: Vec<(Rc<RefCell<ProcessStanzaElementsEntry>>,usize)>,
    points_per_shape: usize,
    shape_count: usize,
    active: Rc<RefCell<bool>>
}

impl ProcessStanzaElements {
    pub(super) fn new(stanza_builder: &mut ProcessStanzaBuilder, shape_count: usize, indexes: &[u16]) -> ProcessStanzaElements {
        let mut out = ProcessStanzaElements {
            points_per_shape: indexes.iter().max().map(|x| x+1).unwrap_or(0) as usize,
            elements: vec![],
            shape_count,
            active: stanza_builder.active().clone()
        };
        let bases = out.allocate_entries(stanza_builder);
        out.add_indexes(indexes,&bases);
        out
    }

    fn allocate_entries(&mut self, stanza_builder: &mut ProcessStanzaBuilder) -> Vec<usize> {
        let mut bases = vec![];
        let mut remaining_shapes = self.shape_count;
        while remaining_shapes > 0 {
            let entry = stanza_builder.elements().clone();
            let mut space_in_shapes = entry.borrow().space(self.points_per_shape);
            if space_in_shapes > remaining_shapes { space_in_shapes = remaining_shapes; }
            if space_in_shapes > 0 {
                bases.push(entry.borrow().base());
                self.elements.push((entry,space_in_shapes));
            }
            remaining_shapes -= space_in_shapes;
            if remaining_shapes > 0 {
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
    fn add(&mut self, handle: &AttribHandle, values: Vec<f64>, dims: usize) -> Result<(),Message> {
        let array_size = self.points_per_shape * self.shape_count * dims;
        if values.len() != array_size {
            return Err(Message::CodeInvariantFailed(format!("incorrect array length: expected {} got {}",array_size,values.len())));
        }
        let mut offset = 0;
        for (entry,shape_count) in &mut self.elements {
            let slice_size = *shape_count*self.points_per_shape*dims;
            entry.borrow_mut().add(handle,&values[offset..(offset+slice_size)]);
            offset += slice_size;
        }
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>, dims: usize) -> Result<(),Message> {
        let values_size = values.len();
        let mut offset = 0;
        for (entry,shape_count) in &mut self.elements {
            let mut remaining = *shape_count*self.points_per_shape*dims;
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
