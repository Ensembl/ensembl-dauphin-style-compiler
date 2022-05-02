use peregrine_data::{Asset, Assets, DirectColour };
use keyed::keyed_handle;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ Flat };
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::util::message::Message;

// TODO padding measurements!

keyed_handle!(BitmapHandle);

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

pub(crate) struct Bitmap {
    name: String,
    width: u32,
    height: u32,
    scale: u32,
    bytes: Arc<Vec<u8>>
}

impl Bitmap {
    fn set_from_asset(&mut self, asset: &Asset, name: &str) -> Result<(),Message> {
        self.bytes = asset.bytes().ok_or_else(|| Message::BadAsset(format!("missing asset: {}",name)))?.data().clone();
        self.width = asset.metadata_u32("width").ok_or_else(|| Message::BadAsset(format!("missing width: {}",name)))?;
        self.height = asset.metadata_u32("height").ok_or_else(|| Message::BadAsset(format!("missing height: {}",name)))?;
        self.scale = asset.metadata_u32("scale").unwrap_or(100);
        Ok(())
    }

    fn new(assets: &Assets, name: &str) -> Result<Bitmap,Message> {
        let mut out = Bitmap {
            name: name.to_string(),
            width: 0,
            height: 0,
            scale: 100,
            bytes: Arc::new(vec![])
        };
        let asset = assets.get(name).ok_or_else(|| Message::BadAsset(format!("missing asset: {}",name)))?;
        out.set_from_asset(&asset,name)?;
        Ok(out)
    }
}

impl FlatDrawingItem for Bitmap {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok((self.width*self.scale/100,self.height*self.scale/100))
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> { Ok((PAD,PAD)) }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        canvas.rectangle(pad(mask_origin),size,&DirectColour(0,0,0,255),false)?;
        canvas.draw_png(Some(self.name.clone()),pad(text_origin),size,&self.bytes)?;
        Ok(())
    }
}

pub struct DrawingBitmap {
    manager: FlatDrawingManager<BitmapHandle,Bitmap>,
    assets: Assets
}

impl DrawingBitmap {
    pub fn new(assets: &Assets) -> DrawingBitmap {
        DrawingBitmap {
            manager: FlatDrawingManager::new(),
            assets: assets.clone()
        }
    }

    pub fn add_bitmap(&mut self, asset: &str) -> Result<BitmapHandle,Message> {
        Ok(self.manager.add(Bitmap::new(&self.assets,asset)?))
    }

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        self.manager.calculate_requirements(gl,allocator)
    }

    pub(crate) fn manager(&mut self) -> &mut FlatDrawingManager<BitmapHandle,Bitmap> { &mut self.manager }
}
