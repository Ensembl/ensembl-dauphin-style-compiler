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

const STAMP : u32 = 32;
const PAD : u32 = 8;

fn pad(z: (u32,u32)) -> (u32,u32) {
    (z.0+PAD,z.1+PAD)
}

#[derive(Hash)]
pub(crate) enum Heraldry {
    Stripe(DirectColour,DirectColour,u32,u32)
}

impl FlatDrawingItem for Heraldry {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok(match self {
            Heraldry::Stripe(_,_,_,count) => (STAMP*(*count),STAMP)
        })
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok(match  self {
            &mut Heraldry::Stripe(_,_,_,_) => (PAD,PAD)
        })
    }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        match self {
            Heraldry::Stripe(a,b,prop,count) => {
                let p = (PAD+STAMP) * (*prop) / 100;
                for i in 0..*count {
                    let offset = i*STAMP;
                    canvas.rectangle(pad(mask_origin),size,&DirectColour(0,0,0))?;
                    canvas.rectangle(pad(text_origin),size,b)?;
                    canvas.path(text_origin,&[
                        pad((offset,      0)),
                        pad((offset+p,    0)),
                        pad((offset+STAMP,STAMP-p)),
                        pad((offset+STAMP,STAMP))
                    ],a)?;
                    canvas.path(text_origin,&[
                        pad((offset,  STAMP-p)),
                        pad((offset+p,STAMP)),
                        pad((offset,  STAMP))
                    ],a)?;
                }
                /* bleed */
                let x_far = *count * STAMP + 2*PAD;
                let y_far = STAMP+2*PAD;
                canvas.rectangle(text_origin,(p+PAD,PAD), a)?;
                canvas.rectangle((text_origin.0+x_far-PAD,text_origin.1+y_far-PAD-p),(PAD,PAD+p),a)?;
                canvas.rectangle((text_origin.0,text_origin.1+y_far-PAD-p),(PAD+p,PAD+p),a)?;
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
