use peregrine_data::{Asset, Assets, BackendNamespace };
use keyed::keyed_handle;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ CanvasAndContext };
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
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
        self.bytes = asset.bytes().ok_or_else(|| Message::BadAsset(format!("missing asset: {}",name)))?.data_as_bytes().map_err(|_| Message::BadAsset("expected bytes".to_string()))?.clone();
        self.width = asset.metadata_u32("width").ok_or_else(|| Message::BadAsset(format!("missing width: {}",name)))?;
        self.height = asset.metadata_u32("height").ok_or_else(|| Message::BadAsset(format!("missing height: {}",name)))?;
        self.scale = asset.metadata_u32("scale").unwrap_or(100);
        Ok(())
    }

    fn new(assets: &Assets, channel: &BackendNamespace, name: &str) -> Result<Bitmap,Message> {
        let mut out = Bitmap {
            name: name.to_string(),
            width: 0,
            height: 0,
            scale: 100,
            bytes: Arc::new(vec![])
        };
        let asset = assets.get(Some(channel),name).ok_or_else(|| Message::BadAsset(format!("missing asset: {}",name)))?;
        out.set_from_asset(&asset,name)?;
        Ok(out)
    }
}

fn dpr_round(size: u32, dpr: f32, scale: u32) -> u32 {
    ( ((size*100/scale) as f32) * dpr ).round() as u32
}

impl FlatDrawingItem for Bitmap {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Error> {
        let dpr = gl.device_pixel_ratio();
        Ok((dpr_round(self.width,dpr,self.scale),dpr_round(self.height,dpr,self.scale)))
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Error> { Ok((PAD,PAD)) }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
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

    pub fn add_bitmap(&mut self, channel: &BackendNamespace, asset: &str) -> Result<BitmapHandle,Message> {
        Ok(self.manager.add(Bitmap::new(&self.assets,channel,asset)?))
    }

    pub(crate) async fn calculate_requirements(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, allocator: &mut FlatPositionManager) -> Result<(),Error> {
        self.manager.calculate_requirements(&mut *lock!(gl),allocator)
    }

    pub(crate) fn manager(&mut self) -> &mut FlatDrawingManager<BitmapHandle,Bitmap> { &mut self.manager }
}
