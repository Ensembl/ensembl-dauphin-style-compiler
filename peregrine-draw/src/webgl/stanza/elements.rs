use super::super::program::attribute::{ Attribute, AttribHandle };
use js_sys::Float32Array;
use keyed::{ KeyedData, KeyedDataMaker };
use super::stanza::{AttribSource, ProcessStanza};
use super::builder::{ ProcessStanzaBuilder, ProcessStanzaAddable };
use web_sys::{ WebGlRenderingContext };
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;

const LIMIT : usize = 16384;

pub(super) struct ProcessStanzaElementsEntryCursor(KeyedData<AttribHandle,usize>);

pub(super) struct ProcessStanzaElementsEntry {
    attribs: KeyedData<AttribHandle,AttribSource>,
    index: Vec<u16>,
    offset: u16
}

impl ProcessStanzaElementsEntry {
    pub(super) fn new(maker: &KeyedDataMaker<'static,AttribHandle,AttribSource>) -> ProcessStanzaElementsEntry {
        ProcessStanzaElementsEntry {
            attribs: maker.make(),
            index: vec![],
            offset: 0
        }
    }

    fn make_cursor(&self) -> Result<ProcessStanzaElementsEntryCursor,Message> {
        Ok(ProcessStanzaElementsEntryCursor(self.attribs.map(|_,v| Ok(v.len()))?))
    }

    fn space_in_shapes(&self, points_per_shape: usize, index_len_per_shape: usize) -> usize {
        let index_space = (LIMIT - self.index.len()) / index_len_per_shape;
        let points_space = (LIMIT - self.offset as usize) /points_per_shape;
        index_space.min(points_space)
    }

    fn add_indexes(&mut self, indexes: &[u16], count: u16) -> Result<ProcessStanzaElementsEntryCursor,Message> {
        let cursor = self.make_cursor()?;
        let max_new_index = *(if let Some(x) = indexes.iter().max() { x } else { return Ok(cursor); });
        for index in 0..count {
            let offset = index * (max_new_index+1) + self.offset;
            self.index.extend(indexes.iter().map(|x| *x+offset));
        }
        self.offset += count * (max_new_index+1);
        Ok(cursor)
    }

    fn add(&mut self, handle: &AttribHandle, cursor: &ProcessStanzaElementsEntryCursor, values: &[f32]) -> Result<(),Message> {
        let position = *cursor.0.get(handle);
        let mut target = self.attribs.get_mut(handle).get();
        if position > target.len() {
            return Err(Message::CodeInvariantFailed(format!("cursor after end")));
        } else if position < target.len() {
            target.splice(position..(position+values.len()),values.iter().cloned());
        } else {
            target.extend_from_slice(values);
        }
        Ok(())
    }

    pub(super) async fn make_stanza(&self, values: &KeyedData<AttribHandle,Attribute>, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<Option<ProcessStanza>,Message> {
        let out = ProcessStanza::new_elements(gl,&self.index,values,&self.attribs).await?;
        Ok(out)
    }
}

pub struct ProcessStanzaElements {
    elements: Vec<(Rc<RefCell<ProcessStanzaElementsEntry>>,usize,ProcessStanzaElementsEntryCursor)>,
    points_per_shape: usize,
    index_len_per_shape: usize,
    shape_count: usize,
    active: Rc<RefCell<bool>>,
    self_active: bool
}

impl ProcessStanzaElements {
    pub(super) fn new(stanza_builder: &mut ProcessStanzaBuilder, shape_count: usize, indexes: &[u16]) -> Result<ProcessStanzaElements,Message> {
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

    fn allocate_entries(&mut self, stanza_builder: &mut ProcessStanzaBuilder, indexes: &[u16]) -> Result<(),Message> {
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

impl ProcessStanzaAddable for ProcessStanzaElements {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Message> {
        let array_size = self.points_per_shape * self.shape_count * dims;
        if values.len() != array_size {
            return Err(Message::CodeInvariantFailed(format!("incorrect array length: expected {} got {}",array_size,values.len())));
        }
        let mut offset = 0;
        for (entry,shape_count,cursor) in &mut self.elements {
            let slice_size = *shape_count*self.points_per_shape*dims;
            entry.borrow_mut().add(handle,cursor,&values[offset..(offset+slice_size)])?;
            offset += slice_size;
        }
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f32>, dims: usize) -> Result<(),Message> {
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
