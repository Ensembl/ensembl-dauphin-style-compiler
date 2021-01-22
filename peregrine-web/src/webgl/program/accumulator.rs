use anyhow::{ anyhow as err };
use super::attribute::{ Attribute, AttribHandle, AttributeValues };
use super::keyed::{ KeyedValues, KeyedData, KeyedKeys, KeyedDataMaker };
use web_sys::{ WebGlBuffer, WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;

const LIMIT : u16 = 16384;

fn create_index_buffer(context: &WebGlRenderingContext, values: &[u16]) -> anyhow::Result<WebGlBuffer> {
    let buffer = context.create_buffer().ok_or(err!("failed to create buffer"))?;
    // After `Int16Array::view` be very careful not to do any memory allocations before it's dropped.
    unsafe {
        let value_array = js_sys::Uint16Array::view(values);
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &value_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        drop(value_array);
    }
    handle_context_errors(context)?;
    Ok(buffer)
}

pub(crate) struct AccumulatedRun {
    attribs: KeyedData<AttribHandle,AttributeValues>,
    index: WebGlBuffer,
    len: usize
}

impl AccumulatedRun {
    pub(crate) fn activate(&self, context: &WebGlRenderingContext) -> anyhow::Result<usize> {
        context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&self.index));
        handle_context_errors(context)?;
        for attrib in self.attribs.values() {
            attrib.activate(context)?;
        }
        Ok(self.len)
    }

    pub(crate) fn delete(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        context.delete_buffer(Some(&self.index));
        handle_context_errors(context)?;
        for attrib in self.attribs.values_mut() {
            attrib.delete(context)?;
        }
        Ok(())
    }
}

struct AccumulatorEntry {
    attribs: KeyedData<AttribHandle,Vec<f32>>,
    index: Vec<u16>,
    values_count: u16
}

impl AccumulatorEntry {
    fn new(maker: &KeyedDataMaker<'static,AttribHandle,Vec<f32>>) -> AccumulatorEntry {
        AccumulatorEntry {
            attribs: maker.make(),
            index: vec![],
            values_count: 0
        }
    } 

    fn add_values(&mut self, handle: &AttribHandle, mut values: Vec<f32>) {
        self.attribs.get_mut(handle).append(&mut values);
    }

    fn space(&self, size: u16) -> bool {
        self.values_count + size < LIMIT
    }

    fn add_indexes(&mut self, indexes: &[u16], values_count: u16) -> u16 {
        let values_count = self.values_count;
        let offset = self.index.len() as u16;
        self.index.extend(indexes.iter().map(|x| x+values_count));
        self.values_count += values_count;
        offset
    }

    fn make(self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> anyhow::Result<AccumulatedRun> {
        Ok(AccumulatedRun {
            index: create_index_buffer(context,&self.index)?,
            len: self.index.len(),
            attribs: self.attribs.into(|k,v| AttributeValues::new2(values.get(&k).clone(),v,context))?
        })
    }
}

pub(crate) struct AccumulatorValues<'a>(&'a mut Accumulator);

impl<'a> AccumulatorValues<'a> {
    fn new(accumulator: &'a mut Accumulator, indexes: &[u16]) -> AccumulatorValues<'a> {
        let values_count = *indexes.iter().max().unwrap_or(&0)+1;
        accumulator.ensure_space(values_count);
        accumulator.entry().add_indexes(indexes,values_count);
        AccumulatorValues(accumulator)
    }

    pub(crate) fn add(&mut self, handle: &AttribHandle, values: Vec<f32>) {
        self.0.entry().add_values(handle,values);
    }
}

pub(crate) struct Accumulator {
    entries: Vec<AccumulatorEntry>,
    attribs: KeyedValues<AttribHandle,Attribute>,
    maker: KeyedDataMaker<'static,AttribHandle,Vec<f32>>
}

impl Accumulator {
    pub(crate) fn new(attribs: KeyedValues<AttribHandle,Attribute>) -> Accumulator {
        let maker = attribs.keys().make_maker(|| vec![]);
        Accumulator {
            entries: vec![],
            attribs: attribs,
            maker
        }
    }

    fn ensure_space(&mut self, size: u16) {
        if !self.entries.last().map(|x| x.space(size)).unwrap_or(false) {
            self.entries.push(AccumulatorEntry::new(&self.maker));
        }
    }

    fn entry(&mut self) -> &mut AccumulatorEntry {
        self.entries.last_mut().unwrap()
    }

    pub(super) fn get_attrib_handle(&mut self, name: &str) -> anyhow::Result<AttribHandle> {
        self.attribs.get_handle(name)
    }

    pub(crate) fn add_values<'a>(&'a mut self,indexes: &[u16]) -> AccumulatorValues<'a> {
        AccumulatorValues::new(self,indexes)
    }

    pub(super) fn make(mut self, context: &WebGlRenderingContext) -> anyhow::Result<Vec<AccumulatedRun>> {
        let data = self.attribs.data();
        Ok(self.entries.drain(..).map(|x| x.make(&data,context)).collect::<Result<_,_>>()?)
    }
}
