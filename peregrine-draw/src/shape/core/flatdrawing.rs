use keyed::{KeyedData, KeyedHandle};
use crate::webgl::canvas::flatplotallocator::FlatPositionAllocator;
use crate::webgl::{ DrawingFlatsDrawable, FlatId, FlatStore, Flat, FlatPlotRequestHandle };
use crate::webgl::global::WebGlGlobal;
use super::texture::CanvasTextureAreas;
use crate::util::message::Message;

pub(crate) trait FlatDrawingItem {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message>;
    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message>;
}

pub(crate) struct FlatBoundary {
    text_origin: Option<(u32,u32)>,
    mask_origin: Option<(u32,u32)>,
    size: Option<(u32,u32)>,
}

impl FlatBoundary {
    fn new() -> FlatBoundary {
        FlatBoundary { text_origin: None, mask_origin: None, size: None }
    }

    fn size(&self) -> Result<(u32,u32),Message> {
        self.size.ok_or_else(|| Message::CodeInvariantFailed("texture get size unset".to_string()))
    }

    fn set_size(&mut self, size: (u32,u32)) {
        self.size = Some(size);
    }

    fn set_origin(&mut self, text: (u32,u32), mask: (u32,u32)) {
        self.text_origin = Some(text);
        self.mask_origin = Some(mask);
    }

    fn get_texture_areas(&self) -> Result<CanvasTextureAreas,Message> {
        Ok(CanvasTextureAreas::new(
            self.text_origin.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure, t origin".to_string()))?,
            self.mask_origin.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure. m origin".to_string()))?,
            self.size.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure. size A".to_string()))?
        ))
    }
}

pub(crate) struct FlatDrawingManager<H: KeyedHandle,T: FlatDrawingItem> {
    texts: KeyedData<H,(T,FlatBoundary)>,
    request: Option<FlatPlotRequestHandle>,
    canvas: Option<FlatId>
}

impl<H: KeyedHandle,T: FlatDrawingItem> FlatDrawingManager<H,T> {
    pub fn new() -> FlatDrawingManager<H,T> {
        FlatDrawingManager {
            texts: KeyedData::new(),
            request: None,
            canvas: None
        }
    }

    pub(crate) fn add(&mut self, item: T) -> H {
        self.texts.add((item,FlatBoundary::new()))
    }

    fn calc_sizes<F>(&mut self, gl: &mut WebGlGlobal, sorter: F) -> Result<(),Message>
                where F: FnOnce(&mut Vec<&mut (T,FlatBoundary)>) {
        let mut texts = self.texts.values_mut().collect::<Vec<_>>();
        sorter(&mut texts);
        for v in texts.iter_mut() {
            let size = v.0.calc_size(gl)?;
            v.1.set_size(size);
        }
        Ok(())
    }

    pub(crate) fn calculate_requirements<F>(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionAllocator, sorter: F) -> Result<(),Message>
                where F: FnOnce(&mut Vec<&mut (T,FlatBoundary)>) {
        self.calc_sizes(gl,sorter)?;
        let mut sizes = vec![];
        for (text,boundary) in self.texts.values_mut() {
            let size = boundary.size()?;
            /* mask and text */
            sizes.push(size);
            sizes.push(size);
        }
        self.request = Some(allocator.insert(&sizes));
        Ok(())
    }

    pub(crate) fn draw_at_locations(&mut self, store: &mut FlatStore, builder: &DrawingFlatsDrawable, allocator: &mut FlatPositionAllocator) -> Result<(),Message> {
        self.canvas = Some(allocator.canvas()?);
        let mut origins = allocator.origins(self.request.as_ref().unwrap());
        let mut origins_iter = origins.drain(..);
        let canvas_id = allocator.canvas()?;
        let canvas = store.get_mut(&canvas_id)?;
        for (text,boundary) in self.texts.values_mut() {
            let mask_origin = origins_iter.next().unwrap();
            let text_origin = origins_iter.next().unwrap();
            boundary.set_origin(text_origin,mask_origin);
            let size = boundary.size()?;
            text.build(canvas,text_origin,mask_origin,size)?;
        }
        Ok(())
    }

    pub(crate) fn canvas_id(&self, builder: &DrawingFlatsDrawable) -> Result<FlatId,Message> {
        self.canvas.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed(format!("no associated canvas")))
    }

    pub(crate) fn get_texture_areas(&self, handle: &H) -> Result<CanvasTextureAreas,Message> {
        self.texts.get(handle).1.get_texture_areas()
    }
}
