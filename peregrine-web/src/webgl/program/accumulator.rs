use anyhow::{ anyhow as err, bail };
use super::attribute::{ Attribute, AttribHandle, AttributeValues };
use super::keyed::{ KeyedValues, KeyedData, KeyedDataMaker };
use web_sys::{ WebGlBuffer, WebGlRenderingContext };
use crate::webgl::util::handle_context_errors;
use std::rc::Rc;
use std::cell::RefCell;

const LIMIT : usize = 16384;

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
    attribs: KeyedData<AttribHandle,Vec<f64>>,
    index: Vec<u16>
}

impl AccumulatorEntry {
    fn new(maker: &KeyedDataMaker<'static,AttribHandle,Vec<f64>>) -> AccumulatorEntry {
        AccumulatorEntry {
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

    fn make(self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> anyhow::Result<AccumulatedRun> {
        Ok(AccumulatedRun {
            index: create_index_buffer(context,&self.index)?,
            len: self.index.len(),
            attribs: self.attribs.map_into(|k,v| AttributeValues::new(values.get(&k),v,context))?
        })
    }
}

pub struct AccumulatorCampaign {
    entries: Vec<(Rc<RefCell<AccumulatorEntry>>,usize)>,
    tuple_size: usize,
    count: usize,
    active: Rc<RefCell<bool>>
}

impl AccumulatorCampaign {
    fn new(accumulator: &mut Accumulator, count: usize, indexes: &[u16]) -> AccumulatorCampaign {
        let mut out = AccumulatorCampaign {
            tuple_size: indexes.iter().max().map(|x| x+1).unwrap_or(0) as usize,
            entries: vec![],
            count,
            active: accumulator.active().clone()
        };
        let bases = out.allocate_entries(accumulator);
        out.add_indexes(indexes,&bases);
        out
    }

    fn allocate_entries(&mut self, accumulator: &mut Accumulator) -> Vec<usize> {
        let mut bases = vec![];
        let mut remaining = self.count;
        while remaining > 0 {
            let entry = accumulator.entry().clone();
            let mut space = entry.borrow().space(self.tuple_size);
            if space > remaining { space = remaining; }
            if space > 0 {
                bases.push(entry.borrow().base());
                self.entries.push((entry,space));
            }
            remaining -= space;
            if remaining > 0 {
                accumulator.make_entry();
            }
        }
        bases
    }

    fn add_indexes(&mut self, indexes: &[u16], bases: &[usize]) {
        for (i,(entry,count)) in self.entries.iter().enumerate() {
            let these_indexes : Vec<u16> = indexes.iter().map(|x| *x+(bases[i] as u16)).collect();
            entry.borrow_mut().add_indexes(&these_indexes,*count);
        }
    }

    pub(crate) fn add(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        let array_size = self.tuple_size * self.count;
        if values.len() != array_size {
            bail!("incorrect array length: expected {} got {}",array_size,values.len());
        }
        let mut offset = 0;
        for (entry,count) in &mut self.entries {
            let slice_size = *count*self.tuple_size;
            entry.borrow_mut().add(handle,&values[offset..(offset+slice_size)]);
            offset += slice_size;
        }
        Ok(())
    }

    pub(crate) fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        let values_size = values.len();
        let mut offset = 0;
        for (entry,count) in &mut self.entries {
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

    pub(crate) fn close(&mut self) {
        *self.active.borrow_mut() = false;
    }
}

pub struct Accumulator {
    entries: Vec<Rc<RefCell<AccumulatorEntry>>>,
    maker: KeyedDataMaker<'static,AttribHandle,Vec<f64>>,
    active: Rc<RefCell<bool>>

}

impl Accumulator {
    pub(crate) fn new(attribs: &KeyedValues<AttribHandle,Attribute>) -> Accumulator {
        let maker = attribs.keys().make_maker(|| vec![]);
        Accumulator {
            maker,
            entries: vec![],
            active: Rc::new(RefCell::new(false))
        }
    }

    fn active(&self) -> &Rc<RefCell<bool>> { &self.active }

    fn make_entry(&mut self) {
        self.entries.push(Rc::new(RefCell::new(AccumulatorEntry::new(&self.maker))));
    }

    fn entry<'a>(&'a mut self) -> &Rc<RefCell<AccumulatorEntry>> {
        self.entries.last_mut().unwrap()
    }

    pub(crate) fn make_campaign(&mut self, count: usize, indexes: &[u16]) -> anyhow::Result<AccumulatorCampaign> {
        if *self.active.borrow() {
            bail!("can only have one active campaign at once");
        }        
        if self.entries.len() == 0 {
            self.make_entry();
        }
        *self.active.borrow_mut() = true;
        Ok(AccumulatorCampaign::new(self,count,indexes))
    }

    pub(super) fn make(&self, context: &WebGlRenderingContext, attribs: &KeyedValues<AttribHandle,Attribute>) -> anyhow::Result<Vec<AccumulatedRun>> {
        Ok(self.entries.iter().map(|x| x.replace(AccumulatorEntry::new(&self.maker)).make(attribs.data(),context)).collect::<Result<_,_>>()?)
    }
}
