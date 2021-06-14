use std::collections::HashMap;

use keyed::{KeyedData, KeyedHandle};
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ FlatId, FlatStore, Flat, FlatPositionCampaignHandle };
use crate::webgl::global::WebGlGlobal;
use super::texture::CanvasTextureAreas;
use crate::util::message::Message;

pub(crate) trait FlatDrawingItem {
    fn compute_hash(&self) -> Option<u64> { None }
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message>;
    fn padding(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> { Ok((0,0)) }
    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message>;
}

/* here, size and origins are inclusive of padding */
pub(crate) struct FlatBoundary {
    text_origin: Option<(u32,u32)>,
    mask_origin: Option<(u32,u32)>,
    size: Option<(u32,u32)>,
    padding: (u32,u32)
}

impl FlatBoundary {
    fn new() -> FlatBoundary {
        FlatBoundary { text_origin: None, mask_origin: None, size: None, padding: (0,0) }
    }

    fn size_without_padding(&self) -> Result<(u32,u32),Message> {
        self.size.ok_or_else(|| Message::CodeInvariantFailed("texture get size unset".to_string()))
    }

    fn size_with_padding(&self) -> Result<(u32,u32),Message> {
        let size = self.size_without_padding()?;
        Ok((size.0+self.padding.0,size.1+self.padding.1))
    }

    fn set_size(&mut self, size: (u32,u32), padding: (u32,u32)) {
        self.size = Some(size);
        self.padding = padding;
    }

    fn set_origin(&mut self, text: (u32,u32), mask: (u32,u32)) {
        self.text_origin = Some(text);
        self.mask_origin = Some(mask);
    }

    fn pad(&self, v: (u32,u32)) -> (u32,u32) {
        (v.0+self.padding.0,v.1+self.padding.1)
    }

    fn get_texture_areas(&self) -> Result<CanvasTextureAreas,Message> {
        Ok(CanvasTextureAreas::new(
            self.pad(unpack(&self.text_origin)?),
            self.pad(unpack(&self.mask_origin)?),
            unpack(&self.size)?
        ))
    }
}

fn unpack<T: Clone>(data: &Option<T>) -> Result<T,Message> {
    data.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure, t origin".to_string()))
}

pub(crate) struct FlatDrawingManager<H: KeyedHandle,T: FlatDrawingItem> {
    hashed_items: HashMap<u64,H>,
    texts: KeyedData<H,(T,FlatBoundary)>,
    request: Option<FlatPositionCampaignHandle>,
    canvas: Option<FlatId>
}

impl<H: KeyedHandle+Clone,T: FlatDrawingItem> FlatDrawingManager<H,T> {
    pub fn new() -> FlatDrawingManager<H,T> {
        FlatDrawingManager {
            hashed_items: HashMap::new(),
            texts: KeyedData::new(),
            request: None,
            canvas: None
        }
    }

    pub(crate) fn add(&mut self, item: T) -> H {
        let hash = item.compute_hash();
        if let Some(hash) = hash {
            if let Some(old) = self.hashed_items.get(&hash) {
                return old.clone();
            }
        }
        let handle = self.texts.add((item,FlatBoundary::new()));
        if let Some(hash) = hash {
            self.hashed_items.insert(hash,handle.clone());
        }
        handle
    }

    fn calc_sizes<F>(&mut self, gl: &mut WebGlGlobal, sorter: F) -> Result<(),Message>
                where F: FnOnce(&mut Vec<&mut (T,FlatBoundary)>) {
        let mut texts = self.texts.values_mut().collect::<Vec<_>>();
        sorter(&mut texts);
        for v in texts.iter_mut() {
            let size = v.0.calc_size(gl)?;
            let padding = v.0.padding(gl)?;
            v.1.set_size(size,padding);
        }
        Ok(())
    }

    pub(crate) fn calculate_requirements<F>(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionManager, sorter: F) -> Result<(),Message>
                where F: FnOnce(&mut Vec<&mut (T,FlatBoundary)>) {
        self.calc_sizes(gl,sorter)?;
        let mut sizes = vec![];
        for (text,boundary) in self.texts.values_mut() {
            let size = boundary.size_with_padding()?;
            /* mask and text */
            sizes.push(size);
            sizes.push(size);
        }
        self.request = Some(allocator.insert(&sizes));
        Ok(())
    }

    pub(crate) fn draw_at_locations(&mut self, store: &mut FlatStore, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        self.canvas = Some(allocator.canvas()?);
        let mut origins = allocator.origins(self.request.as_ref().unwrap());
        let mut origins_iter = origins.drain(..);
        let canvas_id = allocator.canvas()?;
        let canvas = store.get_mut(&canvas_id)?;
        for (text,boundary) in self.texts.values_mut() {
            let mask_origin = origins_iter.next().unwrap();
            let text_origin = origins_iter.next().unwrap();
            boundary.set_origin(text_origin,mask_origin);
            let size = boundary.size_with_padding()?;
            text.build(canvas,text_origin,mask_origin,size)?;
        }
        Ok(())
    }

    pub(crate) fn canvas_id(&self) -> Result<FlatId,Message> {
        self.canvas.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed(format!("no associated canvas")))
    }

    pub(crate) fn get_texture_areas(&self, handle: &H) -> Result<CanvasTextureAreas,Message> {
        self.texts.get(handle).1.get_texture_areas()
    }
}
