use peregrine_data::{ Pen, DirectColour };
use keyed::keyed_handle;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ CanvasWeave, Flat };
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::util::message::Message;

// TODO padding measurements!

keyed_handle!(BitmapHandle);

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

pub(crate) struct Bitmap {
    asset: String,
}

impl Bitmap {
    fn new(asset: &str) -> Bitmap {
        Bitmap { asset: asset.to_string() }
    }
}

impl FlatDrawingItem for Bitmap {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        todo!()
        /*
        let document = gl.document().clone();
        let canvas = gl.flat_store_mut().scratch(&document,&CanvasWeave::Crisp,(100,100))?;
        canvas.set_font(&self.pen)?;
        canvas.measure(&self.text)
        */
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> { Ok((PAD,PAD)) }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.asset.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        todo!();
        /*
        canvas.set_font(&self.pen)?;
        let background = self.background.clone().unwrap_or_else(|| DirectColour(255,255,255,255));
        canvas.text(&self.text,pad(text_origin),size,&self.colour,&background)?;
        if self.background.is_some() {
            canvas.rectangle(pad(mask_origin),size, &DirectColour(0,0,0,255))?;
        } else{
            canvas.text(&self.text,pad(mask_origin),size,&DirectColour(0,0,0,255),&DirectColour(255,255,255,255))?;
        }
        Ok(())
        */
    }
}

pub struct DrawingBitmap(FlatDrawingManager<BitmapHandle,Bitmap>);

impl DrawingBitmap {
    pub fn new() -> DrawingBitmap { DrawingBitmap(FlatDrawingManager::new()) }

    pub fn add_bitmap(&mut self, asset: &str) -> BitmapHandle {
        self.0.add(Bitmap::new(asset))
    }

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        self.0.calculate_requirements(gl,allocator)
    }

    pub(crate) fn manager(&mut self) -> &mut FlatDrawingManager<BitmapHandle,Bitmap> { &mut self.0 }
}
