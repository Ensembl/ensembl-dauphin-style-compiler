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
    index: Option<WebGlBuffer>,
    len: usize
}

impl AccumulatedRun {
    pub(crate) fn activate(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        if let Some(index) = &self.index {
            context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,Some(index));
            handle_context_errors(context)?;
        }
        for attrib in self.attribs.values() {
            attrib.activate(context)?;
        }
        Ok(())
    }

    pub(crate) fn deactivate(&self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,None);
        handle_context_errors(context)?;
        Ok(())
    }

    pub fn draw(&self, context: &WebGlRenderingContext, method: u32) -> anyhow::Result<()> {
        if self.index.is_some() {
            context.draw_elements_with_i32(method,self.len as i32,WebGlRenderingContext::UNSIGNED_SHORT,0);
            handle_context_errors(context)?;
        } else {
            context.draw_arrays(method,0,self.len as i32);
            handle_context_errors(context)?;
        }
        Ok(())
    }

    pub fn discard(&mut self, context: &WebGlRenderingContext) -> anyhow::Result<()> {
        if let Some(index) = &self.index {
            context.delete_buffer(Some(index));
            handle_context_errors(context)?;
        }
       for attrib in self.attribs.values_mut() {
            attrib.discard(context)?;
        }
        Ok(())
    }
}

struct AccumulatorElements {
    attribs: KeyedData<AttribHandle,Vec<f64>>,
    index: Vec<u16>
}

impl AccumulatorElements {
    fn new(maker: &KeyedDataMaker<'static,AttribHandle,Vec<f64>>) -> AccumulatorElements {
        AccumulatorElements {
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

    fn make(self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> anyhow::Result<Option<AccumulatedRun>> {
        if self.index.len() > 0 {
            Ok(Some(AccumulatedRun {
                index: Some(create_index_buffer(context,&self.index)?),
                len: self.index.len(),
                attribs: self.attribs.map_into(|k,v| AttributeValues::new(values.get(&k),v,context))?
            }))
        } else {
            Ok(None)
        }
    }
}

pub trait AccumulatorAddable {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()>;
    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub(crate) struct AccumulatorArray {
    attribs: Rc<RefCell<KeyedData<AttribHandle,Vec<f64>>>>,
    len: usize,
    active: Rc<RefCell<bool>>
}

impl AccumulatorArray {
    fn new(accumulator: &Rc<RefCell<bool>>, maker: &KeyedDataMaker<'static,AttribHandle,Vec<f64>>, len: usize) -> AccumulatorArray {
        AccumulatorArray {
            attribs: Rc::new(RefCell::new(maker.make())),
            active: accumulator.clone(),
            len
        }
    }

    fn make(&self, values: &KeyedData<AttribHandle,Attribute>, context: &WebGlRenderingContext) -> anyhow::Result<Option<AccumulatedRun>> {
        if self.len > 0 {
            Ok(Some(AccumulatedRun {
                index: None,
                len: self.len,
                attribs: self.attribs.replace(KeyedData::new()).map_into(|k,v| AttributeValues::new(values.get(&k),v,context))?
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn close(&mut self) {
        *self.active.borrow_mut() = false;
    }
}

impl AccumulatorAddable for AccumulatorArray {
    fn add(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        self.attribs.borrow_mut().get_mut(handle).extend_from_slice(&values);
        Ok(())
    }

    fn add_n(&mut self, handle: &AttribHandle, values: Vec<f64>) -> anyhow::Result<()> {
        let values_size = values.len();
        let mut offset = 0;
        let mut remaining = self.len;
        while remaining > 0 {
            let mut real_count = remaining;
            if offset+real_count > values_size { real_count = values_size-offset; }
            self.attribs.borrow_mut().get_mut(handle).extend_from_slice(&values[offset..(offset+real_count)]);
            remaining -= real_count;
            offset += real_count;
            if offset == values_size { offset = 0; }
        }
        Ok(())
    }
}

pub struct AccumulatorCampaign {
    elements: Vec<(Rc<RefCell<AccumulatorElements>>,usize)>,
    tuple_size: usize,
    count: usize,
    active: Rc<RefCell<bool>>
}

impl AccumulatorCampaign {
    fn new(accumulator: &mut Accumulator, count: usize, indexes: &[u16]) -> AccumulatorCampaign {
        let mut out = AccumulatorCampaign {
            tuple_size: indexes.iter().max().map(|x| x+1).unwrap_or(0) as usize,
            elements: vec![],
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
            let entry = accumulator.elements().clone();
            let mut space = entry.borrow().space(self.tuple_size);
            if space > remaining { space = remaining; }
            if space > 0 {
                bases.push(entry.borrow().base());
                self.elements.push((entry,space));
            }
            remaining -= space;
            if remaining > 0 {
                accumulator.make_elements();
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

impl AccumulatorAddable for AccumulatorCampaign {
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

pub struct Accumulator {
    elements: Vec<Rc<RefCell<AccumulatorElements>>>,
    arrays: Vec<AccumulatorArray>,
    maker: KeyedDataMaker<'static,AttribHandle,Vec<f64>>,
    active: Rc<RefCell<bool>>

}

impl Accumulator {
    pub(crate) fn new(attribs: &KeyedValues<AttribHandle,Attribute>) -> Accumulator {
        let maker = attribs.keys().make_maker(|| vec![]);
        Accumulator {
            maker,
            elements: vec![],
            arrays: vec![],
            active: Rc::new(RefCell::new(false))
        }
    }

    fn active(&self) -> &Rc<RefCell<bool>> { &self.active }

    fn make_elements(&mut self) {
        self.elements.push(Rc::new(RefCell::new(AccumulatorElements::new(&self.maker))));
    }

    fn elements<'a>(&'a mut self) -> &Rc<RefCell<AccumulatorElements>> {
        self.elements.last_mut().unwrap()
    }

    pub(crate) fn make_campaign(&mut self, count: usize, indexes: &[u16]) -> anyhow::Result<AccumulatorCampaign> {
        if *self.active.borrow() {
            bail!("can only have one active campaign/array at once");
        }
        if self.elements.len() == 0 {
            self.make_elements();
        }
        *self.active.borrow_mut() = true;
        Ok(AccumulatorCampaign::new(self,count,indexes))
    }

    pub(crate) fn make_array(&mut self, len: usize) -> anyhow::Result<AccumulatorArray> {
        if *self.active.borrow() {
            bail!("can only have one active campaign/array at once");
        }
        let out = AccumulatorArray::new(&self.active,&self.maker,len);
        self.arrays.push(out.clone());
        *self.active.borrow_mut() = true;
        Ok(out)
    }

    pub(super) fn make(&self, context: &WebGlRenderingContext, attribs: &KeyedValues<AttribHandle,Attribute>) -> anyhow::Result<Vec<AccumulatedRun>> {
        if *self.active.borrow() {
            bail!("can only make when inactive");
        }
        let mut out = self.elements.iter().map(|x| x.replace(AccumulatorElements::new(&self.maker)).make(attribs.data(),context)).collect::<Result<Vec<_>,_>>()?;
        out.append(&mut self.arrays.iter().map(|x| x.make(attribs.data(),context)).collect::<Result<_,_>>()?);
        Ok(out.drain(..).filter(|x| x.is_some()).map(|x| x.unwrap()).collect())
    }
}
