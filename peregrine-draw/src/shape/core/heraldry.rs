use keyed::KeyedData;
use peregrine_data::{ DirectColour };
use keyed::keyed_handle;
use crate::webgl::canvas::flatplotallocator::FlatPositionAllocator;
use crate::webgl::{ CanvasWeave, DrawingFlatsDrawable, FlatId, FlatStore, Flat, FlatPlotRequestHandle };
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use super::texture::CanvasTextureAreas;
use crate::util::message::Message;


keyed_handle!(HeraldryHandle);

pub(crate) enum Heraldry {
    Stripe(DirectColour,DirectColour)
}


impl FlatDrawingItem for Heraldry {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok((16,16))
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        match self {
            Heraldry::Stripe(a,b) => {
                canvas.rectangle(mask_origin,size,&DirectColour(0,0,0))?;
                canvas.rectangle(text_origin,size,a)?;
                canvas.path(text_origin,&[(0,0),(8,0),(16,8),(16,16)],b)?;
                canvas.path(text_origin,&[(0,8),(8,16),(0,16)],b)?;
            }
        }
        Ok(())
    }
}

pub struct DrawingHeraldry(FlatDrawingManager<HeraldryHandle,Heraldry>);

impl DrawingHeraldry {
    pub fn new() -> DrawingHeraldry { DrawingHeraldry(FlatDrawingManager::new()) }

    pub(crate) fn add(&mut self, heraldry: Heraldry) -> HeraldryHandle {
        self.0.add(heraldry)
    }

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionAllocator) -> Result<(),Message> {
        self.0.calculate_requirements(gl,allocator,|_| {})
    }

    pub(crate) fn draw_at_locations(&mut self, store: &mut FlatStore, builder: &DrawingFlatsDrawable, allocator: &mut FlatPositionAllocator) -> Result<(),Message> {
        self.0.draw_at_locations(store,builder,allocator)
    }

    pub(crate) fn canvas_id(&self, builder: &DrawingFlatsDrawable) -> Result<FlatId,Message> {
        self.0.canvas_id(builder)
    }

    pub(crate) fn get_texture_areas(&self, handle: &HeraldryHandle) -> Result<CanvasTextureAreas,Message> {
        self.0.get_texture_areas(handle)
    }
}
