use super::super::program::attribute::{ AttribHandle };
use super::elementsentry::{ProcessStanzaElementsEntry, ProcessStanzaElementsEntryCursor};
use peregrine_toolkit::error::Error;
use super::builder::{ ProcessStanzaBuilder, ProcessStanzaAddable };
use std::rc::Rc;
use std::cell::RefCell;

pub struct ProcessStanzaElements {
    elements: Vec<(Rc<RefCell<ProcessStanzaElementsEntry>>,usize,ProcessStanzaElementsEntryCursor)>,
    points_per_shape: usize,
    index_len_per_shape: usize,
    shape_count: usize,
    active: Rc<RefCell<bool>>,
    self_active: bool
}

impl ProcessStanzaElements {
    pub(super) fn new(stanza_builder: &mut ProcessStanzaBuilder, shape_count: usize, indexes: &[u16]) -> Result<ProcessStanzaElements,Error> {
        let mut out = ProcessStanzaElements {
            points_per_shape: indexes.iter().max().map(|x| x+1).unwrap_or(0) as usize,
            index_len_per_shape: indexes.len(),
            elements: vec![],
            shape_count,
            active: stanza_builder.active().clone(),
            self_active: false
        };
        out.open()?;
        out.allocate_entries(stanza_builder,indexes)?;
        Ok(out)
    }

    fn allocate_entries(&mut self, stanza_builder: &mut ProcessStanzaBuilder, indexes: &[u16]) -> Result<(),Error> {
        let mut remaining_shapes = self.shape_count;
        while remaining_shapes > 0 {
            let entry = stanza_builder.elements().clone();
            let mut space_in_shapes = entry.borrow().space_in_shapes(self.points_per_shape,self.index_len_per_shape);
            if space_in_shapes > remaining_shapes { space_in_shapes = remaining_shapes; }
            if space_in_shapes > 0 {
                let cursor = entry.borrow_mut().add_indexes(&indexes,space_in_shapes as u16)?;
                self.elements.push((entry,space_in_shapes,cursor));
            }
            remaining_shapes -= space_in_shapes;
            if remaining_shapes > 0 {
                stanza_builder.make_elements_entry();
            }
        }
        Ok(())
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

impl ProcessStanzaAddable for ProcessStanzaElements {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Error> {
        let array_size = self.points_per_shape * self.shape_count * dims;
        if values.len() != array_size {
            return Err(Error::fatal(&format!("incorrect array length: expected {} ({}*{}*{}) got {}",array_size,self.points_per_shape,self.shape_count,dims,values.len())));
        }
        let mut offset = 0;
        for (entry,shape_count,cursor) in &mut self.elements {
            let slice_size = *shape_count*self.points_per_shape*dims;
            entry.borrow_mut().add(handle,cursor,&values[offset..(offset+slice_size)])?;
            offset += slice_size;
        }
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Error> {
        let values_size = values.len();
        if values_size == 0 { return Ok(()); }
        let mut offset = 0;
        for (entry,shape_count,cursor) in &mut self.elements {
            let mut remaining = *shape_count*self.points_per_shape*dims;
            while remaining > 0 {
                let mut real_count = remaining;
                if offset+real_count > values_size { real_count = values_size-offset; }
                entry.borrow_mut().add(handle,cursor,&values[offset..(offset+real_count)])?;
                remaining -= real_count;
                offset += real_count;
                if offset == values_size { offset = 0; }
            }
        }
        Ok(())
    }
}
