use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use peregrine_data::{ DirectColour };
use keyed::keyed_handle;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ FlatId, FlatStore, Flat };
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use super::texture::CanvasTextureAreas;
use crate::util::message::Message;

keyed_handle!(HeraldryHandle);

#[derive(Hash)]
pub(crate) enum Heraldry {
    Stripe(DirectColour,DirectColour,u32)
}

impl FlatDrawingItem for Heraldry {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok(match self {
            Heraldry::Stripe(_,_,count) => (16*(*count),16)
        })
    }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        match self {
            Heraldry::Stripe(a,b,count) => {
                for i in 0..*count {
                    let offset = i*16;
                    canvas.rectangle(mask_origin,size,&DirectColour(0,0,0))?;
                    canvas.rectangle(text_origin,size,a)?;
                    canvas.path(text_origin,&[(offset,0),(offset+8,0),(offset+16,8),(offset+16,16)],b)?;
                    canvas.path(text_origin,&[(offset,8),(offset+8,16),(offset,16)],b)?;
                }
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

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        self.0.calculate_requirements(gl,allocator,|_| {})
    }

    pub(crate) fn draw_at_locations(&mut self, store: &mut FlatStore, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        self.0.draw_at_locations(store,allocator)
    }

    pub(crate) fn canvas_id(&self) -> Result<FlatId,Message> {
        self.0.canvas_id()
    }

    pub(crate) fn get_texture_areas(&self, handle: &HeraldryHandle) -> Result<CanvasTextureAreas,Message> {
        self.0.get_texture_areas(handle)
    }
}
