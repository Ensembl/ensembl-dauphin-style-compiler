use std::sync::{Arc, Mutex};
use keyed::{KeyedData, KeyedDataMaker};
use peregrine_toolkit::error::Error;
use crate::webgl::{ProcessStanza, AttribHandle, Attribute, global::WebGlGlobal};
use super::stanza::AttribSource;

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

    fn make_cursor(&self) -> Result<ProcessStanzaElementsEntryCursor,Error> {
        Ok(ProcessStanzaElementsEntryCursor(self.attribs.map(|_,v| Ok(v.len()))?))
    }

    pub(super) fn space_in_shapes(&self, points_per_shape: usize, index_len_per_shape: usize) -> usize {
        let index_space = (LIMIT - self.index.len()) / index_len_per_shape;
        let points_space = (LIMIT - self.offset as usize) /points_per_shape;
        index_space.min(points_space)
    }

    pub(super) fn add_indexes(&mut self, indexes: &[u16], count: u16) -> Result<ProcessStanzaElementsEntryCursor,Error> {
        let cursor = self.make_cursor()?;
        let max_new_index = *(if let Some(x) = indexes.iter().max() { x } else { return Ok(cursor); });
        for index in 0..count {
            let offset = index * (max_new_index+1) + self.offset;
            self.index.extend(indexes.iter().map(|x| *x+offset));
        }
        self.offset += count * (max_new_index+1);
        Ok(cursor)
    }

    pub(super) fn add(&mut self, handle: &AttribHandle, cursor: &ProcessStanzaElementsEntryCursor, values: &[f32]) -> Result<(),Error> {
        let position = *cursor.0.get(handle);
        let mut target = self.attribs.get_mut(handle).get();
        if position > target.len() {
            return Err(Error::fatal("cursor after end"));
        } else if position < target.len() {
            target.splice(position..(position+values.len()),values.iter().cloned());
        } else {
            target.extend_from_slice(values);
        }
        Ok(())
    }

    pub(super) async fn make_stanza(&self, values: &KeyedData<AttribHandle,Attribute>, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<Option<ProcessStanza>,Error> {
        let out = ProcessStanza::new_elements(gl,&self.index,values,&self.attribs).await?;
        Ok(out)
    }
}
